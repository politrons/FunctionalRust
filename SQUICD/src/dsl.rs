use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use quiche::{Config, ConnectionId, Header, RecvInfo};
use rand::Rng;
use std::error::Error;
use std::sync::{Arc};
use std::thread;
use std::time::Duration;
use ring::rand::SystemRandom;
// Import the common module
use crate::common::{Message, serialize_and_compress, decompress_and_deserialize};
use ring::rand::*;


pub struct Squicd {
    result: Result<(), Box<dyn Error>>,
    port: String,
    cert: String,
    key: String,
    handler: Arc<MessageHandler>,
}

// Type alias for the message handler callback
pub type MessageHandler = dyn Fn(Message) -> () + Send + Sync + 'static;

impl Squicd {
    pub fn with_handler(handler: Arc<MessageHandler>) -> Self {
        Squicd {
            result: Ok(()),
            port: "".to_string(),
            cert: "".to_string(),
            key: "".to_string(),
            handler,
        }
    }

    pub fn with_port(&mut self, port: &str) -> &mut Self {
        self.port = port.to_string();
        self
    }

    pub fn with_cert(&mut self, cert: &str) -> &mut Self {
        self.cert = cert.to_string();
        self
    }

    pub fn with_key(&mut self, key: &str) -> &mut Self {
        self.key = key.to_string();
        self
    }


