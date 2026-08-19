# kafence

`kafence` is a Rust library for building Kafka-backed streams where a write is not considered complete until the stream that owns the target partition has materialized the record in RocksDB.

Kafka already provides ordering inside a partition. The harder part in a distributed service is that an HTTP request can land on an instance that does not own the partition for the record key. `kafence` keeps a local routing table for partition ownership and can proxy the write to the correct service instance when needed.

## How It Works

Each service instance starts two streams:

- A business stream that consumes the main topic and materializes records into RocksDB.
- A routing stream that consumes a compacted topic containing the current owner for each partition.

When Kafka assigns partitions to an instance, `kafence` publishes that ownership information to the routing topic. Every instance consumes that compacted topic and keeps a local copy of the route table.

When the application calls `strong_consistency`, the library calculates the partition for the key using Kafka's partitioner. If the current instance owns that partition, it produces the record and waits until the materializer writes it into RocksDB. If another instance owns the partition, the call is proxied to that instance and the response is returned to the original caller.

## DSL Configuration

The DSL contains only the values each service instance needs to connect to Kafka, start its streams, materialize state, and participate in routing.

| Method | Purpose |
| --- | --- |
| `with_brokers` | Kafka bootstrap servers, for example `localhost:9092`. |
| `with_topic` | Business topic consumed and materialized by the stream. |
| `with_consumer_group` | Shared consumer group used by all instances of the service. |
| `with_partitions` | Number of partitions in the business topic. |
| `with_rocksdb_path` | Local RocksDB directory for this instance. |
| `with_service_url` | Public URL for this instance, used when another instance needs to proxy a write. |

The routing topic name is derived from the business topic. For example, `orders` uses `orders_router`.

## Basic Usage

This is the intended shape from the service point of view: create the DSL, start the streams during service startup, then reuse the producer from request handlers.

```rust
use kafence::{Kafence, KafenceProducerContract};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let kafence = Kafence::new()
        .with_brokers("localhost:9092".to_string())
        .with_topic("orders")
        .with_consumer_group("orders-service")
        .with_partitions(2)
        .with_rocksdb_path("./state/orders-store")
        .with_service_url("http://127.0.0.1:8080")
        .build();

    kafence.stream().await?;

    let producer = kafence.producer()?;

    producer
        .strong_consistency("orders", "order-123", "created")
        .await?;

    Ok(())
}
```

In a real service, `kafence.stream().await` belongs in the startup path. Each endpoint that needs a strongly consistent write calls `strong_consistency`.

## Endpoint Example

```rust
use std::sync::Arc;

async fn create_order(
    kafence: Arc<Kafence>,
    order_id: String,
    payload: String,
) -> anyhow::Result<()> {
    let producer = kafence.producer()?;

    producer
        .strong_consistency("orders", order_id, payload)
        .await?;

    Ok(())
}
```

The caller does not need to know whether the key belongs to the local instance or a remote one. That decision stays inside `strong_consistency`.

## Consistency Model

`strong_consistency` does not return immediately after sending a record to Kafka. For a local write, it waits until the consumer receives the record, writes it into RocksDB, and releases the key that was blocked for materialization.

This lets an HTTP endpoint respond only after the stream state reflects the write. If the partition belongs to another instance, the request is routed to the owner and the final response is propagated back to the original caller.

## Operational Notes

- The business topic must exist with the same partition count configured in `with_partitions`.
- All instances of the same service must use the same `consumer_group`.
- Each instance must publish its own `service_url`.
- RocksDB is local to each service instance; it is not shared storage.
- The routing topic should be compacted so Kafka keeps the latest owner for each partition.
