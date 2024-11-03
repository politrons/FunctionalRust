use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use SQUICD::common::Message;
use SQUICD::dsl::Squicd;

fn main() -> Result<(), Box<dyn Error>> {
    Squicd::with_handler(
        |message: Message| {
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
        .with_cert("cert.crt")
        .with_key("cert.key")
        .with_port("4433")
        .start();
}
