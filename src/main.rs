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
    lobby.test_channel();

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

    // set timeout for thread to close if no
    // message read recently
    const TIMEOUT: Duration = Duration::from_secs(60 * 10);
    stream.set_read_timeout(Some(TIMEOUT)).unwrap_or_default();

    // create channel pair for duplex communication
    let (tx_t, rx) = channel::<String>();
    let (tx, rx_t) = channel::<String>();

    let t = thread::spawn(move|| {
        let mut data = [0 as u8; 50]; // 50 byte buffer
        while match stream.read(&mut data) {
            Ok(_) => {
                println!("{:?}",&data);
                let recv = String::from(str::from_utf8(&data).unwrap());
                //println!("Received: {}", &recv);
                tx_t.send(recv).unwrap();
                let recv = rx_t.recv().unwrap();
                println!("test receive end of channel with size {}: {:?}", recv.len() ,recv.as_bytes());
                match stream.write(recv.as_bytes()) {
                    Ok(size) => println!("Successfuly echoed message with size {size}."),
                    Err(e) => println!("Error echoing message. {e}"),
                };
                true
            },
            Err(e) => {
                println!("{e}.");
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
    });

    lobby.add_and_print_connections(games::Player {
        thread: t,
        addr: client,
        tx,
        rx,
    });
}
