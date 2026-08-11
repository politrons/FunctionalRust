use anyhow::Result;
use kafka::Error;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::message::{BorrowedMessage, Message};
use rdkafka::producer::FutureProducer;
use rocksdb::{DB, Options};
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

const ROCKSDB_PATH: &str = "./state/orders-store";

struct KafenceProducer {
    producer: FutureProducer,
}

#[derive(Debug)]
struct KafenceProducerError {
    message: String,
}

#[derive(Debug)]
struct KafenceStreamError {
    message: String,
}

#[derive(Clone)]
struct KafenceStream {}

#[derive(Clone)]
struct Kafence {
    brokers: String,
    topic: String,
    consumer_group: String,
    partitions: u32,
    rocksdb_path: String,
}

trait KafenceContract {
    fn with_brokers(self, brokers: String) -> Kafence;
    fn with_topic(self, topic: &str) -> Kafence;
    fn with_consumer_group(self, consumer_group: &str) -> Kafence;
    fn with_partitions(self, partitions: u32) -> Kafence;
    fn with_rocksdb_path(self, rocksdb_path: &str) -> Kafence;
    fn build(self) -> Arc<Kafence>;
    fn producer(&self) -> Result<KafenceProducer, KafenceProducerError>;
    async fn stream(&self) -> Result<()>;
}

impl Kafence {
    fn new() -> Kafence {
        Kafence {
            brokers: "".to_string(),
            consumer_group: "".to_string(),
            partitions: 1,
            topic: "".to_string(),
            rocksdb_path: ROCKSDB_PATH.to_string(),
        }
    }
    async fn create_stream(&self) -> Result<()> {
        create_topic_if_not_exists(&self.brokers, &self.topic, self.partitions).await?;
        let rocks_db = open_rocksdb(&self.rocksdb_path)?;
        let stream_consumer = create_consumer(&self.brokers, &self.consumer_group)?;
        stream_consumer.subscribe(&[self.topic.as_str()])?;
        materialize_loop(&stream_consumer, &rocks_db).await
    }
}

impl KafenceContract for Kafence {
    fn with_brokers(self, brokers: String) -> Kafence {
        Kafence {
            brokers: brokers,
            topic: self.topic,
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_topic(self, topic: &str) -> Kafence {
        Kafence {
            brokers: self.brokers,
            topic: topic.to_string(),
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }
    fn with_consumer_group(self, consumer_group: &str) -> Kafence {
        Kafence {
            brokers: self.brokers,
            topic: self.topic,
            consumer_group: consumer_group.to_string(),
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_partitions(self, partitions: u32) -> Kafence {
        Kafence {
            brokers: self.brokers,
            topic: self.topic,
            consumer_group: self.consumer_group,
            partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_rocksdb_path(self, rocksdb_path: &str) -> Kafence {
        Kafence {
            brokers: self.brokers,
            topic: self.topic,
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            rocksdb_path: rocksdb_path.to_string(),
        }
    }

    async fn stream(&self) -> Result<()> {
        // let kafence = Arc::new(self);
        let kafence = self.clone();
        let stream_task = task::spawn(async move { kafence.create_stream().await });
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(
            !stream_task.is_finished(),
            "stream task stopped before producing"
        );
        Ok(())
    }

    fn producer(&self) -> Result<KafenceProducer, KafenceProducerError> {
        match ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("message.timeout.ms", "5000")
            .create()
        {
            Ok(producer) => Ok(KafenceProducer { producer }),
            Err(e) => {
                println!("Error creating Kafka producer. Caused by {}", e);
                Err(KafenceProducerError {
                    message: e.to_string(),
                })
            }
        }
    }
    fn build(self) -> Arc<Kafence> {
        Arc::new(self)
    }
}
async fn create_topic_if_not_exists(brokers: &str, topic: &str, partitions: u32) -> Result<()> {
    if partitions == 0 {
        anyhow::bail!("topic partitions must be greater than zero");
    }

    let partitions = i32::try_from(partitions)?;
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("session.timeout.ms", "6000")
        .create()?;

    //TODO:Make replication factor configurable
    let new_topic = NewTopic::new(topic, partitions, TopicReplication::Fixed(1));
    let admin_options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(10)));

    for result in admin.create_topics(&[new_topic], &admin_options).await? {
        match result {
            Ok(name) => {
                println!("Created topic name {}", name)
            }
            Err((_, RDKafkaErrorCode::TopicAlreadyExists)) => {}
            Err((name, code)) => {
                anyhow::bail!("error creating topic {} with error {}", name, code)
            }
        }
    }
    Ok(())
}

fn create_consumer(brokers: &str, group_id: &str) -> Result<StreamConsumer, KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("enable.auto.commit", "false")
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

async fn materialize_loop(stream_consumer: &StreamConsumer, rocks_db: &DB) -> Result<()> {
    loop {
        let message = stream_consumer.recv().await?;
        materialize(&message, rocks_db)?;
        stream_consumer.commit_message(&message, CommitMode::Async)?;
    }
}

fn materialize(message: &BorrowedMessage<'_>, rocks_db: &DB) -> Result<()> {
    let key = materialized_key(message);

    match message.payload() {
        Some(value) => {
            println!(
                "Materialized message topic={} partition={} offset={} key={:?} value={:?}",
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key),
                String::from_utf8_lossy(value)
            );
            rocks_db.put(&key, value)?
        }
        None => {
            println!(
                "Deleted tombstone topic={} partition={} offset={} key={:?}",
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key)
            );
            rocks_db.delete(&key)?
        }
    }

    Ok(())
}

fn materialized_key(message: &BorrowedMessage<'_>) -> Vec<u8> {
    match message.key() {
        Some(key) => key.to_vec(),
        None => format!(
            "{}:{}:{}",
            message.topic(),
            message.partition(),
            message.offset()
        )
        .into_bytes(),
    }
}

#[cfg(test)]
mod test {
    use rdkafka::producer::FutureRecord;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::task;

    use crate::{Kafence, KafenceContract, create_topic_if_not_exists};

    #[tokio::test]
    async fn producer_test() {
        let broker = "localhost:9092";
        let run_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let topic = format!("kafka_topic_{run_id}");
        let consumer_group = format!("kafka_group_{run_id}");
        let rocksdb_path = std::env::temp_dir()
            .join(format!("kafence-{run_id}"))
            .to_string_lossy()
            .into_owned();

        create_topic_if_not_exists(broker, &topic, 1).await.unwrap();

        let kaference = Kafence::new()
            .with_brokers(broker.to_string())
            .with_topic(&topic)
            .with_consumer_group(&consumer_group)
            .with_partitions(1)
            .with_rocksdb_path(&rocksdb_path)
            .build();

        let kaference_stream = Arc::clone(&kaference);

        kaference_stream.stream().await.unwrap();

        let producer = kaference.producer().unwrap().producer;
        let producer_topic = topic.clone();
        let producer_task = task::spawn(async move {
            let payload = format!("kafka stream works");
            let record = FutureRecord::to(&producer_topic)
                .key("my-key")
                .payload(&payload);

            producer
                .send(record, Duration::from_secs(5))
                .await
                .expect("record must be delivered");
        });

        producer_task.await.unwrap();
    }
}
