use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use quiche::{Config, ConnectionId, Header, RecvInfo};
use rand::Rng;
use log::*;

fn main() {
    env_logger::init();

    println!("Server starting...");

    // Bind to UDP port 4433 (standard port for QUIC over TLS)
    let socket = UdpSocket::bind("0.0.0.0:4433").expect("Failed to bind socket");

    // Create QUIC configuration
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();

    // Set the ALPN protocols
    config
        .set_application_protos(&[b"example-proto"])
        .expect("Failed to set application protocols");

    // Load server's TLS certificate and private key
    config
        .load_cert_chain_from_pem_file("cert.crt")
        .expect("Failed to load certificate");
    config
        .load_priv_key_from_pem_file("cert.key")
        .expect("Failed to load private key");

    // Configure various QUIC parameters
    config.set_max_idle_timeout(5000);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);

    // Disable peer verification (for testing; in practice, the server doesn't verify the client)
    config.verify_peer(false);

    // HashMap to store connections using the server's SCID as the key
    let mut connections: HashMap<ConnectionId<'static>, (quiche::Connection, SocketAddr)> =
        HashMap::new();

    // HashMap to map client's DCIDs to the server's SCIDs
    let mut connection_ids: HashMap<ConnectionId<'static>, ConnectionId<'static>> = HashMap::new();

    // Buffers for sending and receiving data
    let mut buf = [0u8; 65535];
    let mut out = [0u8; 1350];

    loop {
        // Receive data from a client
        let (read, from) = match socket.recv_from(&mut buf) {
            Ok((len,socketAddr)) => (len,socketAddr),
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

        // Get the DCID from the packet header
        let dcid = hdr.dcid.clone();


        // Map the DCID to our internal connection ID (SCID)
        let conn_id = if let Some(scid) = connection_ids.get(&dcid) {
            scid.clone()
        } else {
            dcid.clone()
        };

        // Retrieve the connection associated with the conn_id
        let (conn, _) = if let Some(connSocketAddr) = connections.get_mut(&conn_id) {
            connSocketAddr
        } else {
            println!("Received packet Pre Handshake with DCID: {:?}", dcid);
            // If no existing connection, accept a new one
            // Generate a new SCID for the server
            let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
            rand::thread_rng().fill(&mut scid_bytes);
            let scid = ConnectionId::from_vec(scid_bytes.to_vec());

            // Accept the new connection
            let conn = quiche::accept(
                &scid,                      // Server's SCID
                Some(&hdr.dcid),            // Client's DCID
                socket.local_addr().unwrap(), // Server's socket address
                from,                       // Client's socket address
                &mut config,
            ).expect("Failed to accept connection");

            println!("New connection from {}", from);

            // Store the connection using the server's SCID
            connections.insert(scid.clone(), (conn, from));

            // Map the client's initial DCID to the server's SCID
            connection_ids.insert(hdr.dcid.clone(), scid.clone());

            // Retrieve the connection
            connections.get_mut(&scid).unwrap()
        };

        // Information about the received packet
        let recv_info = RecvInfo {
            from,
            to: socket.local_addr().unwrap(),
        };

        // Check error from the received packet
        if let Err(e) = conn.recv(&mut buf[..read], recv_info) {
            eprintln!("Connection recv failed: {:?}", e);
            continue;
        }

        // Send any pending handshake messages data
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket
                .send_to(&out[..write], send_info.to)
                .expect("Failed to send data");
        }

        // Read data from streams that have data available
        let mut stream_buf = [0u8; 65535];
        for stream_id in conn.readable() {
            while let Ok((read, fin)) = conn.stream_recv(stream_id, &mut stream_buf) {
                let data = &stream_buf[..read];
                println!(
                    "Server received {} bytes on stream {} (fin: {})",
                    data.len(),
                    stream_id,
                    fin
                );

                // Echo the data back to the client
                conn.stream_send(stream_id, data, fin)
                    .expect("Failed to send data");
            }
        }

        // Clean up closed connections
        if conn.is_closed() {
            connections.remove(&conn_id);

            // Remove all mappings for this connection
            connection_ids.retain(|_, v| v != &conn_id);
        }
    }
}