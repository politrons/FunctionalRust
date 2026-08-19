use anyhow::Result;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::{ClientContext, DefaultClientContext};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, DefaultConsumerContext, Rebalance,
    StreamConsumer,
};
use rdkafka::error::KafkaError;
use rdkafka::error::RDKafkaErrorCode;
use rdkafka::message::{BorrowedMessage, Message, ToBytes};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rocksdb::{DB, Options};
use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;
use tokio::task;

const ROCKSDB_PATH: &str = "./state/orders-store";

type MaterializerAck = Arc<RwLock<HashMap<String, Vec<oneshot::Sender<()>>>>>;

struct KafenceProducer {
    producer: FutureProducer,
    service_url: String,
    topic_router: String,
    partitions: i32,
    route_table: Arc<RwLock<HashMap<String, String>>>,
    materializer_ack: MaterializerAck,
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
    topic_router: String,
    routed_consumer_group: String,
    route_table: Arc<RwLock<HashMap<String, String>>>,
    materializer_ack: MaterializerAck,
    partitions: i32,
    rocksdb_path: String,
    serviice_url: String,
}

struct KafenceConsumerContext {
    topic: String,
    service_host: String,
    router_channel_sender: UnboundedSender<RouteInfo>,
}

#[derive(Clone, Debug)]
struct RouteInfo {
    paritions: Arc<RwLock<HashSet<i32>>>,
    service_host: String,
}

impl ClientContext for KafenceConsumerContext {}

impl ConsumerContext for KafenceConsumerContext {
    fn post_rebalance(&self, consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        println!("Pre rebalance {:?}", rebalance);
        match rebalance {
            Rebalance::Assign(_) | Rebalance::Revoke(_) => {
                let partitions = consumer
                    .assignment()
                    .map(|tp_list| {
                        tp_list
                            .elements()
                            .iter()
                            .filter(|tp| tp.topic() == self.topic)
                            .map(|tp| {
                                println!(
                                    "Topic: {} new partition assigned {}",
                                    tp.topic(),
                                    tp.partition()
                                );
                                tp.partition()
                            })
                            .collect::<HashSet<_>>()
                    })
                    .unwrap_or_default();
                let route_info = RouteInfo {
                    paritions: Arc::new(RwLock::new(partitions)),
                    service_host: self.service_host.clone(),
                };
                if !self.router_channel_sender.is_closed() {
                    match self.router_channel_sender.send(route_info) {
                        Ok(_) => println!("Router channel sent"),
                        Err(e) => println!("Router channel send failed {}", e),
                    }
                }
            }
            Rebalance::Error(e) => {
                println!("rebalance error: {e}");
            }
        }
    }
}

impl Kafence {
    fn new() -> Kafence {
        Kafence {
            client_id: uuid::Uuid::new_v4().to_string(),
            brokers: "".to_string(),
            consumer_group: "".to_string(),
            routed_consumer_group: "".to_string(),
            partitions: 1,
            topic: "".to_string(),
            topic_router: "".to_string(),
            route_table: Arc::new(RwLock::new(HashMap::new())),
            materializer_ack: Arc::new(RwLock::new(HashMap::new())),
            rocksdb_path: ROCKSDB_PATH.to_string(),
            serviice_url: "".to_string(),
        }
    }

