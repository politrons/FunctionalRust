use std::error::Error;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use SQUICD::common::Message;
use SQUICD::dsl::SQUID;

fn main() -> Result<(), Box<dyn Error>> {

    // Start the server
    SQUID::start_server("0.0.0.0:4433".to_string(),
                        // Define the message handler
                        Arc::new(|message: Message| -> Result<(), Box<dyn Error>> {
                            println!("DSL received message: {:?}", message);
                            // Process the message within your DSL
                            thread::sleep(Duration::from_secs(1));
                            let new_message = Message {
                                id: message.id + 1,
                                content: message.content,
                                timestamp: message.timestamp,
                            };
                            match SQUID::send_message("127.0.0.1:4433", new_message) {
                                Ok(m) => println!("ACK:{:?}", m.content),
                                Err(e) => println!("Error:{:?}", e.to_string())
                            }
                            Ok(())
                        })).run();


    // Ok(())
}