    pub fn start(&self) -> ! {
        // Clone to move into the thread
        let addr = format!("{}:{}", "0.0.0.0", self.port.clone());
        let handler = self.handler.clone();
        let cert = self.cert.clone();
        let key = self.key.clone();
        // Spawn a new thread for the server
        thread::spawn(move || {
            // Maximum datagram size
            const MAX_DATAGRAM_SIZE: usize = 1350; // Standard MTU size

            // Bind to the specified UDP port
            let socket = UdpSocket::bind(addr).expect("Failed to bind to address");

            // Create QUIC configuration
            let mut config = Config::new(quiche::PROTOCOL_VERSION).expect("Failed to create config");
            config.set_application_protos(&[b"example-proto"]).expect("Failed to set ALPN");
            config.load_cert_chain_from_pem_file(cert.as_str()).expect("Failed to load cert");
            config.load_priv_key_from_pem_file(key.as_str()).expect("Failed to load key");
            config.set_max_idle_timeout(30_000);
            config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
            config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
            config.set_initial_max_data(10_000_000);
            config.set_initial_max_stream_data_bidi_remote(1_000_000);
            config.set_initial_max_streams_bidi(100);
            config.set_disable_active_migration(true);
            config.verify_peer(false); // Disable peer verification for testing

            let mut connections: HashMap<ConnectionId<'static>, (quiche::Connection, SocketAddr)> =
                HashMap::new();
            let mut connection_ids: HashMap<ConnectionId<'static>, ConnectionId<'static>> =
                HashMap::new();

            let mut buf = [0u8; 65535];
            let mut out = [0u8; MAX_DATAGRAM_SIZE];

            loop {
                // Receive data from a client
                let (read, from) = match socket.recv_from(&mut buf) {
                    Ok((len, addr)) => (len, addr),
                    Err(e) => {
                        eprintln!("Failed to receive data: {:?}", e);
                        continue;
                    }
                };

                // Parse the QUIC packet header
                let hdr = match Header::from_slice(&mut buf[..read], quiche::MAX_CONN_ID_LEN) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Failed to parse header: {:?}", e);
                        continue;
                    }
                };

                // Get DCID from the header
                let dcid = hdr.dcid.clone();

                // Map DCID to internal SCID
                let conn_id = if let Some(scid) = connection_ids.get(&dcid) {
                    scid.clone()
                } else {
                    dcid.clone()
                };

                // Retrieve the associated connection
                let (conn, _) = if let Some(conn_socket_addr) = connections.get_mut(&conn_id) {
                    conn_socket_addr
                } else {
                    // println!("Received packet pre-handshake with DCID: {:?}", dcid);

                    // Generate a new SCID for the server
                    let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
                    rand::thread_rng().fill(&mut scid_bytes);
                    let scid = ConnectionId::from_vec(scid_bytes.to_vec());

                    // Accept the new connection
                    let conn = quiche::accept(
                        &scid,
                        Some(&hdr.dcid),
                        socket.local_addr().unwrap(),
                        from,
                        &mut config,
                    ).expect("Failed to accept connection");

                    // println!("New connection from {}", from);

                    // Store the connection
                    connections.insert(scid.clone(), (conn, from));
                    connection_ids.insert(hdr.dcid.clone(), scid.clone());

                    // Retrieve the connection
                    connections.get_mut(&scid).unwrap()
                };

                // Information about the received packet
                let recv_info = RecvInfo {
                    from,
                    to: socket.local_addr().unwrap(),
                };

                // Process the received packet
                if let Err(e) = conn.recv(&mut buf[..read], recv_info) {
                    eprintln!("Connection recv failed: {:?}", e);
                    continue;
                }

                // Send any pending packets
                while let Ok((write, send_info)) = conn.send(&mut out) {
                    socket
                        .send_to(&out[..write], send_info.to)
                        .expect("Failed to send data");
                }

                let mut stream_buf = [0u8; 65535];
                for stream_id in conn.readable() {
                    while let Ok((read, fin)) = conn.stream_recv(stream_id, &mut stream_buf) {
                        let data = &stream_buf[..read];

                        if fin {
                            // Decompress and deserialize the message
                            let message = match decompress_and_deserialize(data) {
                                Ok(msg) => msg,
                                Err(e) => {
                                    eprintln!("Failed to decompress/deserialize message: {:?}", e);
                                    continue;
                                }
                            };

                            // Spawn a new thread to handle the message
                            let h = handler.clone();
                            thread::spawn(move || {
                                h(message);
                            });
                        }
                    }
                }


                // Handle connection closure
                if conn.is_closed() {
                    println!("Connection closed with {}", from);
                    connections.remove(&conn_id);

                    // Remove mappings for this connection
                    connection_ids.retain(|_, v| v != &conn_id);
                }
            }
        });
        loop {
            std::thread::park();
        }
    }

    pub fn send_message(
        server_addr: &str,
        message: Message,
    ) -> Result<Message, Box<dyn Error>> {
        // Maximum datagram size
        const MAX_DATAGRAM_SIZE: usize = 1350; // Standard MTU size

        // Create UDP socket bound to an ephemeral port
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        // Server address
        let server_addr = server_addr.parse().expect("Invalid server address");

        // Create QUIC configuration
        let mut config = Config::new(quiche::PROTOCOL_VERSION)?;
        config.set_application_protos(&[b"example-proto"])?;
        config.verify_peer(false);

        // Configure QUIC parameters
        config.set_max_idle_timeout(30_000);
        config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
        config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
        config.set_initial_max_data(10_000_000);
        config.set_initial_max_stream_data_bidi_local(1_000_000);
        config.set_initial_max_streams_bidi(100);
        config.set_disable_active_migration(true);

        // Generate a random SCID
        let mut scid = [0; quiche::MAX_CONN_ID_LEN];
        SystemRandom::new()
            .fill(&mut scid)
            .expect("Failed to generate connection ID");
        let scid = ConnectionId::from_ref(&scid);

        // Create a new QUIC connection to the server
        let mut conn = quiche::connect(
            None,
            &scid,
            socket.local_addr().unwrap(),
            server_addr,
            &mut config,
        )?;

        // Buffers for sending and receiving data
        let mut out = [0; MAX_DATAGRAM_SIZE];
        let mut buf = [0; 65535];

        // Send the initial packet to start the handshake
        let (write, send_info) = conn.send(&mut out)?;
        socket.send_to(&out[..write], send_info.to)?;

        // Variables to track state
        let mut handshake_completed = false;
        let mut message_sent = false;
        let mut response_received = false;
        let mut received_message = None;

        // Initialize stream_id for client-initiated bidirectional streams
        let stream_id = 0u64; // First bidirectional stream

        loop {
            // Wait for data from the server
            socket.set_read_timeout(Some(Duration::from_millis(50)))?;
            match socket.recv_from(&mut buf) {
                Ok((len, from)) => {
                    let recv_info = RecvInfo {
                        from,
                        to: socket.local_addr().unwrap(),
                    };
                    if let Err(e) = conn.recv(&mut buf[..len], recv_info) {
                        eprintln!("Connection recv failed: {:?}", e);
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Timeout, continue
                }
                Err(e) => {
                    eprintln!("Failed to receive data: {:?}", e);
                    break;
                }
            }

            // Once the handshake is established, send application data
            if conn.is_established() && !handshake_completed {
                // println!("Handshake completed");
                handshake_completed = true;
            }

            if handshake_completed && !message_sent {
                // Serialize and compress the message
                let compressed_data = serialize_and_compress(&message)?;

                // Send the compressed data over QUIC
                match conn.stream_send(stream_id, &compressed_data, true) {
                    Ok(_) => {
                        // println!("Message sent on stream {}", stream_id);
                        message_sent = true;
                    }
                    Err(quiche::Error::Done) => {
                        // No more data can be sent at the moment
                    }
                    Err(e) => {
                        eprintln!("Failed to send data on stream {}: {:?}", stream_id, e);
                        break;
                    }
                }
            }

            // Send any pending packets generated by `recv` or application data
            while let Ok((write, send_info)) = conn.send(&mut out) {
                socket.send_to(&out[..write], send_info.to)?;
            }

            // Read responses from the server
            for s_id in conn.readable() {
                while let Ok((read, fin)) = conn.stream_recv(s_id, &mut buf) {
                    let data = &buf[..read];
                    // println!(
                    //     "Client received {} bytes on stream {} (fin: {})",
                    //     data.len(),
                    //     s_id,
                    //     fin
                    // );
                    if fin {
                        // Decompress and deserialize the response
                        let response = decompress_and_deserialize(data)?;
                        // println!("Received response: {:?}", response);
                        received_message = Some(response);
                        response_received = true;
                        // Close the connection gracefully
                        conn.close(false, 0x00, b"done")?;
                        break;
                    }
                }
            }

            // Handle connection close
            if conn.is_closed() {
                println!("Connection closed");
                break;
            }

            if response_received {
                println!("Response received, exiting loop");
                break;
            }
        }

        if let Some(msg) = received_message {
            Ok(msg)
        } else {
            Err("No response received".into())
        }
    }
}