    fn with_brokers(self, brokers: String) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: self.consumer_group,
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            materializer_ack: self.materializer_ack,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
            serviice_url: self.serviice_url,
        }
    }

    fn with_topic(self, topic: &str) -> Kafence {
        let topic_router = format!("{topic}_router");
        let routed_consumer_group = format!("{topic}_routed_consumer_group_{}", self.client_id);

        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: topic.to_string(),
            topic_router,
            routed_consumer_group,
            route_table: self.route_table,
            materializer_ack: self.materializer_ack,
            consumer_group: self.consumer_group,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
            serviice_url: self.serviice_url,
        }
    }
    fn with_consumer_group(self, consumer_group: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: consumer_group.to_string(),
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            materializer_ack: self.materializer_ack,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
            serviice_url: self.serviice_url,
        }
    }

    fn with_partitions(self, partitions: i32) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: self.consumer_group,
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            materializer_ack: self.materializer_ack,
            partitions,
            rocksdb_path: self.rocksdb_path,
            serviice_url: self.serviice_url,
        }
    }

    fn with_rocksdb_path(self, rocksdb_path: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: self.consumer_group,
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            materializer_ack: self.materializer_ack,
            partitions: self.partitions,
            rocksdb_path: rocksdb_path.to_string(),
            serviice_url: self.serviice_url,
        }
    }

    fn with_service_url(self, service_url: &str) -> Kafence {
        Kafence {
            client_id: self.client_id,
            brokers: self.brokers,
            topic: self.topic,
            topic_router: self.topic_router,
            consumer_group: self.consumer_group,
            routed_consumer_group: self.routed_consumer_group,
            route_table: self.route_table,
            materializer_ack: self.materializer_ack,
            partitions: self.partitions,
            rocksdb_path: self.rocksdb_path,
            serviice_url: service_url.to_string(),
        }
    }

    async fn stream(&self) -> Result<()> {
        let (sender, recv): (UnboundedSender<RouteInfo>, UnboundedReceiver<RouteInfo>) =
            tokio::sync::mpsc::unbounded_channel();
        let kafence_stream = self.clone();
        let stream_task = task::spawn(async move { kafence_stream.create_stream(sender).await });
        tokio::time::sleep(Duration::from_secs(2)).await;
        let kafence_route_table = self.clone();
        let stream_partition_owner_task =
            task::spawn(async move { kafence_route_table.create_routed_stream(recv).await });
        assert!(
            !stream_task.is_finished(),
            "stream task stopped before producing"
        );
        assert!(
            !stream_partition_owner_task.is_finished(),
            "stream partition owner task stopped before producing"
        );
        Ok(())
    }

    fn producer(&self) -> Result<KafenceProducer, KafenceProducerError> {
        match ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("message.timeout.ms", "5000")
            .create()
        {
            Ok(producer) => Ok(KafenceProducer {
                producer: producer,
                service_url: self.serviice_url.clone(),
                topic_router: self.topic_router.clone(),
                partitions: self.partitions,
                route_table: self.route_table.clone(),
                materializer_ack: self.materializer_ack.clone(),
            }),
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

    async fn create_stream(&self, sender: UnboundedSender<RouteInfo>) -> Result<()> {
        let rocks_db = open_rocksdb(&self.rocksdb_path)?;
        let context = KafenceConsumerContext {
            topic: self.topic.to_string(),
            service_host: self.serviice_url.to_string(),
            router_channel_sender: sender,
        };
        let stream_consumer = create_consumer(
            &self.client_id,
            &self.brokers,
            &self.consumer_group,
            context,
        )?;
        stream_consumer.subscribe(&[self.topic.as_str()])?;
        materialize_loop(
            &self.client_id,
            &stream_consumer,
            &rocks_db,
            &self.materializer_ack,
        )
        .await
    }

    async fn create_routed_stream(&self, recv: UnboundedReceiver<RouteInfo>) -> Result<()> {
        create_route_topic_if_not_exists(&self.brokers, &self.topic_router).await?;
        let stream_consumer = create_consumer(
            &self.client_id,
            &self.brokers,
            &self.routed_consumer_group,
            DefaultConsumerContext,
        )?;
        stream_consumer.subscribe(&[self.topic_router.as_str()])?;

        let brokers = self.brokers.clone();
        let topic_router = self.topic_router.clone();
        tokio::task::spawn(async move {
            publish_route_info(&brokers, &topic_router, recv).await;
        });
        materialize_route_loop(&self.client_id, &self.route_table, stream_consumer).await
    }
}

async fn create_route_topic_if_not_exists(brokers: &str, topic: &str) -> anyhow::Result<()> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("session.timeout.ms", "6000")
        .create()?;

    //TODO:Make replication factor configurable
    let new_topic =
        NewTopic::new(topic, 1, TopicReplication::Fixed(1)).set("cleanup.policy", "compact");
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

type KafenceStreamConsumer = StreamConsumer<KafenceConsumerContext>;

fn create_consumer<C>(
    client_id: &str,
    brokers: &str,
    group_id: &str,
    context: C,
) -> Result<StreamConsumer<C>, KafkaError>
where
    C: ConsumerContext + 'static,
{
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group_id)
        .set("client.id", client_id)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .create_with_context(context)
        .map_err(Into::into)
}

