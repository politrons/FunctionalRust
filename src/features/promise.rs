use std::{thread, time};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::io;
use std::net::TcpListener;

use futures::executor::block_on;

/// We can implement the [Promise] pattern in Rust using [Channels]
/// We create a [Sender(Promise)] and the [Receiver(Future)].
/// we can run an async task passing the Sender channel, and then continue the logic of your program
/// without blocks.
/// Then once you need to get the result form the Promise, you can subscribe from the [Receiver]
/// and once the Promise finish you will receive the [Result] with the value specify in the type,
/// or side-effect [RecvError]
fn promise_feature() {
    let (promise, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
    async_std::task::spawn(async_task(promise));
    println!("Continue the work....");
    let result = receiver.recv();
    match result {
        Ok(v) => println!("Received: {}", v),
        Err(error) => println!("{}", error.to_string()),
    }
}

/// Here we do an async computation that it will take 2 seconds. Then we will finish the promise
/// using [send] operator.
async fn async_task(promise: Sender<String>) {
    thread::sleep(time::Duration::from_secs(2));
    let ack = promise.send(String::from("I finish my task successfully"));
    match ack {
        Ok(()) => println!("Promise sent successfully"),
        Err(error) => println!("{}", error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promise() {
        promise_feature()
    }
}





