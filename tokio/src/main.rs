use std::thread;

mod tokio_async;
mod tokio_green_thread;
mod tokio_http_hyper;
mod tokio_select;
mod tokio_grpc_server;


#[tokio::main]
async fn main() {
    // tokio_async::run().await;
    // tokio_green_thread::run().await;
    // tokio_http_hyper::run_server().await;
    // tokio_http_hyper::run_client().await;
    // tokio_select::race_condition().await;
    // server::run_server();

}