fn open_rocksdb(path: &str) -> Result<DB, rocksdb::Error> {
    let mut options = Options::default();
    options.create_if_missing(true);
    Ok(DB::open(&options, path)?)
}

async fn materialize_loop(
    client_id: &str,
    stream_consumer: &KafenceStreamConsumer,
    rocks_db: &DB,
    materializer_ack: &MaterializerAck,
) -> Result<()> {
    loop {
        let message = stream_consumer.recv().await?;
        materialize_rocksdb(client_id, &message, rocks_db, materializer_ack)?;
        stream_consumer.commit_message(&message, CommitMode::Async)?;
    }
}

async fn materialize_route_loop(
    client_id: &str,
    route_table: &Arc<RwLock<HashMap<String, String>>>,
    stream_consumer: StreamConsumer,
) -> Result<()> {
    loop {
        let message = stream_consumer.recv().await?;
        materialize_route_table(client_id, &message, route_table)?;
        stream_consumer.commit_message(&message, CommitMode::Async)?;
    }
}

fn materialize_rocksdb(
    client_id: &str,
    message: &BorrowedMessage<'_>,
    rocks_db: &DB,
    materializer_ack: &MaterializerAck,
) -> Result<()> {
    let key = materialized_key(message);
    let materializer_key = String::from_utf8_lossy(&key).into_owned();

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
    acknowledge_materialized_key(materializer_ack, &materializer_key);
    Ok(())
}

