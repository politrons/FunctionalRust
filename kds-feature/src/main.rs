use std::{
    str,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use anyhow::Result;
use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use aws_sdk_kinesis::{
    config::Credentials,
    primitives::Blob,
    types::{ShardIteratorType, StreamStatus},
    Client,
};

// ---------- MAIN ----------
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // ——— configuration & client ——————————————————————————
    let creds = Credentials::new("test", "test", None, None, "static");
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let shared_cfg = sdk_config(creds, region_provider).await;
    let client = Client::new(&shared_cfg);

    // ——— make sure the stream exists and is ACTIVE ——————————
    ensure_stream_ready(&client, 4).await?;

    // ——— produce 1 000 records ————————————————————————————
    produce_records(&client, 1_000).await?;

    // ——— consume in parallel ————————————————————————————
    let shards = list_shards(&client).await?;
    let expected = 1_000usize;
    let counter = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();

    // launch one task per shard
    let mut handles = Vec::new();
    for shard_id in shards {
        let c = client.clone();
        let cnt = Arc::clone(&counter);
        handles.push(tokio::spawn(async move {
            consume_shard(c, &shard_id, cnt, expected).await
        }));
    }

    // wait for every task to finish
    for h in handles {
        h.await??;
    }

    let elapsed = start.elapsed();
    println!("Read {expected} records in {:?}\n", elapsed);

    Ok(())
}
// ---------- PRODUCER ----------
async fn produce_records(client: &Client, n: usize) -> Result<()> {
    for i in 0..n {
        client
            .put_record()
            .stream_name("test-stream")
            .partition_key(format!("pk-{}", i % 4)) // spread over 4 shards
            .data(Blob::new(format!("hello-{:04}", i)))
            .send()
            .await?;
    }
    Ok(())
}
// ---------- CONSUMER (per shard) ----------
async fn consume_shard(
    client: Client,
    shard_id: &str,
    total: Arc<AtomicUsize>,
    expected: usize,
) -> Result<()> {
    // iterator at the very beginning of the shard
    let mut it = client
        .get_shard_iterator()
        .stream_name("test-stream")
        .shard_id(shard_id)
        .shard_iterator_type(ShardIteratorType::TrimHorizon)
        .send()
        .await?
        .shard_iterator()
        .unwrap()
        .to_owned();

    while total.load(Ordering::Relaxed) < expected {
        let out = client.get_records().shard_iterator(&it).send().await?;
        it = match out.next_shard_iterator() {
            Some(next) => next.to_owned(),
            None => break, // shard ended / closed
        };
        for rec in out.records() {
            let data = str::from_utf8(rec.data.as_ref())?;
            let pk = rec.partition_key();
            println!("{} ⇒ {}", pk, data);
        }
        total.fetch_add(out.records().len(), Ordering::Relaxed);
        // small pause so we don’t overwhelm LocalStack
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    Ok(())
}
// ---------- HELPERS ----------
async fn ensure_stream_ready(client: &Client, shards: i32) -> Result<()> {
    // create the stream only if it doesn’t exist
    if client
        .describe_stream_summary()
        .stream_name("test-stream")
        .send()
        .await
        .is_err()
    {
        client
            .create_stream()
            .stream_name("test-stream")
            .shard_count(shards)
            .send()
            .await?;
    }

    // wait until the status becomes ACTIVE
    loop {
        let output = client
            .describe_stream_summary()
            .stream_name("test-stream")
            .send()
            .await?;
        let status = output
            .stream_description_summary() // Option<&StreamDescriptionSummary>
            .unwrap()
            .stream_status();            
        if status.clone() == StreamStatus::Active {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
}

async fn sdk_config(creds: Credentials, region: RegionProviderChain) -> SdkConfig {
    aws_config::from_env()
        .endpoint_url("http://localhost:4566") // LocalStack endpoint
        .region(region)
        .credentials_provider(creds)
        .load()
        .await
}

async fn list_shards(client: &Client) -> Result<Vec<String>> {
    let resp = client
        .list_shards()
        .stream_name("test-stream")
        .send()
        .await?;

    // `shards()` gives a slice, so just iterate over it.
    let shard_ids: Vec<String> = resp
        .shards()            // &[Shard]
        .iter()
        .filter_map(|shard| {
            Some(shard
                .shard_id().to_owned())  // Option<&str>

        })
        .collect();

    Ok(shard_ids)
}




