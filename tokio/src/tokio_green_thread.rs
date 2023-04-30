use std::thread;

pub async fn run() {
    let handle = tokio::spawn(async {
        // Do some async work
        println!("Running in Thread:{:?}", thread::current().name());
        "return value"
    });

    // Do some other work

    let out = handle.await.unwrap();
    println!("GOT {}", out);
}