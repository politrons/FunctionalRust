# Kinesis Local Benchmark (Rust ‑ AWS SDK v1)

## 1 · Purpose

Benchmark end‑to‑end latency when writing **1 000** records to an Amazon Kinesis‑compatible endpoint (LocalStack) and consuming them concurrently with **4 parallel tasks**.

## 2 · How it works

```
┌────────────┐          ┌──────────────────┐          ┌────────────┐
│ Producer   │─1 000→──▶│ 4 × Kinesis Shard │─records─▶│ Consumer   │
└────────────┘          └──────────────────┘   read   └────────────┘
```

* **Stream creation** – The program ensures a stream called `test-stream` exists with `shard_count = 4`.
* **Produce phase** – Sends 1 000 individual `PutRecord` calls (`partition_key = pk‑(0‥3)`), spreading load evenly across the 4 shards.
* **Consume phase** – Spawns 4 Tokio tasks (one per shard). Each task:

    1. Retrieves an iterator at `TrimHorizon`.
    2. Polls until the global counter reaches 1 000.
* **Timing** – A `std::time::Instant` marks the moment all consumes start and stops once all tasks have finished.

## 3 · Prerequisites

| Tool           | Version (tested)              |
| -------------- | ----------------------------- |
| Rust toolchain | 1.78.0‑stable                 |
| LocalStack     | 3.7+ (with `kinesis` service) |
| Docker         | 24.x                          |


### Expected console output (truncated)

```text
pk-3 ⇒ hello-0996
pk-0 ⇒ hello-0997
pk-1 ⇒ hello-0998
pk-2 ⇒ hello-0999
✅ Read 1000 records in 240.847871ms
```

## 5 · Tweaking the experiment

| Variable            | Location              | Description                                          |
| ------------------- | --------------------- | ---------------------------------------------------- |
| `TOTAL_RECORDS`     | `produce_records()`   | Change the number of events produced                 |
| `SHARD_COUNT`       | `ensure_stream_ready` | Scale shards up/down (and adjust consumer tasks)     |
| `worker_threads`    | `#[tokio::main]`      | Matches number of concurrent shard readers           |
| Sleep between polls | `consume_shard()`     | Tune `tokio::time::sleep(Duration::from_millis(20))` |

> \*\*Tip \*\* For higher throughput in real AWS, switch to `PutRecords` (batch ≤ 500) instead of individual calls.

## 6 · Results & Interpretation

| Records | Shards | Runtime (ms) | Notes                                          |
| ------- | ------ | ------------ | ---------------------------------------------- |
| 1 000   | 4      | ≈ 241 ms     | Measured on M1 Pro / macOS 14 / LocalStack 3.7 |

*LocalStack latency is dominated by Docker networking; real Kinesis usually adds ≈ 10–20 ms per hop but has stricter throughput limits.*

## 8 · License

MIT — do whatever you want, but attribution is appreciated.

---