fn materialize_route_table(
    client_id: &str,
    message: &BorrowedMessage<'_>,
    route_table: &Arc<RwLock<HashMap<String, String>>>,
) -> Result<()> {
    let key = materialized_key(message);

    match message.payload() {
        Some(value) => {
            println!(
                "Materialized partition owner client_id={} topic={} partition={} offset={} key={:?} value={:?}",
                client_id,
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key),
                String::from_utf8_lossy(value)
            );
            let mut route_table = route_table.write().unwrap();
            let key = String::from_utf8_lossy(&key).to_string();
            let value = String::from_utf8_lossy(value).to_string();
            route_table.insert(key, value);
        }
        None => {
            println!(
                "Deleted tombstone topic={} partition={} offset={} key={:?}",
                message.topic(),
                message.partition(),
                message.offset(),
                String::from_utf8_lossy(&key)
            );
            let key = String::from_utf8_lossy(&key).to_string();
            route_table.write().unwrap().remove(&key);
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

fn acknowledge_materialized_key(acknowledge: &MaterializerAck, key: &str) {
    if let Some(waiters) = acknowledge.write().unwrap().remove(key) {
        for waiter in waiters {
            let _ = waiter.send(());
        }
    }
}

// Kafka Producer
// --------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum StrongConsistencyPath {
    Local,
    Proxied(String),
}

trait KafenceProducerContract<K, V> {
    async fn strong_consistency(
        &self,
        topic: &str,
        key: K,
        value: V,
    ) -> Result<StrongConsistencyPath>;

    async fn local_persistance(
        &self,
        topic: &str,
        partition: i32,
        key: &K,
        value: &V,
    ) -> Result<()>;
}

impl<K: ToBytes + Send + Sync + Clone + 'static, V: ToBytes + Send> KafenceProducerContract<K, V>
    for KafenceProducer
{
    async fn strong_consistency(
        &self,
        topic: &str,
        key: K,
        value: V,
    ) -> Result<StrongConsistencyPath> {
        let partition = partition_for_key(key.to_bytes(), self.partitions);
        let route_key = format!("{}:{}", self.topic_router, partition);

        let key_string = String::from_utf8_lossy(key.to_bytes()).into_owned();
        let value_string = String::from_utf8_lossy(value.to_bytes()).into_owned();

        let target_host = self.route_table.read().unwrap().get(&route_key).cloned();

        match target_host {
            Some(target_host) if target_host == self.service_url => {
                self.local_persistance(topic, partition, &key, &value)
                    .await?;
                Ok(StrongConsistencyPath::Local)
            }
            Some(target_host) => {
                proxy_strong_consistency(&target_host, key_string, value_string).await?;
                Ok(StrongConsistencyPath::Proxied(target_host))
            }
            None => {
                anyhow::bail!("route not ready for {route_key}");
            }
        }
    }

    async fn local_persistance(
        &self,
        topic: &str,
        partition: i32,
        key: &K,
        value: &V,
    ) -> Result<()> {
        let materializer_key = String::from_utf8_lossy(key.to_bytes()).into_owned();
        let materialized =
            lock_materialization_key(&self.materializer_ack, materializer_key.clone()).await;

        println!(
            "Instance for key={:?} owner of partition={:?} persitance locally.",
            String::from_utf8_lossy(key.to_bytes()),
            partition
        );
        let record = FutureRecord::to(topic)
            .partition(partition)
            .key(key)
            .payload(value);

        match self.producer.send(record, Duration::from_secs(5)).await {
            Ok(_) => {}
            Err((e, _)) => {
                acknowledge_materialized_key(&self.materializer_ack, &materializer_key);
                anyhow::bail!("record delivery failed: {e}");
            }
        }

        materialized.await.map_err(|_| {
            anyhow::anyhow!("materializer acknowledge dropped for key {materializer_key}")
        })?;

        Ok(())
    }
}
async fn proxy_strong_consistency(target_host: &str, key: String, value: String) -> Result<()> {
    println!(
        "Instance for key={:?} is not owner of partition. Proxy request to {} .",
        String::from_utf8_lossy(key.to_bytes()),
        target_host
    );
    let request = hyper::Request::builder()
        .method(hyper::Method::POST)
        .uri(format!("{}/", target_host.trim_end_matches('/')))
        .header("record-key", key)
        .header("x-kafence-proxied", "true")
        .body(hyper::Body::from(value))?;

    let response = hyper::Client::new().request(request).await?;

    if !response.status().is_success() {
        anyhow::bail!("proxy failed with status {}", response.status());
    }

    Ok(())
}

async fn lock_materialization_key(
    acknowledge: &MaterializerAck,
    key: String,
) -> oneshot::Receiver<()> {
    loop {
        let wait_previous = {
            match acknowledge.write().unwrap().entry(key.clone()) {
                Entry::Vacant(entry) => {
                    let (sender, receiver) = oneshot::channel();
                    entry.insert(vec![sender]);
                    return receiver;
                }
                Entry::Occupied(mut entry) => {
                    let (sender, receiver) = oneshot::channel();
                    entry.get_mut().push(sender);
                    receiver
                }
            }
        };

        let _ = wait_previous.await;
    }
}

async fn publish_route_info(
    brokers: &str,
    topic_router: &str,
    mut recv: UnboundedReceiver<RouteInfo>,
) {
    while let Some(route_info) = recv.recv().await {
        let partitions = route_info
            .paritions
            .read()
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        println!("New Route info {:?}", route_info);
        match ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create::<FutureProducer>()
        {
            Ok(producer) => {
                for partition in partitions {
                    let key = format!("{}:{}", topic_router, partition);
                    let record = FutureRecord::to(topic_router)
                        .key(&key)
                        .payload(&route_info.service_host);

                    producer
                        .send(record, Duration::from_secs(5))
                        .await
                        .expect("record must be delivered");
                }
            }
            Err(e) => {
                println!("Error creating Kafka producer. Caused by {}", e);
            }
        }
    }
}

use rdkafka::bindings::rd_kafka_msg_partitioner_murmur2;
use std::ffi::c_void;
use std::ptr;

fn partition_for_key(key: &[u8], partition_count: i32) -> i32 {
    unsafe {
        rd_kafka_msg_partitioner_murmur2(
            ptr::null(),
            key.as_ptr().cast::<c_void>(),
            key.len(),
            partition_count,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    }
}

#[cfg(test)]
mod tests;
