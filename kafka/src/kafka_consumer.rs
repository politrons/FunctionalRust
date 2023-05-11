use kafka::consumer::{Consumer, FetchOffset};
use std::str;

pub fn run() {
    let topic = "topic-name";
    let hosts = vec!["localhost:9092".to_owned()];
    let consumer = create_consumer(hosts, topic);
    println!("Kafka Consumer {} subscribe to topic {} ", consumer.client().client_id(), topic);
    run_consumer(consumer)
}

fn create_consumer(hosts: Vec<String>, topic: &str) -> Consumer {
    return Consumer::from_hosts(hosts)
        .with_topic(topic.to_owned())
        .with_fallback_offset(FetchOffset::Latest)
        .create()
        .unwrap();
}

fn run_consumer(mut consumer: Consumer) {
    loop {
        for ms in consumer.poll().unwrap().iter() {
            for m in ms.messages() {
                // If the consumer receives an event, this block is executed
                println!("{:?}", str::from_utf8(m.value).unwrap());
            }

            consumer.consume_messageset(ms).unwrap();
        }

        consumer.commit_consumed().unwrap();
    }
}