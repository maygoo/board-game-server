use std::{
    env,
    thread,
    time::Duration,
    sync::mpsc::channel,
    net::{TcpListener, SocketAddr, TcpStream},
};

mod games;

type ChannelBuf = Vec<u8>;

const WAIT: Duration = Duration::from_millis(100);

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
    // start game
    lobby.begin_game();

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

fn handle_connection(stream: TcpStream, lobby: &games::Lobby) {
    // convert stream to websocket
    let client = stream.peer_addr().unwrap();
    let mut websocket = match tungstenite::accept(stream) {
        Ok(ws) => {
            println!("Connected to {client}");
            ws
        },
        Err(e) => {
            panic!("Error creating websocket for {client}. {e}");
        }
    };

    // set underlying stream to nonblocking mode
    // need to do this after the websocket handshake
    websocket.get_mut().set_nonblocking(true).unwrap();

    // create channel pair for duplex communication
    let (tx_t, rx) = channel::<ChannelBuf>();
    let (tx, rx_t) = channel::<ChannelBuf>();

    let t = thread::spawn(move|| {     
        //client.set_nonblocking(true).unwrap();

        loop {
            match websocket.read_message() {
                Ok(msg) if msg.is_binary() => {
                    // send the data to through channel to the game controller
                    tx_t.send(msg.into_data()).unwrap();
                },
                Ok(msg) => {
                    if msg.is_text() {
                        // not expecting text messages
                        // print them out
                        println!("Text msg received from {client}: {:?}", msg.to_text());
                    } else if msg.is_close() {
                        break;
                    }
                },
                Err(tungstenite::Error::Io(_)) => (), // read timeout,
                Err(e) => {
                    // break on other errors
                    println!("Error: {e}");
                    break;
                },
            }

            // receive data through channel from game controller
            match rx_t.try_recv() {
                Ok(send) => {
                    websocket.write_message(tungstenite::Message::binary(send)).unwrap();
                },
                _ => (),
            }
        }
    });

    lobby.add_and_print_connections(games::Player::new(t, client, tx, rx));
}
