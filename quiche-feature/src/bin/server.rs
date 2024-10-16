use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use quiche::{Config, ConnectionId, Header, RecvInfo};
use rand::Rng;
use log::*;

fn main() {
    env_logger::init();

    println!("Server starting...");

    // Bind to UDP port 4433
    let socket = UdpSocket::bind("0.0.0.0:4433").expect("Failed to bind socket");

    // Create QUIC configuration
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();
    
    // Correctly set the application protocols
    config
        .set_application_protos(&[b"example-proto"])
        .expect("Failed to set application protocols");

    config
        .load_cert_chain_from_pem_file("cert.crt")
        .expect("Failed to load certificate");
    config
        .load_priv_key_from_pem_file("cert.key")
        .expect("Failed to load private key");
    config.set_max_idle_timeout(5000);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);
    config.verify_peer(false); // For testing purposes only

    // Use unique identifiers for connections
    let mut connections: HashMap<ConnectionId<'static>, (quiche::Connection, SocketAddr)> =
        HashMap::new();

    // Map from Connection IDs to the server's SCID (unique identifier)
    let mut connection_ids: HashMap<ConnectionId<'static>, ConnectionId<'static>> = HashMap::new();

    let mut buf = [0u8; 65535];
    let mut out = [0u8; 1350];

    loop {
        // Receive data from a client
        let (read, from) = match socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to receive data: {:?}", e);
                continue;
            }
        };

        // Parse packet header
        let hdr = match Header::from_slice(&mut buf[..read], quiche::MAX_CONN_ID_LEN) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse header: {:?}", e);
                continue;
            }
        };

        // Get the DCID from the packet
        let dcid = hdr.dcid.clone();

        println!("Received packet with DCID: {:?}", dcid);

        // Map the DCID to our internal connection ID (SCID)
        let conn_id = if let Some(orig_conn_id) = connection_ids.get(&dcid) {
            orig_conn_id.clone()
        } else {
            dcid.clone()
        };

        // Retrieve the connection
        let (conn, _) = if let Some(c) = connections.get_mut(&conn_id) {
            c
        } else {
            // New connection, generate a new scid
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
            )
                .expect("Failed to accept connection");

            println!("New connection from {}", from);

            // Store the connection using the server's SCID
            connections.insert(scid.clone(), (conn, from));

            // Map the client's initial DCID to our SCID
            connection_ids.insert(hdr.dcid.clone(), scid.clone());

            // Return the connection
            connections.get_mut(&scid).unwrap()
        };

        let recv_info = RecvInfo {
            from,
            to: socket.local_addr().unwrap(),
        };

        // Process incoming packet
        if let Err(e) = conn.recv(&mut buf[..read], recv_info) {
            eprintln!("Connection recv failed: {:?}", e);
            continue;
        }

        // Send any pending data
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket
                .send_to(&out[..write], send_info.to)
                .expect("Failed to send data");
        }

        // Read application data from readable streams
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
