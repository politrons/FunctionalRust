use std::net::UdpSocket;
use quiche::{Config, ConnectionId, RecvInfo};
use ring::rand::*;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Client starting...");

    // Crear socket UDP enlazado a un puerto efímero
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Dirección del servidor
    let server_addr = "127.0.0.1:4433".parse().unwrap();

    // Crear configuración QUIC
    let mut config = Config::new(quiche::PROTOCOL_VERSION)?;
    config.set_application_protos(&[b"example-proto"]);
    config.verify_peer(false);

    // Configurar parámetros QUIC
    config.set_max_idle_timeout(30_000); // Incrementar el timeout a 30 segundos
    config.set_max_recv_udp_payload_size(1350);
    config.set_max_send_udp_payload_size(1350);
    config.set_initial_max_data(100_000_000); // Incrementar límites de datos
    config.set_initial_max_stream_data_bidi_local(10_000_000);
    config.set_initial_max_streams_bidi(100); // Ajustar según necesidad
    config.set_disable_active_migration(true);

    // Generar un SCID aleatorio
    let mut scid = [0; quiche::MAX_CONN_ID_LEN];
    SystemRandom::new()
        .fill(&mut scid)
        .expect("Failed to generate connection ID");
    let scid = ConnectionId::from_ref(&scid);

    // Crear una nueva conexión QUIC al servidor
    let mut conn = quiche::connect(
        None,
        &scid,
        socket.local_addr().unwrap(),
        server_addr,
        &mut config,
    )?;

    // Tamaño máximo del datagrama
    const MAX_DATAGRAM_SIZE: usize = 1350;

    // Buffers para enviar y recibir datos
    let mut out = [0; MAX_DATAGRAM_SIZE];
    let mut buf = [0; 65535];

    // Enviar el paquete inicial para iniciar el handshake
    let (write, send_info) = conn.send(&mut out)?;
    socket.send_to(&out[..write], send_info.to)?;

    // Variables para rastrear el tiempo y conteos
    let mut start_time = None;
    let total_requests = 100;
    let mut requests_sent = 0;
    let mut responses_received = 0;

    // Iniciar el stream_id para streams bidireccionales iniciados por el cliente
    let mut next_stream_id = 0u64;

    loop {
        // Esperar datos del servidor
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
                // Timeout, continuar
            }
            Err(e) => {
                eprintln!("Failed to receive data: {:?}", e);
                break;
            }
        }

        // Enviar cualquier paquete pendiente generado por `recv` o datos de aplicación
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket.send_to(&out[..write], send_info.to)?;
        }

        // Una vez establecido el handshake, enviar datos de aplicación
        if conn.is_established() && requests_sent < total_requests {
            if start_time.is_none() {
                start_time = Some(Instant::now());
            }

            // Enviar solicitudes en nuevos streams
            while requests_sent < total_requests {
                // Verificar si podemos abrir un nuevo stream comparando con el límite
                let max_streams = conn.peer_streams_left_bidi();

                if max_streams == 0 {
                    // No podemos abrir más streams en este momento
                    break;
                }

                // Obtener el siguiente stream_id válido
                let stream_id = next_stream_id;
                next_stream_id += 4; // Incrementar en 4 para streams bidireccionales

                // Datos de 50 KB para enviar
                let data = vec![b'a'; 50 * 1024];

                if let Err(e) = conn.stream_send(stream_id, &data, true) {
                    if e == quiche::Error::StreamLimit {
                        // Hemos alcanzado el límite de streams
                        break;
                    } else {
                        eprintln!("Failed to send data on stream {}: {:?}", stream_id, e);
                        break;
                    }
                }
                println!("Sent {} bytes on stream {}", data.len(), stream_id);
                requests_sent += 1;
            }
        }

        // Leer respuestas del servidor
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
                    // Comprobar si se han recibido todas las respuestas
                    if responses_received == total_requests {
                        let duration = start_time.unwrap().elapsed();
                        println!("All responses received in {:?}", duration);
                        println!("Total requests sent: {}", requests_sent);
                        println!("Total responses received: {}", responses_received);
                        // Cerrar la conexión de forma segura
                        conn.close(false, 0x00, b"done")?;
                        break;
                    }
                }
            }
        }

        // Enviar paquetes pendientes después de leer respuestas
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket.send_to(&out[..write], send_info.to)?;
        }

        // Manejar cierre de conexión
        if conn.is_closed() {
            println!("Connection closed");
            break;
        }
    }

    Ok(())
}