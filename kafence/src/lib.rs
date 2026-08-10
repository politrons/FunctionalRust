use anyhow::Result;
use kafka::producer::{Producer, Record};
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication, TopicResult};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::message::{BorrowedMessage, Message};
use rocksdb::{DB, Options};
use std::error::Error;
use std::time::Duration;

const ROCKSDB_PATH: &str = "./state/orders-store";

struct KafenceProducer {
    producer: Producer,
}

#[derive(Debug)]
struct KafenceProducerError {
    message: String,
}

#[derive(Debug)]
struct KafenceStreamError {
    message: String,
}

struct KafenceStream {}
struct Kafence {
    brokers: String,
    topic: String,
    consumer_group: String,
    partitions: u32,
    producer: Option<KafenceProducer>,
    stream: Option<KafenceStream>,
}

trait KafenceContract {
    fn with_brokers(self, brokers: String) -> Kafence;
    fn with_topic(self, topic: &str) -> Kafence;
    fn with_consumer_group(self, consumer_group: &str) -> Kafence;
    fn with_partitions(self, partitions: u32) -> Kafence;
    fn producer(self) -> Result<KafenceProducer, KafenceProducerError>;
    async fn stream(self) -> Result<()>;
}

impl Kafence {
    fn new() -> Kafence {
        Kafence {
            brokers: "".to_string(),
            consumer_group: "".to_string(),
            partitions: 1,
            topic: "".to_string(),
            producer: None,
            stream: None,
        }
    }
}

impl KafenceContract for Kafence {
    fn with_brokers(self, brokers: String) -> Kafence {
        Kafence {
            brokers: brokers,
            topic: self.topic,
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            producer: self.producer,
            stream: self.stream,
        }
    }

    fn with_topic(self, topic: &str) -> Kafence {
        Kafence {
            brokers: self.brokers,
            topic: topic.to_string(),
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            producer: self.producer,
            stream: self.stream,
        }
    }
    fn with_consumer_group(self, consumer_group: &str) -> Kafence {
        Kafence {
            brokers: self.brokers,
            topic: self.topic,
            consumer_group: consumer_group.to_string(),
            partitions: self.partitions,
            producer: self.producer,
            stream: self.stream,
        }
    }

    fn with_partitions(self, partitions: u32) -> Kafence {
        Kafence {
            brokers: self.brokers,
            topic: self.topic,
            consumer_group: self.consumer_group,
            partitions: partitions,
            producer: self.producer,
            stream: self.stream,
        }
    }
    async fn stream(self) -> Result<()> {
        create_topic_if_not_exists(&self.brokers, &self.topic).await?;
        let rocks_db = open_rocksdb(ROCKSDB_PATH)?;
        let consumer = create_consumer(&self.brokers, &self.consumer_group)?;
        materialize_loop(&consumer, &rocks_db).await
    }

    fn producer(self) -> Result<KafenceProducer, KafenceProducerError> {
        match Producer::from_hosts(Vec::from([self.brokers]))
            .with_connection_idle_timeout(Duration::from_secs(10))
            .with_ack_timeout(Duration::from_secs(10))
            .create()
        {
            Ok(producer) => Ok(KafenceProducer { producer: producer }),
            Err(e) => {
                println!("Error creating Kafka producer. Caused by {}", e);
                Err(KafenceProducerError {
                    message: e.to_string(),
                })
            }
        }
    }
}
async fn create_topic_if_not_exists(brokers: &str, topic: &str) -> Result<()> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("enable.auto.commit", "false")
        .set("session.timeout.ms", "6000")
        .create()?;

    //TODO:Make replication factor configurable
    let new_topic = NewTopic::new(topic, 1, TopicReplication::Fixed(1));

    for result in admin
        .create_topics(&[new_topic], &AdminOptions::new())
        .await?
    {
        match result {
            Ok(name) => {
                println!("Created topic name {}", name)
            }
            Err((name, code)) => {
                println!("Error creating topic {} with error {}", name, code)
            }
        }
    }
    Ok(())
}

fn create_consumer(brokers: &str, group_id: &str) -> Result<StreamConsumer, KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .create()
        .map_err(Into::into)
}

fn open_rocksdb(path: &str) -> Result<DB, rocksdb::Error> {
    let mut options = Options::default();
    options.create_if_missing(true);
    Ok(DB::open(&options, path)?)
}

async fn materialize_loop(consumer: &StreamConsumer, rocks_db: &DB) -> Result<()> {
    loop {
        let message = consumer.recv().await?;
        materialize(&message, &rocks_db).await?;
    }
}

async fn materialize(message: &BorrowedMessage<'_>, rocks_db: &DB) -> Result<()> {
    //TODO:Handle side-effects
    rocks_db.put(message.key().unwrap(), message.payload().unwrap())?;
    Ok(())
}

mod test {
    use crate::{Kafence, KafenceContract};

    #[test]
    fn producer_test() {
        Kafence::new().with_topic("kafka").with_partitions(1);
    }
}
