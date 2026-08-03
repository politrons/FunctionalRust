use futures::executor::block_on;
use std::future::Future;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::{thread, time};

pub fn run() {
    local_channel();
}

fn local_channel() {
    let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
    let sender_future = send_message(sender);
    let receive_future = receive_message(receiver);
    block_on(sender_future);
    block_on(receive_future);
}

async fn send_message(sender: Sender<String>) {
    async_std::task::sleep(Duration::from_secs(2)).await;
    let ack = sender.send(String::from("Hello channel"));
    match ack {
        Ok(()) => println!("Message sent successful"),
        Err(error) => println!("{}", error.to_string()),
    }
}

async fn receive_message(receiver: Receiver<String>) {
    let result = receiver.recv();
    match result {
        Ok(v) => println!("Received: {}", v),
        Err(error) => println!("{}", error.to_string()),
    }
}

// Channel DSL
// ---------------

trait ChannelConnector<T> {
    async fn send_message(&self, message: T);

    fn subscribe_channel(&self) -> Vec<T>;
}

struct Channel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> Channel<T> {
    fn new() -> Channel<T> {
        let (sender, receiver): (Sender<T>, Receiver<T>) = mpsc::channel();
        Channel { sender, receiver }
    }
}

impl<T> ChannelConnector<T> for Channel<T> {
    async fn send_message(&self, message: T) {
        self.sender.send(message).unwrap()
    }

    fn subscribe_channel(&self) -> Vec<T> {
        let mut messages = Vec::new();
        while let Ok(message) = self.receiver.try_recv() {
            messages.push(message);
        }
        messages
    }
}

#[cfg(test)]
mod tests {
    use crate::features::channels_feature::{local_channel, Channel, ChannelConnector};
    use async_std::task::block_on;
    use futures::future::join;

    #[test]
    fn test_local_channel() {
        local_channel();
    }

    #[test]
    fn test_dsl() {
        let channel = Channel::<String>::new();
        let send_fut_1 = channel.send_message(String::from("hello"));
        let send_fut_2 = channel.send_message(String::from("channel"));
        block_on(join(send_fut_1, send_fut_2));
        let messages = channel.subscribe_channel();
        println!("message: {:?}", messages);
    }
}
