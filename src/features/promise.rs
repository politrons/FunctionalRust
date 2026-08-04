use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SendError};
use std::sync::mpsc::Sender;
use std::{thread, time};
use std::time::Duration;

/// We can implement the [Promise] pattern in Rust using [Channels]
/// We create a [Sender(Promise)] and the [Receiver(Future)].
/// we can run an async task passing the Sender channel, and then continue the logic of your program
/// without blocks.
/// Then once you need to get the result form the Promise, you can subscribe from the [Receiver]
/// and once the Promise finish you will receive the [Result] with the value specify in the type,
/// or side-effect [RecvError]
fn promise_feature() {
    let (promise, future): (Sender<String>, Receiver<String>) = mpsc::channel();
    async_std::task::spawn(async_task(promise));
    println!("Continue the work....");
    let result = future.recv();
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

// DSL promise
// ------------
struct Promise<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

struct PromiseCompleter<T> {
    sender: Sender<T>,
}

trait PromiseI<T> {
    fn promise() -> Promise<T>;
    fn completer(&self) -> PromiseCompleter<T>;
    fn future(&self, timeout: Duration) -> Result<T, RecvTimeoutError>;
}

impl<T> PromiseI<T> for Promise<T> {
    fn promise() -> Promise<T> {
        let (sender, receiver): (Sender<T>, Receiver<T>) = mpsc::channel();
        Promise { sender, receiver }
    }

    fn completer(&self) -> PromiseCompleter<T> {
        PromiseCompleter {
            sender: self.sender.clone(),
        }
    }

    fn future(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
}

impl<T> PromiseCompleter<T> {
    fn succeed(&self, value: T) -> Result<(), SendError<T>> {
        self.sender.send(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promise_dsl() {
        let promise: Promise<String> = Promise::promise();
        let completer = promise.completer();

        async_std::task::spawn(async move{
            async_std::task::sleep(time::Duration::from_secs(2)).await;
            completer.succeed(String::from("test")).unwrap();
        });

        match promise.future(Duration::from_secs(5)) {
            Ok(value) => {println!("message received: {value}")}
            Err(error) => {println!("error {}", error.to_string())}
        }
    }

    #[test]
    fn promise() {
        promise_feature()
    }
}
