use std::{
    env, thread,
    io::prelude::*,
    net::{TcpListener, TcpStream, Shutdown},
    str::from_utf8,
};

// to get this running across the internet:
//      port forward the corresponding port to the machine ip
//      change listening ip for server to the machine ip (e.g. 192.168.1.24)
//      change the sending ip for client to the global ip

fn main() {
    let args = env::args().collect::<Vec<String>>();

    match &args {
        i if i.len() > 1 && i[1] == "server" => start_server(),
        i if i.len() > 1 && i[1] == "client" => handle_server(),
        _ => panic!("wrong args. needs server or client"),
    }
}

fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:3333").unwrap();
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

fn handle_server() {
    match TcpStream::connect("0.0.0.0:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server on port 3333");

            let msg = b"Hello!";

            stream.write(msg).unwrap();
            
            let mut data = [0 as u8; 6];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    if &data == msg {
                        println!("Reply is ok!");
                    } else {
                        let text = from_utf8(&data).unwrap();
                        println!("Expected reply: {}", text);
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}