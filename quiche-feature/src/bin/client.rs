use std::net::{SocketAddr, UdpSocket};
use quiche::{Config, ConnectionId, RecvInfo, PROTOCOL_VERSION};

fn main() {
    println!("Start running Quiche client....");
    // Server address
    let server_addr: SocketAddr = "127.0.0.1:4433".parse().expect("Invalid server address");

    // Create UDP socket bound to loopback interface with ephemeral port
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");

    // Create QUIC configuration
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();
    config.set_application_protos(&[b"example-proto"])
        .expect("Failed to set application protocols");
    config.verify_peer(false);
    config.set_max_idle_timeout(5000);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);

    // Generate source connection ID
    let scid = ConnectionId::from_ref(&[0xde, 0xad, 0xbe, 0xef]);

    // Create QUIC connection
    let mut conn = quiche::connect(
        None,                         // Server name (SNI)
        &scid,                        // Source connection ID
        socket.local_addr().unwrap(), // Local address
        server_addr,                  // Server address
        &mut config,
    )
        .expect("Failed to create connection");

    let mut out = [0u8; 1350];
    let mut buf = [0u8; 65535];

    let mut handshake_completed = false;

    // Initial packet
    let (write, send_info) = conn.send(&mut out).expect("Failed to create initial packet");
    socket
        .send_to(&out[..write], send_info.to)
        .expect("Failed to send initial packet");

    loop {
        // Receive data from server
        let (len, from) = match socket.recv_from(&mut buf) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to receive data: {:?}", e);
                continue;
            }
        };

        let recv_info = RecvInfo {
            from,
            to: socket.local_addr().unwrap(),
        };

        if let Err(e) = conn.recv(&mut buf[..len], recv_info) {
            eprintln!("Connection recv failed: {:?}", e);
            if conn.is_closed() {
                break;
            }
            continue;
        }

        // Send any pending data
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket
                .send_to(&out[..write], send_info.to)
                .expect("Failed to send data");
        }

        // Send data once the handshake is completed
        if conn.is_established() && !handshake_completed {
            handshake_completed = true;

            // Use stream ID 0 for the first bidirectional stream
            let stream_id = 0;

            // Send data on the stream
            let data = vec![b'a'; 5 * 1024]; // 5KB of data
            conn.stream_send(stream_id, &data, true)
                .expect("Failed to send data");

            println!("Data sent on stream {}", stream_id);
        }

        // Read server's response from readable streams
        let mut stream_buf = [0u8; 65535];
        for stream_id in conn.readable() {
            while let Ok((read, fin)) = conn.stream_recv(stream_id, &mut stream_buf) {
                let data = &stream_buf[..read];
                println!(
                    "Client received {} bytes on stream {} (fin: {})",
                    data.len(),
                    stream_id,
                    fin
                );
            }
        }

        if conn.is_closed() {
            println!("Connection closed");
            break;
        }
    }
}
