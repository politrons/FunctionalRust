use anyhow::Result;
use kafka::Error;
use kafka::producer::Record;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::{ClientContext, DefaultClientContext};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, DefaultConsumerContext, Rebalance,
    StreamConsumer,
};
use rdkafka::error::KafkaError;
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::message::{BorrowedMessage, Message, ToBytes};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rocksdb::{DB, Options};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::task;

const ROCKSDB_PATH: &str = "./state/orders-store";

struct KafenceProducer {
    producer: FutureProducer,
}

#[derive(Debug)]
struct KafenceProducerError {
    message: String,
}

#[derive(Debug)]
struct KafenceStreamError {
    message: String,
}

#[derive(Clone)]
struct KafenceStream {}

#[derive(Clone)]
struct Kafence {
    client_id: String,
    brokers: String,
    topic: String,
    consumer_group: String,
    topic_router: String,
    routed_consumer_group: String,
    route_table: Arc<RwLock<HashMap<String, String>>>,
    partitions: u32,
    rocksdb_path: String,
}


struct KafenceConsumerContext {
    topic: String,
    partitions: Arc<RwLock<HashSet<i32>>>,
}

impl ClientContext for KafenceConsumerContext {}

impl ConsumerContext for KafenceConsumerContext {
    fn post_rebalance(&self, consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        println!("Pre rebalance {:?}", rebalance);
        match rebalance {
            Rebalance::Assign(_) | Rebalance::Revoke(_) => {
                let partitions = consumer
                    .assignment()
                    .map(|tp_list| {
                        tp_list
                            .elements()
                            .iter()
                            .filter(|tp| tp.topic() == self.topic)
                            .map(|tp| {
                                println!(
                                    "Topic: {} new partition assigned {}",
                                    tp.topic(),
                                    tp.partition()
                                );
                                tp.partition()
                            })
                            .collect::<HashSet<_>>()
                    })
                    .unwrap_or_default();
                *self.partitions.write().unwrap() = partitions;
            }
            Rebalance::Error(e) => {
                println!("rebalance error: {e}");
            }
        }
    }
}

impl Kafence {
    fn new() -> Kafence {
        Kafence {
            client_id: uuid::Uuid::new_v4().to_string(),
            brokers: "".to_string(),
            consumer_group: "".to_string(),
            routed_consumer_group: "".to_string(),
            partitions: 1,
            topic: "".to_string(),
            topic_router: "".to_string(),
            route_table: Arc::new(RwLock::new(HashMap::new())),
            rocksdb_path: ROCKSDB_PATH.to_string(),
        }
    }

    fn with_brokers(self, brokers: String) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: self.consumer_group,
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_topic(self, topic: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: topic.to_string(),
            topic_router: topic.to_string() + "_router",
            routed_consumer_group: topic.to_string() + "routed_consumer_group",
            route_table: self.route_table,
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }
    fn with_consumer_group(self, consumer_group: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: consumer_group.to_string(),
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_partitions(self, partitions: u32) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: self.consumer_group,
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_rocksdb_path(self, rocksdb_path: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: self.consumer_group,
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            partitions: self.partitions,
            rocksdb_path: rocksdb_path.to_string(),
        }
    }

    async fn stream(&self) -> Result<()> {
        let kafence_stream = self.clone();
        let stream_task = task::spawn(async move { kafence_stream.create_stream().await });
        tokio::time::sleep(Duration::from_secs(2)).await;
        let kafence_owner_partition = self.clone();
        let stream_partition_owner_task =
            task::spawn(async move { kafence_owner_partition.create_stream().await });
        assert!(
            !stream_task.is_finished(),
            "stream task stopped before producing"
        );
        assert!(
            !stream_partition_owner_task.is_finished(),
            "stream partition owner task stopped before producing"
        );
        Ok(())
    }

    fn producer(&self) -> Result<KafenceProducer, KafenceProducerError> {
        match ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("message.timeout.ms", "5000")
            .create()
        {
            Ok(producer) => Ok(KafenceProducer { producer }),
            Err(e) => {
                println!("Error creating Kafka producer. Caused by {}", e);
                Err(KafenceProducerError {
                    message: e.to_string(),
                })
            }
        }
    }
    fn build(self) -> Arc<Kafence> {
        Arc::new(self)
    }

    async fn create_stream(&self) -> Result<()> {
        let rocks_db = open_rocksdb(&self.rocksdb_path)?;
        let context = KafenceConsumerContext {
            topic: self.topic.to_string(),
            partitions: Arc::new(RwLock::new(HashSet::new())),
        };
        let stream_consumer = create_consumer(
            &self.client_id,
            &self.topic,
            &self.brokers,
            &self.consumer_group,
            context,
        )?;
        stream_consumer.subscribe(&[self.topic.as_str()])?;
        materialize_loop(&self.client_id, &stream_consumer, &rocks_db).await
    }

    async fn create_routed_stream(&self) -> Result<()> {
        create_route_topic_if_not_exists(&self.brokers, &self.topic_router, self.partitions)
            .await?;
        let stream_consumer = create_consumer(
            &self.client_id,
            &self.topic_router,
            &self.brokers,
            &self.routed_consumer_group,
            DefaultConsumerContext,
        )?;
        stream_consumer.subscribe(&[self.topic.as_str()])?;
        loop {
            let message = stream_consumer.recv().await?;
            materialize_route_table(&self.client_id, &message, &self.route_table)?;
            stream_consumer.commit_message(&message, CommitMode::Async)?;
        }
    }
}

