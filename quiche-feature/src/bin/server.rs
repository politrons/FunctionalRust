use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use quiche::{Config, ConnectionId, Header, RecvInfo};
use rand::Rng;
use log::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("Server starting...");

    // Enlazar al puerto UDP 4433
    let socket = UdpSocket::bind("0.0.0.0:4433")?;

    // Crear configuración QUIC
    let mut config = Config::new(quiche::PROTOCOL_VERSION)?;

    // Establecer protocolos ALPN
    config.set_application_protos(&[b"example-proto"]);


    // Cargar certificado y clave privada del servidor
    config.load_cert_chain_from_pem_file("cert.crt")?;
    config.load_priv_key_from_pem_file("cert.key")?;

    // Configurar parámetros QUIC
    config.set_max_idle_timeout(30_000); // Incrementar el timeout a 30 segundos
    config.set_initial_max_data(100_000_000); // Incrementar límites de datos
    config.set_initial_max_stream_data_bidi_remote(10_000_000);
    config.set_initial_max_streams_bidi(10_000);
    config.set_disable_active_migration(true);

    // Deshabilitar verificación de par (solo para pruebas)
    config.verify_peer(false);

    // Mapas para almacenar conexiones y IDs
    let mut connections: HashMap<ConnectionId<'static>, (quiche::Connection, SocketAddr)> =
        HashMap::new();
    let mut connection_ids: HashMap<ConnectionId<'static>, ConnectionId<'static>> = HashMap::new();

    // Buffers para enviar y recibir datos
    let mut buf = [0u8; 65535];
    let mut out = [0u8; 1350];

    loop {
        // Recibir datos de un cliente
        let (read, from) = match socket.recv_from(&mut buf) {
            Ok((len, addr)) => (len, addr),
            Err(e) => {
                eprintln!("Failed to receive data: {:?}", e);
                continue;
            }
        };

        // Parsear el encabezado del paquete QUIC
        let hdr = match Header::from_slice(&mut buf[..read], quiche::MAX_CONN_ID_LEN) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse header: {:?}", e);
                continue;
            }
        };

        // Obtener DCID del encabezado
        let dcid = hdr.dcid.clone();

        // Mapear DCID a SCID interno
        let conn_id = if let Some(scid) = connection_ids.get(&dcid) {
            scid.clone()
        } else {
            dcid.clone()
        };

        // Obtener la conexión asociada
        let (conn, _) = if let Some(conn_socket_addr) = connections.get_mut(&conn_id) {
            conn_socket_addr
        } else {
            println!("Received packet pre-handshake with DCID: {:?}", dcid);

            // Generar un nuevo SCID para el servidor
            let mut scid_bytes = [0u8; quiche::MAX_CONN_ID_LEN];
            rand::thread_rng().fill(&mut scid_bytes);
            let scid = ConnectionId::from_vec(scid_bytes.to_vec());

            // Aceptar la nueva conexión
            let conn = quiche::accept(
                &scid,
                Some(&hdr.dcid),
                socket.local_addr().unwrap(),
                from,
                &mut config,
            )?;

            println!("New connection from {}", from);

            // Almacenar la conexión
            connections.insert(scid.clone(), (conn, from));
            connection_ids.insert(hdr.dcid.clone(), scid.clone());

            // Obtener la conexión
            connections.get_mut(&scid).unwrap()
        };

        // Información sobre el paquete recibido
        let recv_info = RecvInfo {
            from,
            to: socket.local_addr().unwrap(),
        };

        // Procesar el paquete recibido
        if let Err(e) = conn.recv(&mut buf[..read], recv_info) {
            eprintln!("Connection recv failed: {:?}", e);
            continue;
        }

        // Enviar cualquier paquete pendiente
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket
                .send_to(&out[..write], send_info.to)
                .expect("Failed to send data");
        }

        // Leer datos de streams disponibles
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

                // Enviar respuesta al cliente
                let response = b"Response from server";
                if let Err(e) = conn.stream_send(stream_id, response, true) {
                    if e == quiche::Error::Done {
                        println!("No more data to send on stream {}", stream_id);
                    } else {
                        eprintln!("Failed to send data on stream {}: {:?}", stream_id, e);
                    }
                } else {
                    println!("Sent response on stream {}", stream_id);
                }
            }
        }

        // Enviar paquetes pendientes después de manejar streams
        while let Ok((write, send_info)) = conn.send(&mut out) {
            socket
                .send_to(&out[..write], send_info.to)
                .expect("Failed to send data");
        }

        // Manejar cierre de conexión
        if conn.is_closed() {
            println!("Connection closed with {}", from);
            connections.remove(&conn_id);

            // Eliminar mapeos de esta conexión
            connection_ids.retain(|_, v| v != &conn_id);
        }
    }
}
