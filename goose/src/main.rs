use std::time::Duration;

use goose::prelude::*;

mod mock_http_server;
mod goose_load_test;

#[tokio::main]
async fn main() {
    // mock_http_server::run_server().await;
    let result = goose_load_test::run().await;
    println!("{:?}", result)
}
