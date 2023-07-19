//Server
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Method, StatusCode};

//Client
use std::error::Error;
use std::num::NonZeroU16;
use std::thread;
use std::time::Duration;
use hyper::Client;
use hyper::body::HttpBody as _;
use hyper::client::HttpConnector;
use tokio::io;
use tokio::io::{stdout, AsyncWriteExt as _};

//Producer
use std::fs;
use std::future::Future;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use uuid::Uuid;

//Consumer
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::error::KafkaResult;
use rdkafka::message::{BorrowedMessage, Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;

#[tokio::main]
async fn main() {
    run_server().await;
}

pub async fn run_server() {
    let port = 1981;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Running Server on port {}...", port);
    let server = Server::bind(&addr)
        .serve(make_service_fn(|_conn| async {
            println!("New request received.");
            Ok::<_, Infallible>(service_fn(create_service))
        }));
    if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}

async fn create_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let (path, topic, brokers) = load_program();
    let consumer = create_and_subscribe(&brokers, &topic);
    let producer = &create_producer(&brokers);
    let body = &fs::read_to_string("/home/pablo_garcia/development/FunctionalRust/red_panda_benchmark/resources/uuid.txt").unwrap();
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/panda/produce") => {
            let id = Uuid::new_v4();
            let key = id.to_string();
            produce(producer, &key, body, &topic).await;
            *response.status_mut() = StatusCode::OK;
        }
        (&Method::GET, "/panda/consume") => {
            consume_all_records(consumer).await;
            *response.status_mut() = StatusCode::OK;
        }
        (&Method::GET, "/panda/produce_consume") => {
            produce_and_consume(producer, consumer, &body, &topic).await;
            *response.status_mut() = StatusCode::OK;
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}

fn load_program() -> (String, String, String) {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => (args.get(0).unwrap().to_string(), "panda".to_string(), "34.83.74.100:9092".to_string()),
        2 => (args.get(0).unwrap().to_string(), args.get(1).unwrap().to_string(), "34.83.74.100:9092".to_string()),
        3 => (args.get(0).unwrap().to_string(), args.get(1).unwrap().to_string(), args.get(2).unwrap().to_string()),
        _ => ("/home/pablo_garcia/development/FunctionalRust/red_panda_benchmark/resources/uuid.txt".to_string(), "panda".to_string(), "34.83.74.100:9092".to_string())
    }
}


/// Red Panda produce/consumer
/// ---------------------------
async fn produce_and_consume(producer: &FutureProducer, consumer: StreamConsumer<CustomContext>, body: &str, topic: &str) {
    let id = Uuid::new_v4();
    let key = id.to_string();
    produce(producer, &key, body, topic).await;
    let (promise, future): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let now = std::time::SystemTime::now();
    tokio::task::spawn(consume_records(consumer, promise, move |_, bm| {
        let record_key = u8_slice_to_string(bm.key().unwrap());
        return key == record_key;
    }));
    match future.recv() {
        Ok(_) => {
            let duration = now.elapsed().unwrap().as_millis();
            println!("Consume the record took ${:?} millis", duration)
        }
        Err(_) => println!("Error Consuming record"),
    }
}

fn u8_slice_to_string(key: &[u8]) -> String {
    match std::str::from_utf8(key) {
        Ok(str_key) => str_key.to_string(),
        Err(_) => String::from("Invalid Key UTF-8 data"),
    }
}


/// Red Panda Producer
/// --------------------

/// Having a [FutureProducer] we use [send] operator to pass a [FutureRecord] together with Duration [Timeout]
/// If we set timeout with 0 value it will wait forever.
/// Since the scope of the [send] is async by design we need to create the FutureRecord inside the invocation.
async fn produce(producer: &FutureProducer, key: &String, body: &str, topic: &str) {
    let record = FutureRecord::to(topic)
        .payload(body)
        .key(key);
    let delivery_result = producer.send(record, Duration::from_secs(0)).await;
    println!("Delivery status for message received is ok {}", delivery_result.is_ok());
}

