use std::{
    env,
    thread,
    io::prelude::*,
    net::{TcpListener, Shutdown, SocketAddr}
};

fn main() {
    // get port from heroku
    let port = env::var("PORT").ok().and_then(|port| port.parse::<u16>().ok()).unwrap_or(3333);
    let addr = [0,0,0,0];

    let listener = TcpListener::bind(SocketAddr::from((addr, port))).unwrap();
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection with {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    let mut data = [0 as u8; 50]; // 50 byte buffer
                    while match stream.read(&mut data) {
                        Ok(size) => {
                            stream.write(&data[0..size]).unwrap();
                            true
                        },
                        Err(_) => {
                            println!("An error occured. Terminating connection.");
                            stream.shutdown(Shutdown::Both).unwrap();
                            false
                        }
                    } {}
                });
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}