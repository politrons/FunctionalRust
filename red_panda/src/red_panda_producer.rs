use std::fs;
use std::future::Future;
use std::time::Duration;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::producer::future_producer::OwnedDeliveryResult;

async fn produce(brokers: &str, topic_name: &str) {
    let producer = &create_producer(brokers);
    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..1000)
        .map(|i| async move {
            let delivery_result = send_record(topic_name, &producer, &i).await;
            println!("Delivery status for message {} received", i);
            delivery_result
        })
        .collect::<Vec<_>>();

    process_results(futures).await;
}

/// We create a producer using [ClientConfig] builder, where we set several keys as properties
///  like server to connect, and timeout of the sent messages.
/// We use [create] to create a [KafkaResult], that control side-effect trying to connect to Kafka broker.
/// We use [expect] to try to get the [OK] value from the [KafkaResult], and otherwise print a specific error message in case of Panic.
fn create_producer(brokers: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Red Panda Producer creation error")
}


/// Having a [FutureProducer] we use [send] operator to pass a [FutureRecord] together with Duration [Timeout]
/// If we set timeout with 0 value it will wait forever.
/// Since the scope of the [send] is async by design we need to create the FutureRecord inside the invocation.
///
async fn send_record(topic_name: &str, producer: &FutureProducer, i: &i32) -> OwnedDeliveryResult {
    let body = "Hello world";
    let key = &format!("Key {}", i);
    let record = FutureRecord::to(topic_name)
        .payload(body)
        .key(key);
    producer.send(record, Duration::from_secs(0)).await
}

/// This loop will wait until all delivery statuses have been received.
async fn process_results(futures: Vec<impl Future<Output=OwnedDeliveryResult> + Sized>) {
    for future in futures {
        println!("Future completed. Result: {:?}", future.await);
    }
}


#[tokio::main]
async fn main() {
    let topic = "my_red_panda_topic";
    let brokers = "34.127.78.47:9092";
    produce(brokers, topic).await;
}