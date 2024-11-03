# Squicd

Author: Pablo Picouto Garcia

Is this project useful? Please ⭐ Star this repository and share the love.

Squicd is a Domain-Specific Language (DSL) in Rust designed to simplify the creation of QUIC-based servers and clients for message passing.

## Usage

### Setting Up the Server

```rust
use squicd::dsl::Squicd;
use squicd::common::Message;

fn main() {
    Squicd::with_handler(|message: Message| {
            println!("Received message: {:?}", message);
            // Additional processing...
        })
        .with_error_handler(
            |err| {
              // Handle side effects
            })
        .with_cert("cert.crt")
        .with_key("cert.key")
        .with_port("4433")
        .start();
}

```

### Sending Messages
```rust
use squicd::common::Message;
use squicd::dsl::Squicd;

fn main() {
    let message = Message {
        id: 1,
        content: "Hello, Squicd!".to_string(),
        timestamp: 1234567890,
    };

    if let Err(e) = Squicd::send_message("127.0.0.1:4433", message) {
        eprintln!("Error sending message: {:?}", e);
    }
}

```

## How It Works

* Initialization: Configure the server with the provided certificate, key, and port.

* Starting the Server: Call start() to begin listening for incoming QUIC connections.
* Accepting Connections: New QUIC connections are accepted using the quiche library.
* Receiving Messages: Messages are decompressed, deserialized, and passed to the message handler.
* Processing Messages: The message handler processes incoming messages and can send responses or forward messages.
* Sending Messages: Use send_message to establish a QUIC connection and send messages to other services.