/// We create a producer using [ClientConfig] builder, where we set several keys as properties
///  like server to connect, and timeout of the sent messages.
/// We use [create] to create a [KafkaResult], that control side-effect trying to connect to Kafka broker.
/// We use [expect] to try to get the [OK] value from the [KafkaResult], and otherwise print a specific error message in case of Panic.
fn create_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("request.required.acks", "all")
        .create()
        .expect("Red Panda Producer creation error")
}

/// Red Panda consumer
/// -------------------

///Expected records

async fn consume_all_records(consumer: StreamConsumer<CustomContext>) {
    let (promise, future): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let now = std::time::SystemTime::now();
    tokio::task::spawn(consume_records(consumer, promise, |counter, bm| counter.eq(&1000)));
    match future.recv() {
        Ok(_) => {
            let duration = now.elapsed().unwrap().as_millis();
            println!("Consuming all records took ${:?} millis", duration)
        }
        Err(_) => println!("Error Consuming records"),
    }
}

/// Struct type to override [ClientContext], and override the callback functions.
struct CustomContext;

impl ClientContext for CustomContext {}

///Callbacks to receive information when a consumer is in rebalance process, and every time it commit an offset.
impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}

fn create_and_subscribe(brokers: &str, topic: &str) -> StreamConsumer<CustomContext> {
    let topics = [topic];
    let group_id = "red_panda_group-id";
    let consumer: StreamConsumer<CustomContext> = create_stream_consumer(brokers, group_id, CustomContext);
    consumer.subscribe(&topics.to_vec())
        .expect("Can't subscribe to specified topics");
    consumer
}

/// Creation of Consumer using [ClientConfig::new()] builder.
/// We set the broker to connect, groupId, session timeout, and if we want to do auto commit.
/// We use [create_with_context] to being able to create an pass our own implementation of the [ConsumerContext],
/// that we already override some methods.
/// We use [expect] to unwrap the [KafkaResult] obtained from the create operator, and we try to get the Ok,
/// otherwise we print the error messages passed, and we have a panic.
fn create_stream_consumer(brokers: &str, group_id: &str, context: CustomContext) -> StreamConsumer<CustomContext> {
    ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create_with_context(context)
        .expect("Red panda Consumer creation failed")
}

///Using the consumer we invoke [recv] to block the consumer until it get a record from the broker.
/// Since this operation is async we await, and once is resolved, we obtain a [Result]
/// We pattern matching the Result to extract a [KafkaError] and continue, or get a [BorrowedMessage]
/// With the we use [payload_view] to transform byte array into the type specify in the method.
/// This will return an [Option] that we match to control side-effect of nullable.
/// For this example we dont use the payload returned, that's why we define a [let _] in the match
async fn consume_records<F: Fn(&i32, &BorrowedMessage) -> bool>(consumer: StreamConsumer<CustomContext>, promise: Sender<bool>, escape_func: F) {
    let mut counter = 0;
    loop {
        match consumer.recv().await {
            Err(e) => println!("Error consuming Event. Caused by: {}", e),
            Ok(bm) => unsafe {
                let _ = match bm.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                println!("key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}", bm.key(), bm.topic(), bm.partition(), bm.offset(), bm.timestamp());
                commit_message(&consumer, &bm);
                if escape_func(&counter, &bm) {
                    println!("All records processed");
                    promise.send(true).expect("Unrecoverable Error responding promise");
                } else {
                    println!("Current counter ${:?}", counter);
                    counter += 1;
                }
            }
        };
    }
}

///Commit the message and control the side-effect of [KafkaResult]
fn commit_message(consumer: &StreamConsumer<CustomContext>, bm: &BorrowedMessage) {
    match consumer.commit_message(&bm, CommitMode::Async) {
        Ok(_) => println!("Message commit successfully"),
        Err(e) => println!("Error committing message. Caused by ${}", e.to_string()),
    }
}
