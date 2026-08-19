use crate::{Kafence, KafenceProducerContract, StrongConsistencyPath};
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use rdkafka::ClientConfig;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::error::RDKafkaErrorCode;
use std::convert::Infallible;
use std::net::TcpListener;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[tokio::test]
async fn producer_test() {
    let broker = "localhost:9092";
    let run_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let topic = format!("kafka_topic_{run_id}");
    let consumer_group = format!("kafka_group_{run_id}");

    create_topic_if_not_exists(broker, &topic, 2).await.unwrap();

    // DSL
    // -----

    // Services
    // Service 1
    // ---------
    let service_1_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let service_1_addr = service_1_listener.local_addr().unwrap();
    let service_1_url = format!("http://{}", service_1_addr);
    let uuid = uuid::Uuid::new_v4();
    let rocksdb_path_1 = std::env::temp_dir()
        .join(format!("kafence-{run_id}-{uuid}"))
        .to_string_lossy()
        .into_owned();

    let kaference_1 = Kafence::new()
        .with_brokers(broker.to_string())
        .with_topic(&topic)
        .with_consumer_group(&consumer_group)
        .with_partitions(2)
        .with_rocksdb_path(&rocksdb_path_1)
        .with_service_url(&service_1_url)
        .build();
    let service_1 = tokio::spawn(run_server(service_1_listener, Arc::clone(&kaference_1)));

    // Service 2
    // ---------
    let service_2_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let service_2_addr = service_2_listener.local_addr().unwrap();
    let service_2_url = format!("http://{}", service_2_addr);

    let uuid = uuid::Uuid::new_v4();
    let rocksdb_path_2 = std::env::temp_dir()
        .join(format!("kafence-{run_id}-{uuid}"))
        .to_string_lossy()
        .into_owned();

    let kaference_2 = Kafence::new()
        .with_brokers(broker.to_string())
        .with_topic(&topic)
        .with_consumer_group(&consumer_group)
        .with_partitions(2)
        .with_rocksdb_path(&rocksdb_path_2)
        .with_service_url(&service_2_url)
        .build();
    let service_2 = tokio::spawn(run_server(service_2_listener, Arc::clone(&kaference_2)));

    tokio::time::sleep(Duration::from_secs(5)).await;

    wait_for_routes(&kaference_1, 2).await;
    wait_for_routes(&kaference_2, 2).await;

    let local_partition = partition_owned_by(&kaference_1, &service_1_url);
    let proxy_partition = partition_not_owned_by(&kaference_1, &service_1_url);
    let proxy_target = route_owner(&kaference_1, proxy_partition);

    let local_key = key_for_partition("local_record_key", local_partition, kaference_1.partitions);
    let proxy_key = key_for_partition("proxy_record_key", proxy_partition, kaference_1.partitions);

    let client = Client::new();

    let local_response =
        post_record(&client, service_1_addr, &local_key, "hello local world").await;
    assert_eq!(StatusCode::ACCEPTED, local_response.status());
    assert_eq!(
        "local",
        local_response
            .headers()
            .get("x-kafence-route")
            .unwrap()
            .to_str()
            .unwrap()
    );

    let proxy_response =
        post_record(&client, service_1_addr, &proxy_key, "hello proxy world").await;
    assert_eq!(StatusCode::ACCEPTED, proxy_response.status());
    assert_eq!(
        "proxy",
        proxy_response
            .headers()
            .get("x-kafence-route")
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(
        proxy_target,
        proxy_response
            .headers()
            .get("x-kafence-proxy-target")
            .unwrap()
            .to_str()
            .unwrap()
    );

    tokio::time::sleep(Duration::from_secs(5)).await;

    service_1.abort();
    service_2.abort();
}

struct TestServiceState {
    topic: String,
    producer: crate::KafenceProducer,
}

pub async fn run_server(listener: TcpListener, kafence: Arc<Kafence>) {
    println!("Preparing Service...");
    kafence.stream().await.unwrap();
    let state = Arc::new(TestServiceState {
        topic: kafence.topic.clone(),
        producer: kafence.producer().unwrap(),
    });

    let server = Server::from_tcp(listener)
        .unwrap()
        .serve(make_service_fn(move |_conn| {
            let state = Arc::clone(&state);
            async move {
                let state = Arc::clone(&state);
                Ok::<_, Infallible>(service_fn(move |request| {
                    create_service(request, Arc::clone(&state))
                }))
            }
        }));
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

async fn create_service(
    req: Request<Body>,
    state: Arc<TestServiceState>,
) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Service is running");
        }
        (&Method::POST, "/") => {
            let key = req
                .headers()
                .get("record-key")
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("record_key_{}", Uuid::new_v4()));

            match hyper::body::to_bytes(req.into_body()).await {
                Ok(body) => {
                    let value = String::from_utf8_lossy(&body).into_owned();
                    match state
                        .producer
                        .strong_consistency(&state.topic, key.clone(), value)
                        .await
                    {
                        Ok(StrongConsistencyPath::Local) => {
                            response
                                .headers_mut()
                                .insert("x-kafence-route", HeaderValue::from_static("local"));
                            *response.status_mut() = StatusCode::ACCEPTED;
                            *response.body_mut() = Body::from(format!("published {key}"));
                        }
                        Ok(StrongConsistencyPath::Proxied(target_host)) => {
                            response
                                .headers_mut()
                                .insert("x-kafence-route", HeaderValue::from_static("proxy"));
                            response.headers_mut().insert(
                                "x-kafence-proxy-target",
                                HeaderValue::from_str(&target_host).unwrap(),
                            );
                            *response.status_mut() = StatusCode::ACCEPTED;
                            *response.body_mut() = Body::from(format!("proxied {key}"));
                        }
                        Err(e) => {
                            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                            *response.body_mut() = Body::from(format!("publish failed: {e}"));
                        }
                    }
                }
                Err(e) => {
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    *response.body_mut() = Body::from(format!("invalid body: {e}"));
                }
            }
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}