async fn create_route_topic_if_not_exists(
    brokers: &str,
    topic: &str,
    partitions: u32,
) -> anyhow::Result<()> {
    if partitions == 0 {
        anyhow::bail!("routed topic partitions must be greater than zero");
    }

    let partitions = i32::try_from(partitions)?;
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("session.timeout.ms", "6000")
        .set("cleanup.policy", "compact")
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
            Err((name, code)) => {
                anyhow::bail!("error creating topic {} with error {}", name, code)
            }
        }
    }
    Ok(())
}

// Kafka Producer
// --------------

trait KafenceProducerContract<K, V> {
    async fn strong_consistency(&self, topic: &str, key: K, value: V);
}

impl<K: ToBytes + Send + Sync + Clone + 'static, V: ToBytes + Send> KafenceProducerContract<K, V>
    for KafenceProducer
{
    async fn strong_consistency(&self, topic: &str, key: K, value: V) {
        let record = FutureRecord::to(topic).key(&key).payload(&value);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .expect("record must be delivered");
    }
}

type KafenceStreamConsumer = StreamConsumer<KafenceConsumerContext>;

fn create_consumer<C>(
    client_id: &str,
    topic: &str,
    brokers: &str,
    group_id: &str,
    context: C,
) -> Result<StreamConsumer<C>, KafkaError>
where
    C: ConsumerContext + 'static,
{
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("client.id", client_id)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .create_with_context(context)
        .map_err(Into::into)
}

fn open_rocksdb(path: &str) -> Result<DB, rocksdb::Error> {
    let mut options = Options::default();
    options.create_if_missing(true);
    Ok(DB::open(&options, path)?)
}

async fn materialize_loop(
    client_id: &str,
    stream_consumer: &KafenceStreamConsumer,
    rocks_db: &DB,
) -> Result<()> {
    loop {
        let message = stream_consumer.recv().await?;
        materialize_rocksdb(client_id, &message, rocks_db)?;
        stream_consumer.commit_message(&message, CommitMode::Async)?;
    }
}

fn materialize_rocksdb(
    client_id: &str,
    message: &BorrowedMessage<'_>,
    rocks_db: &DB,
) -> Result<()> {
    let key = materialized_key(message);

    match message.payload() {
        Some(value) => {
            println!(
                "Materialized message client_id={} topic={} partition={} offset={} key={:?} value={:?}",
                client_id,
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key),
                String::from_utf8_lossy(value)
            );
            rocks_db.put(&key, value)?
        }
        None => {
            println!(
                "Deleted tombstone topic={} partition={} offset={} key={:?}",
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key)
            );
            rocks_db.delete(&key)?
        }
    }
    Ok(())
}

fn materialize_route_table(
    client_id: &str,
    message: &BorrowedMessage<'_>,
    route_table: &Arc<RwLock<HashMap<String, String>>>,
) -> Result<()> {
    let key = materialized_key(message);

    match message.payload() {
        Some(value) => {
            println!(
                "Materialized partition owner client_id={} topic={} partition={} offset={} key={:?} value={:?}",
                client_id,
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key),
                String::from_utf8_lossy(value)
            );
            let mut route_table = route_table.write().unwrap();
            let key = String::from_utf8_lossy(message.key().unwrap()).to_string();
            let value = String::from_utf8_lossy(message.payload().unwrap()).to_string();
            route_table.insert(key, value).unwrap();
            // rocks_db.put(&key, value)?
        }
        None => {
            println!(
                "Deleted tombstone topic={} partition={} offset={} key={:?}",
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key)
            );
            // rocks_db.delete(&key)?
        }
    }
    Ok(())
}

fn materialized_key(message: &BorrowedMessage<'_>) -> Vec<u8> {
    match message.key() {
        Some(key) => key.to_vec(),
        None => format!(
            "{}:{}:{}",
            message.topic(),
            message.partition(),
            message.offset()
        )
        .into_bytes(),
    }
}

#[cfg(test)]
mod test {
    use crate::{Kafence, KafenceProducerContract};
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
            .build();

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
            .build();

        // Services
        let service_1_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let service_1_addr = service_1_listener.local_addr().unwrap();
        let service_1 = tokio::spawn(run_server(service_1_listener, Arc::clone(&kaference_1)));

        let service_2_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let service_2_addr = service_2_listener.local_addr().unwrap();
        let service_2 = tokio::spawn(run_server(service_2_listener, Arc::clone(&kaference_2)));

        tokio::time::sleep(Duration::from_secs(5)).await;

        let client = Client::new();

        for i in 1..=10 {
            let addr = if i % 2 == 0 {
                service_1_addr
            } else {
                service_2_addr
            };
            let request = Request::builder()
                .method(Method::POST)
                .uri(format!("http://{addr}/"))
                .header("record-key", format!("record_key_{i}"))
                .body(Body::from(format!("hello world {i}")))
                .unwrap();

            let response = client.request(request).await.unwrap();
            assert_eq!(StatusCode::ACCEPTED, response.status());
        }

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
                        state
                            .producer
                            .strong_consistency(&state.topic, key.clone(), value)
                            .await;
                        *response.status_mut() = StatusCode::ACCEPTED;
                        *response.body_mut() = Body::from(format!("published {key}"));
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
                Err((name, code)) => {
                    anyhow::bail!("error creating topic {} with error {}", name, code)
                }
            }
        }
        Ok(())
    }
}
