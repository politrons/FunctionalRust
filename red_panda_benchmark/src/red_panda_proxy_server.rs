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
use rdkafka::error::{KafkaError, KafkaResult};
use rdkafka::message::{BorrowedMessage, Headers, Message, OwnedMessage};
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
    let topic = "panda";
    let brokers = "34.83.74.100:9092,34.168.129.145:9092,34.168.132.253:9092";
    println!("Creating consumer....");
    let consumer = create_and_subscribe(&brokers, &topic);
    println!("Creating producer....");
    let producer = &create_producer(&brokers);
    let body = &fs::read_to_string("/home/pablo_garcia/development/FunctionalRust/red_panda_benchmark/resources/uuid.txt").unwrap();
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/panda/produce") => {
            let id = Uuid::new_v4();
            let key = id.to_string();
            let delivery_result = produce(producer, &key, body, &topic).await;
            if delivery_result.is_err() {
                match delivery_result.err() {
                    None => {}
                    Some(e) => {
                        println!("Error response {}", e.0.to_string());
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    }
                }
            }else{
                *response.status_mut() = StatusCode::OK;
            }
        }
        (&Method::GET, "/panda/consume") => {
           match  consume_all_records(consumer).await  {
               Ok(_) =>  *response.status_mut() = StatusCode::OK,
               Err(_) =>  *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR,
           }
            *response.status_mut() = StatusCode::OK;
        }
        (&Method::GET, "/panda/produce_consume") => {
            match produce_and_consume(producer, consumer, &body, &topic).await {
                Ok(_) => *response.status_mut() = StatusCode::OK,
                Err(_) => *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR,
            }

        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}

/// Red Panda produce/consumer
/// ---------------------------
async fn produce_and_consume(producer: &FutureProducer, consumer: StreamConsumer<CustomContext>, body: &str, topic: &str) -> Result<String, String> {
    let id = Uuid::new_v4();
    let key = id.to_string();
    let response = produce(producer, &key, body, topic).await;
    if response.is_err() {
        println!("Error producing record {}", response.err().unwrap().0.to_string());
        return Err("Error producing record".to_string());
    }else{
        println!("New record produce {}", response.ok().unwrap().0.to_string());
    }
    let (promise, future): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let now = std::time::SystemTime::now();
    tokio::task::spawn(consume_records(consumer, promise, move |_, bm| {
        let record_key = u8_slice_to_string(bm.key().unwrap());
        println!("Record key received {}", record_key);
        return key == record_key;
    }));
    return match future.recv() {
        Ok(v) => {
            let duration = now.elapsed().unwrap().as_millis();
            println!("Consume the record took ${:?} millis", duration);
            Ok("".to_string())
        }
        Err(e) => {
            println!("Error Consuming record");
            Err("".to_string())
        },
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
async fn produce(producer: &FutureProducer, key: &String, body: &str, topic: &str) -> OwnedDeliveryResult {
    let record = FutureRecord::to(topic)
        .payload(body)
        .key(key);
    let delivery_result: OwnedDeliveryResult = producer.send(record, Duration::from_secs(0)).await;
    return delivery_result;
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
async fn consume_all_records(consumer: StreamConsumer<CustomContext>) -> Result<&'static str, &'static str> {
    let (promise, future): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let now = std::time::SystemTime::now();
    tokio::task::spawn(consume_records(consumer, promise, |counter, bm| counter.eq(&100)));
    return match future.recv() {
        Ok(_) => {
            let duration = now.elapsed().unwrap().as_millis();
            println!("Consuming all records took ${:?} millis", duration);
            Ok("All record consumed")
        }
        Err(e) => {
            println!("Error Consuming records: {}", e.to_string());
            Err("Error Consuming records")
        },
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
    let consumer: StreamConsumer<CustomContext> = create_stream_consumer(brokers, CustomContext);
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
fn create_stream_consumer(brokers: &str, context: CustomContext) -> StreamConsumer<CustomContext> {
    ClientConfig::new()
        .set("group.id", "red_panda_group-id")
        .set("bootstrap.servers", brokers)
        .set_log_level(RDKafkaLogLevel::Info)
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
            Ok(bm) => {
                println!("key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}", bm.key(), bm.topic(), bm.partition(), bm.offset(), bm.timestamp());
                let _ = match bm.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
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
