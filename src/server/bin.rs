use std::{
    env,
    thread,
    sync::mpsc::channel,
    net::{TcpListener, SocketAddr, TcpStream},
    fs::File, io::Read,
};
use native_tls::{Identity, TlsAcceptor, TlsStream};

use common::{WAIT, ChannelBuf};

mod games;

/// Starts the board game server.
/// 
/// Default port is specified in [`common`](common::REMOTE_PORT)
/// but can be changed by passing in a cli argument
/// when running the server.
/// 
/// The server requires a valid `pkcs #12` keystore. To
/// generate one for local testing you can use the following
/// commands:
/// ```bash
/// openssl req -new -newkey rsa:4096 -x509 -nodes -out cert.crt -keyout key.pem
/// openssl pkcs12 -export -out keystore.pkcs -inkey key.pem -in cert.crt
/// ```
fn main() {
    // initialise server with default binding 0.0.0.0:3334
    const DEFAULT_IP: [u8; 4] = [0,0,0,0];
    // check command line args for port
    let port: u16 = env::args().collect::<Vec<String>>().get(1).and_then(|a| a.parse().ok()).unwrap_or(common::REMOTE_PORT);
    let addr = SocketAddr::from((DEFAULT_IP, port));

    // create shared vector for list of active connections
    let lobby = games::Lobby::new();
    // spawn thread to monitor connections, removing finished threads
    lobby.monitor();
    // start game
    lobby.begin_game();

    let mut file = File::open("tls/keystore.pkcs").expect("Needs keys");
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "").unwrap();

    match TcpListener::bind(addr) {
        Ok(listener) => {
            println!("Server listening on {}", listener.local_addr().unwrap());
            println!("promoting to tls");

            let acceptor = TlsAcceptor::new(identity).unwrap();
            //let acceptor = Arc::new(acceptor);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        match acceptor.accept(stream) {
                            Ok(stream) => handle_connection(stream, &lobby),
                            Err(e) => eprintln!("Incoming connection not using ssl. {e}")
                        };
                    },
                    Err(e) => eprintln!("Unable to connect. {e}"),
                }
            }
        },
        Err(e) => panic!("Unable to bind to {addr}. {e}"),
    };
}

fn handle_connection(stream: TlsStream<TcpStream>, lobby: &games::Lobby) {
    // convert stream to websocket
    let client = stream.get_ref().peer_addr().unwrap();
    let mut websocket = match tungstenite::accept(stream) {
        Ok(ws) => {
            println!("Connected to {client}");
            ws
        },
        Err(e) => {
            eprintln!("Error creating websocket for {client}. {e}");
            return;
        }
    };

    // set inner tcpstream to nonblocking mode
    // need to do this after the websocket handshake
    websocket.get_mut().get_mut().set_nonblocking(true).unwrap();

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
