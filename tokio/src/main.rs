mod tokio_async;
mod tokio_green_thread;


#[tokio::main]
async fn main() {
    tokio_async::run().await;
    tokio_green_thread::run().await;
}