async fn post_record(
    client: &Client<hyper::client::HttpConnector>,
    addr: std::net::SocketAddr,
    key: &str,
    value: &str,
) -> Response<Body> {
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("http://{addr}/"))
        .header("record-key", key)
        .body(Body::from(value.to_string()))
        .unwrap();

    client.request(request).await.unwrap()
}

async fn wait_for_routes(kafence: &Arc<Kafence>, expected_routes: usize) {
    for _ in 0..100 {
        let route_count = kafence.route_table.read().unwrap().len();
        if route_count >= expected_routes {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    panic!(
        "route table was not ready. expected_routes={expected_routes} actual={}",
        kafence.route_table.read().unwrap().len()
    );
}

fn partition_owned_by(kafence: &Kafence, service_url: &str) -> i32 {
    (0..kafence.partitions)
        .find(|partition| route_owner(kafence, *partition) == service_url)
        .expect("service must own at least one partition")
}

fn partition_not_owned_by(kafence: &Kafence, service_url: &str) -> i32 {
    (0..kafence.partitions)
        .find(|partition| route_owner(kafence, *partition) != service_url)
        .expect("service must have at least one remote partition")
}

fn route_owner(kafence: &Kafence, partition: i32) -> String {
    let route_key = format!("{}:{}", kafence.topic_router, partition);
    kafence
        .route_table
        .read()
        .unwrap()
        .get(&route_key)
        .cloned()
        .expect("route must exist")
}

fn key_for_partition(prefix: &str, target_partition: i32, partition_count: i32) -> String {
    for attempt in 0..10_000 {
        let key = format!("{prefix}_{attempt}");
        if crate::partition_for_key(key.as_bytes(), partition_count) == target_partition {
            return key;
        }
    }

    panic!("could not find key for partition {target_partition}");
}

async fn create_topic_if_not_exists(
    brokers: &str,
    topic: &str,
    partitions: u32,
) -> anyhow::Result<()> {
    if partitions == 0 {
        anyhow::bail!("topic partitions must be greater than zero");
    }

    let partitions = i32::try_from(partitions)?;
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("session.timeout.ms", "6000")
        .create()?;

    //TODO:Make replication factor configurable
    let new_topic = NewTopic::new(topic, partitions, TopicReplication::Fixed(1));
    let admin_options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(10)));

    for result in admin.create_topics(&[new_topic], &admin_options).await? {
        match result {
            Ok(name) => {
                println!("Created topic name {}", name)
            }
            Err((_, RDKafkaErrorCode::TopicAlreadyExists)) => {}
            Err((name, code)) => anyhow::bail!("error creating topic {} with error {}", name, code),
        }
    }
    Ok(())
}
