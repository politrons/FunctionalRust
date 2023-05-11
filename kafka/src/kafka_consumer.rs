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

/**
We can create a Kafka consumer just using [Consumer::from_host(hosts)] builder.
Once we have the Consumer builder we can configure usual attributes, like topic
fallback offset, which it will set from where we need to start consuming([earliest/latest])
max record size we can fetch in on [poll]. Consumer group using [with_group].
Finally using [create] it return a Result of Consumer
 */
fn create_consumer(hosts: Vec<String>, topic: &str) -> Consumer {
    return match Consumer::from_hosts(hosts)
        .with_topic(topic.to_owned())
        .with_fallback_offset(FetchOffset::Latest)
        .with_fetch_max_bytes_per_partition(1000000)
        .with_group(String::from("my_consumer_group"))
        .create() {
        Ok(consumer) => consumer,
        Err(e) => {
            println!("Error creating Kafka consumer. Caused by {}", e);
            panic!()
        }
    };
}

/**
Once that we have the Consumer instance, we can start consuming records.
In rust you can get in infinite loop, why use [loop]
Now inside this loop, in each iteration, we [poll] using the consumer. This it will block,
until we get some records from Kafka as Result<MessageSet>.
[MessageSet] is a collection of possible records for each [topic] and [partition].
It also contains the collection of [Messages], that we extract in each iteration using [messages].

Once we process the record, we can mark the record as consumed by the caller using [consume_messageset]
and then once we process all records, do a commit to Kafka in batch for all of them using [commit_consumed]
*/
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
                println!("Record received offset:{:?}", m.offset);
            }
            consumer.consume_messageset(ms).unwrap();
        }
        consumer.commit_consumed().unwrap();
        if consuming {
            println!("Time to consume records {}", start_time.unwrap().elapsed().unwrap().as_millis());
            consuming = false;
        }
    }
}