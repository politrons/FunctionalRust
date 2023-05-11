use kafka::consumer::{Consumer, FetchOffset};
use std::str;
use std::time::SystemTime;

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
        .with_fetch_max_bytes_per_partition(1000000)
        .create()
        .unwrap();
}

fn run_consumer(mut consumer: Consumer) {
    let mut consuming = false;
    let mut start_time = None;
    loop {
        if start_time.is_none() {
            start_time = Some(SystemTime::now());
        }
        for ms in consumer.poll().unwrap().iter() {
            println!("Records received from topic:{} and partition:{}", ms.topic(), ms.partition());
            consuming = true;
            for m in ms.messages() {
                // If the consumer receives an event, this block is executed
                println!("Record key{:?}", str::from_utf8(m.key).unwrap());
            }
            consumer.consume_messageset(ms).unwrap();
        }
        consumer.commit_consumed().unwrap();
        if consuming {
            println!("Time to consume records {}", start_time.unwrap().elapsed().unwrap().as_millis());
            consuming=false;
        }
    }
}