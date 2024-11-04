use std::thread;
use std::time::Duration;

use SQUICD::common::Message;
use SQUICD::dsl::Squicd;

fn main() -> ! {
    thread::spawn(move || {
        run_server();
    });
    run_client();
    loop {
        std::thread::park();
    }
}

fn run_server() {
    Squicd::with_handler(
        |message| {
            println!("Received message: {:?}", message);
            thread::sleep(Duration::from_secs(1));
            let new_message = Message {
                id: message.id + 1,
                content: message.content.clone(),
                timestamp: message.timestamp,
            };
            //Send message to next service.
            if let Err(e) = Squicd::send_message("127.0.0.1:4433", new_message) {
                eprintln!("Error sending message: {:?}", e);
            }
        })
        .with_error_handler(
            |err| {
                println!("Side-effect handle: {:?}", err);
            })
        .with_cert("cert.crt")
        .with_key("cert.key")
        .with_port("4433")
        .start();
}

fn run_client() {
    let message = Message {
        id: 1,
        content: "Hello, Squicd!".to_string(),
        timestamp: 1234567890,
    };

    if let Err(e) = Squicd::send_message("127.0.0.1:4433", message) {
        eprintln!("Error sending message: {:?}", e);
    }
}

