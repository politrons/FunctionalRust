use SQUICD::dsl::{MessageHandler,  SquidDSL};
use SQUICD::common::Message;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the message handler
    let handler = |message: Message| -> Result<(), Box<dyn std::error::Error>> {
        println!("DSL received message: {:?}", message);
        // Process the message within your DSL
        Ok(())
    };

    
    // Start the server
    SquidDSL::start_server("0.0.0.0:4433".to_string(), Arc::new(handler));

    // Proceed with the rest of your DSL initialization
    // ...

    // Keep the main thread alive or proceed as appropriate for your application
    loop {
        std::thread::park();
    }

    // Ok(())
}
