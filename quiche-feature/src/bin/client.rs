use std::net::UdpSocket;
use quiche::{Config, ConnectionId, RecvInfo};
use ring::rand::*;
use std::time::{Duration, Instant};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Client starting...");

    // Maximum datagram size
    const MAX_DATAGRAM_SIZE: usize = 5000;
    const PACKAGE_SIZE: usize = 50;

    // Create UDP socket bound to an ephemeral port
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Server address
    let server_addr = "127.0.0.1:4433".parse().unwrap();

    // Create QUIC configuration
    let mut config = Config::new(quiche::PROTOCOL_VERSION)?;
    config.set_application_protos(&[b"example-proto"])?;
    config.verify_peer(false);

    // Configure QUIC parameters
    config.set_max_idle_timeout(30_000); // Increase the timeout to 30 seconds
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(100_000_000); // Increase data limits
    config.set_initial_max_stream_data_bidi_local(10_000_000);
    config.set_initial_max_streams_bidi(5000); // Adjust as needed
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

    // Variables to track time and counts
    let mut start_time = None;
    let total_requests = 999;
    let mut requests_sent = 0;
    let mut responses_received = 0;

    // Initialize stream_id for client-initiated bidirectional streams
    let mut next_stream_id = 0u64;

    // Keep track of streams and their data
    let mut streams_data = HashMap::new();

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
        if conn.is_established() && requests_sent < total_requests {
            if start_time.is_none() {
                start_time = Some(Instant::now());
            }

            // Send requests on new streams, respecting the concurrent stream limit
            while requests_sent < total_requests && conn.peer_streams_left_bidi() > 0 {
                // Obtain the next valid stream_id
                let stream_id = next_stream_id;
                next_stream_id += 4; // Increment by 4 for bidirectional streams

                // 50 KB of data to send
                let data = vec![b'a'; PACKAGE_SIZE * 1024];

                // Keep track of how much data has been sent on this stream
                streams_data.insert(stream_id, (data.clone(), 0));

                requests_sent += 1;
            }
        }

        // Send data on streams
        for (&stream_id, (ref data, ref mut offset)) in streams_data.iter_mut() {
            // Continue sending data on the stream until all data is sent
            while *offset < data.len() {
                match conn.stream_send(stream_id, &data[*offset..], true) {
                    Ok(written) => {
                        *offset += written;
                        println!(
                            "Sent {} bytes on stream {}, total sent: {}",
                            written, stream_id, *offset
                        );
                    }
                    Err(quiche::Error::Done) => {
                        // No more data can be sent at the moment
                        break;
                    }
                    Err(e) => {
                        eprintln!("Failed to send data on stream {}: {:?}", stream_id, e);
                        break;
                    }
                }
            }
        }

        // Remove streams that have finished sending all data
        streams_data.retain(|_, (_, offset)| *offset < 50 * 1024);

        // Send any pending packets generated by `recv` or application data
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket.send_to(&out[..write], send_info.to)?;
        }

        // Read responses from the server
        for stream_id in conn.readable() {
            while let Ok((read, fin)) = conn.stream_recv(stream_id, &mut buf) {
                let data = &buf[..read];
                println!(
                    "Client received {} bytes on stream {} (fin: {})",
                    data.len(),
                    stream_id,
                    fin
                );
                if fin {
                    responses_received += 1;
                    // Check if all responses have been received
                    if responses_received == total_requests {
                        let duration = start_time.unwrap().elapsed();
                        println!("All responses received in {:?}", duration);
                        println!("Total requests sent: {}", requests_sent);
                        println!("Total responses received: {}", responses_received);
                        // Close the connection gracefully
                        conn.close(false, 0x00, b"done")?;
                        break;
                    }
                }
            }
        }

        // Handle connection close
        if conn.is_closed() {
            println!("Connection closed");
            break;
        }
    }

    Ok(())
}
