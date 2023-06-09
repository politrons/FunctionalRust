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

async fn consume(promise: Sender<bool>) {
    let topics = ["my_red_panda_topic"];
    let brokers = "localhost:9092";
    let group_id = "red_panda_group-id";
    let consumer: StreamConsumer<CustomContext> = create_consumer(brokers, group_id, CustomContext);
    consumer.subscribe(&topics.to_vec())
        .expect("Can't subscribe to specified topics");
    consume_records(consumer, promise).await;
}

/// Creation of Consumer using [ClientConfig::new()] builder.
/// We set the broker to connect, groupId, session timeout, and if we want to do auto commit.
/// We use [create_with_context] to being able to create an pass our own implementation of the [ConsumerContext],
/// that we already override some methods.
/// We use [expect] to unwrap the [KafkaResult] obtained from the create operator, and we try to get the Ok,
/// otherwise we print the error messages passed, and we have a panic.
fn create_consumer(brokers: &str, group_id: &str, context: CustomContext) -> StreamConsumer<CustomContext> {
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
async fn consume_records(consumer: StreamConsumer<CustomContext>, promise: Sender<bool>) {
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
                println!("key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                         bm.key(), bm.topic(), bm.partition(), bm.offset(), bm.timestamp());
                commit_message(&consumer, &bm);
                if counter == EXPECT_RECORDS {
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

///Expected records
static mut EXPECT_RECORDS: i32 = 1000;

#[tokio::main]
async fn main() {
    let (promise, future): (Sender<bool>, Receiver<bool>) = mpsc::channel();
    let now = std::time::SystemTime::now();
    tokio::task::spawn(consume(promise));
    match future.recv() {
        Ok(_) => {
            let duration = now.elapsed().unwrap().as_millis();
            println!("Consuming all records took ${:?} millis", duration)
        }
        Err(_) => println!("Error Consuming records"),
    }
}