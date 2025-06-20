use std::error::Error;
use aws_sdk_kinesis::{types::ShardIteratorType, Client};
use aws_config::meta::region::RegionProviderChain;
use anyhow::Result;
use aws_config::SdkConfig;
use aws_sdk_kinesis::primitives::Blob;
use aws_sdk_kinesis::config::Credentials;
use aws_sdk_kinesis::operation::get_records::GetRecordsOutput;

#[tokio::main]
async fn main() -> Result<()> {
    
    // Static credentials for LocalStack ‒ NEVER use hard-coded keys in production.
    let creds = create_credentials();

    // Build shared AWS config pointing to the local endpoint
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let shared_config = create_sdk_config(creds, region_provider).await;
    let client = Client::new(&shared_config);

    // Ensure the stream exists; create it if it does not
    ensure_stream_is_ready(&client).await?;

    //Produce a single record
    produce_record(&client).await?;

    // Consume records from the beginning of the shard
    let recs = consume_records(client).await?;

    // Print each record’s payload as UTF-8 text
    for record in recs.records(){
            let value = std::str::from_utf8(record.data.as_ref())?;
            println!("partition:{} value:{}",record.partition_key, value);
    }

    Ok(())
}

async fn consume_records(client: Client) -> Result<GetRecordsOutput> {
    let it = client
        .get_shard_iterator()
        .stream_name("test-stream")
        .shard_id("shardId-000000000000")
        .shard_iterator_type(ShardIteratorType::TrimHorizon)
        .send()
        .await?;

    let recs = client
        .get_records()
        .shard_iterator(it.shard_iterator().unwrap())
        .send()
        .await?;
    Ok(recs)
}

async fn ensure_stream_is_ready(client: &Client) -> Result<()> {
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
            .shard_count(1)
            .send()
            .await?;

        // Wait until the stream status becomes ACTIVE
        use aws_sdk_kinesis::types::StreamStatus;
        use std::time::Duration;
        loop {
            let output = client
                .describe_stream_summary()
                .stream_name("test-stream")
                .send()
                .await?;
            let status = output
                .stream_description_summary() // Option<&StreamDescriptionSummary>
                .unwrap()
                .stream_status();             // Option<&StreamStatus>

            if status.clone() == StreamStatus::Active {
                break;                        // Stream is ready
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    Ok(())
}

async fn produce_record(client: &Client) -> Result<()> {
    client
        .put_record()
        .stream_name("test-stream")
        .partition_key("p1")
        .data(Blob::new("Hello Kinesis!"))
        .send()
        .await?;
    Ok(())
}

async fn create_sdk_config(creds: Credentials, region_provider: RegionProviderChain) -> SdkConfig {
    aws_config::from_env()
        .endpoint_url("http://localhost:4566")
        .region(region_provider)
        .credentials_provider(creds)
        .load()
        .await
}

fn create_credentials() -> Credentials {
    Credentials::new(
        "test",     // access key
        "test",     // secret key
        None,       // session token
        None,       // expiry
        "static",   // provider name (only appears in logs)
    )
}
