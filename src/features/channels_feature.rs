use std::{thread, time};
use std::future::Future;
use std::io;
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError};
use std::sync::mpsc;
use std::sync::mpsc::Sender;

use futures::executor::block_on;

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
    thread::sleep(time::Duration::from_secs(2));
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




