use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use SQUICD::common::Message;
use SQUICD::dsl::SQUID;

fn main() -> Result<(), Box<dyn Error>> {
    SQUID::new()
        .with_handler(Arc::new(|message: Message| {
            println!("Received message: {:?}", message);
            // Process the message as needed
            thread::sleep(Duration::from_secs(1));
            // Optionally, send a message back
            let new_message = Message {
                id: message.id + 1,
                content: message.content.clone(),
                timestamp: message.timestamp,
            };
            if let Err(e) = SQUID::send_message("127.0.0.1:4433", new_message) {
                eprintln!("Error sending message: {:?}", e);
            }
        }))
        .start_server("0.0.0.0:4433".to_string())
        .run();
}
