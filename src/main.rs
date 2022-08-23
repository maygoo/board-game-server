use std::{
    env, str,
    thread,
    io::prelude::*,
    time::Duration,
    sync::mpsc::channel,
    net::{TcpListener, Shutdown, SocketAddr, TcpStream},
};

mod games;

fn main() {
    // initialise server with default binding 0.0.0.0:3334
    const DEFAULT_IP: [u8; 4] = [0,0,0,0];
    const DEFAULT_PORT: u16 = 3334;
    // check command line args for port
    let port: u16 = env::args().collect::<Vec<String>>().get(1).and_then(|a| a.parse().ok()).unwrap_or(DEFAULT_PORT);
    let addr = SocketAddr::from((DEFAULT_IP, port));

    // create shared vector for list of active connections
    let lobby = games::Lobby::new();
    // spawn thread to monitor connections, removing finished threads
    lobby.monitor();

    // test channel comms
    lobby.pair_players();

    match TcpListener::bind(addr) {
        Ok(listener) => {
            println!("Server listening on {}", listener.local_addr().unwrap());
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => handle_connection(stream, &lobby),
                    Err(e) => eprintln!("Unable to connect. {e}"),
                }
            }
        },
        Err(e) => panic!("Unable to bind to {addr}. {e}"),
    };
}

fn handle_connection(mut stream: TcpStream, lobby: &games::Lobby) {
    let client = stream.peer_addr().unwrap();
    println!("Connected to {client}");

    // create channel pair for duplex communication
    let (tx_t, rx) = channel::<String>();
    let (tx, rx_t) = channel::<String>();

    let t = thread::spawn(move|| {
        let mut data = [0 as u8; 50]; // 50 byte buffer
        
        // set a timeout on read so that read is nonblocking
        // i.e. we can send without needing to read
        stream.set_read_timeout(Some(Duration::from_millis(10))).unwrap_or_default();
        while match stream.read(&mut data) {
            Ok(size) if size > 0 => {
                let recv = String::from(str::from_utf8(&data).unwrap());
                // send received data through channel to game controller
                tx_t.send(recv).unwrap();
                true
            },
            Ok(_) => false, // connection is closed for sizes == 0
            Err(_) => true, // continue because errors are timeout errors
        } {
            // receive data through channel from game controller
            match rx_t.try_recv() {
                Ok(recv) => {
                    stream.write(recv.as_bytes()).unwrap();
                },
                _ => (),
            }
        }

        stream.shutdown(Shutdown::Both).unwrap();
    });

    lobby.add_and_print_connections(games::Player {
        thread: t,
        addr: client,
        tx,
        rx,
    });
}
