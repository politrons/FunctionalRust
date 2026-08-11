use anyhow::Result;
use kafka::Error;
use kafka::producer::Record;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::message::{BorrowedMessage, Message, ToBytes};
use rdkafka::producer::{FutureProducer, FutureRecord};
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
    client_id: String,
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
            client_id: uuid::Uuid::new_v4().to_string(),
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
        let stream_consumer =
            create_consumer(&self.client_id, &self.brokers, &self.consumer_group)?;
        stream_consumer.subscribe(&[self.topic.as_str()])?;
        materialize_loop(&self.client_id, &stream_consumer, &rocks_db).await
    }
}

impl KafenceContract for Kafence {
    fn with_brokers(self, brokers: String) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: brokers,
            topic: self.topic,
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_topic(self, topic: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: topic.to_string(),
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }
    fn with_consumer_group(self, consumer_group: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            consumer_group: consumer_group.to_string(),
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_partitions(self, partitions: u32) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            consumer_group: self.consumer_group,
            partitions,
            rocksdb_path: self.rocksdb_path,
        }
    }

    fn with_rocksdb_path(self, rocksdb_path: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
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

trait KafenceProducerContract<K, V> {
    async fn strong_consistency(&self, topic: &str, key: K, value: V);
}

impl<K: ToBytes + Send + Sync + Clone + 'static, V: ToBytes + Send> KafenceProducerContract<K, V>
    for KafenceProducer
{
    async fn strong_consistency(&self, topic: &str, key: K, value: V) {
        let record = FutureRecord::to(topic).key(&key).payload(&value);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .expect("record must be delivered");
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

fn create_consumer(
    client_id: &str,
    brokers: &str,
    group_id: &str,
) -> Result<StreamConsumer, KafkaError> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("client.id", client_id)
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

async fn materialize_loop(
    client_id: &str,
    stream_consumer: &StreamConsumer,
    rocks_db: &DB,
) -> Result<()> {
    loop {
        let message = stream_consumer.recv().await?;
        materialize(client_id, &message, rocks_db)?;
        stream_consumer.commit_message(&message, CommitMode::Async)?;
    }
}

fn materialize(client_id: &str, message: &BorrowedMessage<'_>, rocks_db: &DB) -> Result<()> {
    let key = materialized_key(message);

    match message.payload() {
        Some(value) => {
            println!(
                "Materialized message client_id={} topic={} partition={} offset={} key={:?} value={:?}",
                client_id,
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
    use crate::{Kafence, KafenceContract, KafenceProducerContract, create_topic_if_not_exists};
    use rdkafka::producer::FutureRecord;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::task;
    use uuid::Uuid;

    #[tokio::test]
    async fn producer_test() {
        let broker = "localhost:9092";
        let run_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let topic = format!("kafka_topic_{run_id}");
        let consumer_group = format!("kafka_group_{run_id}");

        create_topic_if_not_exists(broker, &topic, 2).await.unwrap();

        // DSL
        // -----
        let uuid = uuid::Uuid::new_v4();
        let rocksdb_path_1 = std::env::temp_dir()
            .join(format!("kafence-{run_id}-{uuid}"))
            .to_string_lossy()
            .into_owned();

        let kaference_1 = Kafence::new()
            .with_brokers(broker.to_string())
            .with_topic(&topic)
            .with_consumer_group(&consumer_group)
            .with_partitions(2)
            .with_rocksdb_path(&rocksdb_path_1)
            .build();

        let uuid = uuid::Uuid::new_v4();
        let rocksdb_path_2 = std::env::temp_dir()
            .join(format!("kafence-{run_id}-{uuid}"))
            .to_string_lossy()
            .into_owned();

        let kaference_2 = Kafence::new()
            .with_brokers(broker.to_string())
            .with_topic(&topic)
            .with_consumer_group(&consumer_group)
            .with_partitions(2)
            .with_rocksdb_path(&rocksdb_path_2)
            .build();

        //Stream
        let kaference_stream_1 = Arc::clone(&kaference_1);
        kaference_stream_1.stream().await.unwrap();

        let kaference_stream_2 = Arc::clone(&kaference_2);
        kaference_stream_2.stream().await.unwrap();

        //Strong consistency
        let kaference_producer = Arc::clone(&kaference_1);
        let producer = kaference_producer.producer().unwrap();
        for i in 1..=10 {
            producer
                .strong_consistency(
                    &topic,
                    format!("record_key_{i}").to_string(),
                    format!("hello world {i}").to_string(),
                )
                .await;
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
