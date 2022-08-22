use std::{
    env, str, fmt,
    thread::{self, JoinHandle},
    io::prelude::*,
    time::Duration,
    sync::{Arc, Mutex},
    net::{TcpListener, Shutdown, SocketAddr, TcpStream},
};

struct Connection {
    handle: JoinHandle<()>,
    addr: SocketAddr,
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.addr)
    }
}

fn main() {
    // initialise server with default binding 0.0.0.0:3334
    const DEFAULT_IP: [u8; 4] = [0,0,0,0];
    const DEFAULT_PORT: u16 = 3334;
    // check command line args for port
    let port: u16 = env::args().collect::<Vec<String>>().get(1).and_then(|a| a.parse().ok()).unwrap_or(DEFAULT_PORT);
    let addr = SocketAddr::from((DEFAULT_IP, port));

    // create shared vector for list of active connections
    let connections: Arc<Mutex<Vec<Connection>>> = Arc::new(Mutex::new(Vec::new()));
    // spawn thread to monitor connections, removing finished threads
    monitor_connections(Arc::clone(&connections));

    match TcpListener::bind(addr) {
        Ok(listener) => {
            println!("Server listening on {}", listener.local_addr().unwrap());
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => handle_connection(stream, Arc::clone(&connections)),
                    Err(e) => eprintln!("Unable to connect. {e}"),
                }
            }
        },
        Err(e) => panic!("Unable to bind to {addr}. {e}"),
    };
}

fn handle_connection(mut stream: TcpStream, connections: Arc<Mutex<Vec<Connection>>>) {
    let client = stream.peer_addr().unwrap();
    println!("Connected to {client}");

    // set timeout for thread to close if no
    // message read recently
    const TIMEOUT: Duration = Duration::from_secs(20);
    stream.set_read_timeout(Some(TIMEOUT)).unwrap_or_default();

    let t = thread::spawn(move|| {
        let mut data = [0 as u8; 50]; // 50 byte buffer
        while match stream.read(&mut data) {
            Ok(size) => {
                let len = data.iter().filter(|v| **v != 0).count();
                if len > 0 {
                    println!("received msg: {:?}", str::from_utf8(&data));
                    match stream.write(&data[0..size]) {
                        Ok(_) => println!("Successfuly echoed message."),
                        Err(e) => println!("Error echoing message. {e}"),
                    };
                    data = [0 as u8; 50];
                }
                true
            },
            Err(e) => {
                println!("{e}.");
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
    });

    add_and_print_connections(connections, Connection { handle: t, addr: client });
}

// maintains a list of active connections
// by spawning a new thread and removing
// finished TcpStream threads
fn monitor_connections(connections: Arc<Mutex<Vec<Connection>>>) {
    thread::spawn(move|| {
        loop {
            thread::sleep(Duration::from_secs(2));
            let mut data = connections.lock().unwrap();
            
            data.retain(|connection| !connection.handle.is_finished());
            print_connections(&data);
        }
    });
}

fn print_connections(connections: &Vec<Connection>) {
    println!("Active connections:");
    for connection in connections.iter() {
        println!("  {}", connection);
    }
}

fn add_connection(connections: &mut Vec<Connection>, new: Connection) {
    connections.push(new);
}

fn add_and_print_connections(connections: Arc<Mutex<Vec<Connection>>>, new: Connection) {
    let mut data = connections.lock().unwrap();
    print_connections(&data);
    println!("  {}  <--  new", new);
    add_connection(&mut data, new)
}