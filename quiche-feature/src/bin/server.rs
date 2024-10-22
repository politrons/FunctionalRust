use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use quiche::{Config, ConnectionId, Header, RecvInfo};
use rand::Rng;
use log::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("Server starting...");

    // Maximum datagram size
    const MAX_DATAGRAM_SIZE: usize = 5000;

    // Bind to UDP port 4433
    let socket = UdpSocket::bind("0.0.0.0:4433")?;

    // Create QUIC configuration
    let mut config = Config::new(quiche::PROTOCOL_VERSION)?;
    // Set ALPN protocols
    config.set_application_protos(&[b"example-proto"])?;
    // Load server's certificate and private key
    config.load_cert_chain_from_pem_file("cert.crt")?;
    config.load_priv_key_from_pem_file("cert.key")?;

    // Configure QUIC parameters
    config.set_max_idle_timeout(30_000); // Increase the timeout to 30 seconds
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(100_000_000); // Increase data limits
    config.set_initial_max_stream_data_bidi_remote(10_000_000);
    config.set_initial_max_streams_bidi(5000);
    config.set_disable_active_migration(true);

    // Disable peer verification (for testing purposes only)
    config.verify_peer(false);

    // Maps to store connections and IDs
    let mut connections: HashMap<ConnectionId<'static>, (quiche::Connection, SocketAddr)> =
        HashMap::new();
    let mut connection_ids: HashMap<ConnectionId<'static>, ConnectionId<'static>> = HashMap::new();

    // Buffers for sending and receiving data
    let mut buf = [0u8; 65535];
    let mut out = [0u8; MAX_DATAGRAM_SIZE];
    let mut stream_data = HashMap::new();

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
            println!("Received packet pre-handshake with DCID: {:?}", dcid);

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
            )?;

            println!("New connection from {}", from);

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
        // Read data from available streams
        let mut stream_buf = [0u8; 65535];
        for stream_id in conn.readable() {
            while let Ok((read, fin)) = conn.stream_recv(stream_id, &mut stream_buf) {
                let data = &stream_buf[..read];
                let old_data = stream_data.get(&stream_id).or(Some(&0)).unwrap();
                stream_data.insert(stream_id, old_data + data.len());
                println!(
                    "Server received all {} bytes on stream {} (fin: {})",
                    stream_data.get(&stream_id).or(Some(&0)).unwrap(),
                    stream_id,
                    fin
                );

                // Send response to the client
                let response = b"ack";
                if let Err(e) = conn.stream_send(stream_id, response, true) {
                    if e == quiche::Error::Done {
                        println!("No more data to send on stream {}", stream_id);
                    } else {
                        //eprintln!("Failed to send data on stream {}: {:?}", stream_id, e);
                    }
                } else {
                    // println!("Sent response on stream {}", stream_id);
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
}
