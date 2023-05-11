use kafka::producer::{Producer, Record};

pub fn run() {
    let hosts = vec!["localhost:9092".to_owned()];

    let mut producer =
        Producer::from_hosts(hosts)
            .create()
            .unwrap();

    for i in 0..10 {
        let buf = format!("{i}");
        producer.send(&Record::from_value("topic-name", buf.as_bytes())).unwrap();
        println!("Sent: {i}");
    }
}