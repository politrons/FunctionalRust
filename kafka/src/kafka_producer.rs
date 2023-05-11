use std::{fs, time};
use std::string::ToString;
use time::Duration;

use kafka::producer::{Producer, Record};
use uuid::Uuid;

pub fn run() {
    let hosts = vec!["localhost:9092".to_owned()];
    let mut producer = create_producer(hosts);
    send_records(&mut producer);
}

/**
We can create a [Kafka] producer using [Producer] struct. Then using [from_host] we create the builder
to append rest of properties like idle_timeout or ack_timeout.
Once finish we can use [create] to finish the builder, and create a [Result] of Producer
 */
fn create_producer(hosts: Vec<String>) -> Producer {
    match Producer::from_hosts(hosts)
        .with_connection_idle_timeout(Duration::from_secs(10))
        .with_ack_timeout(Duration::from_secs(10))
        .create() {
        Ok(producer) => producer,
        Err(e) => {
            println!("Error creating Kafka producer. Caused by {}", e);
            panic!()
        }
    }
}

/**
Function to create Records and send to Kafka topic using the [Producer] created previously.
[Record] builder can start the creation of the record by [from], [from_key_value] or [from_key_value]
where you need to specify an already created Record, a key-value pair or just the value.
Then a [Result] type of [Record] is returned to control any possible side-effect
 */
fn send_records(producer: &mut Producer) {
    let topic = "topic-name";
    let value = fs::read_to_string("resources/uuid.txt").unwrap();
    for i in 0..1000 {
        match producer.send(&Record::from_value(topic,value.as_bytes())) {
            Ok(_) => println!("Record Sent: {i}"),
            Err(e) => println!("Error sending record. Caused by: {}", e)
        }
    }
}


