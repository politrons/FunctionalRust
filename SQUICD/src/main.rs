use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use SQUICD::common::Message;
use SQUICD::dsl::Squicd;

fn main() -> Result<(), Box<dyn Error>> {
    Squicd::with_handler(Arc::new(|message: Message| {
        println!("Received message: {:?}", message);
        // Process the message as needed
        thread::sleep(Duration::from_secs(1));
        // Optionally, send a message back
        let new_message = Message {
            id: message.id + 1,
            content: message.content.clone(),
            timestamp: message.timestamp,
        };
        if let Err(e) = Squicd::send_message("127.0.0.1:4433", new_message) {
            eprintln!("Error sending message: {:?}", e);
        }
    })).run("0.0.0.0:4433".to_string());
}
