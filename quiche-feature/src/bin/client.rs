use std::net::{SocketAddr, UdpSocket};
use quiche::{Config, ConnectionId, RecvInfo};
use ring::rand::{SecureRandom, SystemRandom};

fn main() {
    println!("Start running Quiche client....");

    // Server address
    let server_addr: SocketAddr = "127.0.0.1:4433"
        .parse()
        .expect("Invalid server address");

    // Create UDP socket bound to an ephemeral port on the loopback interface
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");

    // Create QUIC configuration
    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();

    // Set the ALPN (Application-Layer Protocol Negotiation) protocols
    config
        .set_application_protos(&[b"example-proto"])
        .expect("Failed to set application protocols");

    // Disable certificate verification for testing purposes (insecure)
    config.verify_peer(false);

    // Configure various QUIC parameters
    config.set_max_idle_timeout(5000); // in milliseconds
    config.set_initial_max_data(10_000_000); // in bytes
    config.set_initial_max_stream_data_bidi_local(1_000_000); // in bytes
    config.set_initial_max_streams_bidi(100);
    config.set_disable_active_migration(true);

    // Generate a random source connection ID (SCID)
    let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
    SystemRandom::new()
        .fill(&mut scid_bytes)
        .expect("Failed to generate connection ID");
    let scid = ConnectionId::from_vec(scid_bytes.to_vec());

    // Server name (SNI) must match the server's certificate CN
    let server_name = "localhost";

    // Create a new QUIC connection to the server
    let mut conn = quiche::connect(
        Some(server_name),               // Server name (SNI)
        &scid,                           // Source Connection ID
        socket.local_addr().unwrap(),    // Client's socket address
        server_addr,                     // Server's socket address
        &mut config,
    ).expect("Failed to create connection");

    // Buffers for sending and receiving data
    let mut out = [0u8; 1350]; // Maximum transmission unit (MTU) size
    let mut buf = [0u8; 65535]; // Maximum QUIC packet size

    let mut handshake_completed = false;

    // Send the initial packet to start the handshake
    let (write, send_info) = conn
        .send(&mut out)
        .expect("Failed to create initial packet");
    socket
        .send_to(&out[..write], send_info.to)
        .expect("Failed to send initial packet");

    loop {
        // Receive data from the server
        let (len, from) = match socket.recv_from(&mut buf) {
            Ok((len, socketAddr)) => (len, socketAddr),
            Err(e) => {
                eprintln!("Failed to receive data: {:?}", e);
                continue;
            }
        };

        // Information about the received packet
        let recv_info = RecvInfo {
            from,
            to: socket.local_addr().unwrap(),
        };

        // Check Errors from receive response
        if let Err(e) = conn.recv(&mut buf[..len], recv_info) {
            eprintln!("Connection recv failed: {:?}", e);
            if conn.is_closed() {
                break;
            }
            continue;
        }

        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket
                .send_to(&out[..write], send_info.to)
                .expect("Failed to send data");
        }

        // Once the handshake is complete, send application data
        if conn.is_established() && !handshake_completed {
            handshake_completed = true;

            // Use stream ID 0 for the first bidirectional stream
            let stream_id = 0;

            // Prepare data to send (5KB of 'a's)
            let data = vec![b'a'; 5 * 1024];

            // Send data on the stream with the FIN flag set to true
            conn.stream_send(stream_id, &data, true)
                .expect("Failed to send data");

            println!("Data sent on stream {}", stream_id);
        }
        // Read data from streams that have data available
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


        // Exit the loop if the connection is closed
        if conn.is_closed() {
            println!("Connection closed");
            break;
        }
    }
}