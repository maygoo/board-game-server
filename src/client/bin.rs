use std::{
    io::{self, prelude::*, stdout},
    net::{TcpStream, Shutdown},
    str,
    thread,
    time::Duration,
    sync::mpsc::{channel, Receiver},
};

use common::tic_tac_toe::{
    self,
    Message,
    ClientState,
    Piece,
    Board,
};

fn main() {
    // connect to the server
    #[cfg(debug_assertions)]
    const REMOTE_IP: &str = "127.0.0.1:3334";
    #[cfg(not(debug_assertions))]
    const REMOTE_IP: &str = "ec2-3-25-98-214.ap-southeast-2.compute.amazonaws.com:3334";

    match TcpStream::connect(REMOTE_IP) {
        Ok(mut stream) => {
            println!("Successfully connected to {}.", &stream.peer_addr().unwrap());

            // create two threads, one to block on io reading from stdin
            // the other to handle the TcpStream and sending/receiving
            // to the server

            // thread to read input from stdin
            // and forward to the second thread
            let (tx, rx) = channel::<String>();
            thread::spawn(move|| {
                loop {
                    let mut send = String::new();
                    io::stdin().read_line(&mut send).unwrap();
                    tx.send(send).unwrap();
                }
            });

            // thread to handle the tcp connection
            stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
            // initialise dummy state
            let mut state = ClientState::new(String::new(), Piece::Empty, 0);
            loop {
                let mut recv = [0 as u8; common::BUFF_SIZE];
                match stream.read(&mut recv) {
                    Ok(size) if size > 0 => {
                        let msg: Message = bincode::deserialize(&recv).unwrap();

                        // better way to do this?
                        match play(msg, &mut state, &rx) {
                            Some(msg) => { stream.write(&bincode::serialize(&msg).unwrap()).unwrap(); }
                            None => (),
                        }
                    },
                    Ok(_) => break, // connection is closed if size == 0
                    _ => (),
                }
            }

            stream.shutdown(Shutdown::Both).unwrap();
            println!("Connection to {} closed.", &stream.peer_addr().unwrap());
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}

fn play(msg: Message, state: &mut ClientState, rx: &Receiver<String>) -> Option<Message> {
    match msg {
        Message::Preamble(config) => {
            *state = config;
            state.board = Board::new(state.board.size);

            let order = match state.piece {
                Piece::Cross => "fisrt",
                Piece::Nought => "second",
                Piece::Empty => unreachable!("Player cannot be assigned the empty piece"),
            };

            println!("=====================");
            println!("{}", tic_tac_toe::NAME);
            println!("Playing with {}", state.opponent);
            println!("=====================\n");
            println!("Instructions{}", tic_tac_toe::INSTRUCTIONS);
            println!("You are player {}. You go {}.\n", state.piece, order);
            print!("{}", state.board);

            None
        },
        Message::WaitTurn => {
            println!("Please wait for your opponent to move");
            None
        },
        Message::YourTurn => {
            print!("Enter your move (two coordinates, e.g. a 2)\n{}: ", state.piece);
            stdout().flush().unwrap();

            // receive user input from the first thread
            loop {
                // block while waiting, otherwise try_recv()
                let input = rx.recv().unwrap();

                let parse = input
                    .split_ascii_whitespace()
                    .map(|s| s.chars().collect())
                    .collect::<Vec<Vec<char>>>();

                let offset_lower = 97usize; // ascii value for 'a'
                let offset_upper = 65usize; // ascii value for 'A'

                if parse.len() == 2 {
                    // assume lhs is a single char
                    let y: u32 = parse[0][0].into(); // get ord of char
                    let y = y as usize; // for the following comparisons
                    match parse[1].iter().collect::<String>().parse::<usize>() {
                        //  validate y coord in match guard
                        Ok(x) if x > 0 && x <= state.board.size => {
                            // less the offset for the decimal value, e.g. a:1, b:2, etc
                            // less 1 from y to account for zero-indexed board
                            if y >= offset_lower && y <= offset_lower + state.board.size {
                                return Some(Message::Move((state.piece.clone(), x-1, y-offset_lower)));
                            } else if y >= offset_upper && x <= offset_upper + state.board.size {
                                return Some(Message::Move((state.piece.clone(), x-1, y-offset_upper)));
                            }
                        },
                        _ => (),
                    };
                }

                println!("Invalid input. Please enter valid cell coordinates");
            }
        },
        Message::Move((p, x, y)) => {
            // update board state
            // move has already been validated by server
            state.board.place(x, y, p);
            print!("{}", state.board);
            None
        },
        Message::InvalidMove(e) => {
            println!("{e}");
            play(Message::YourTurn, state, rx)
        },
        Message::GameOver(piece) => {
            match piece {
                Piece::Empty => println!("Opponent has disconnected. Exiting session and returning to lobby"),
                p if p == state.piece => println!("Congratualtions you have won!\nThe session will end and you will be returned to the lobby"),
                p if p != state.piece => println!("You lose!\nThe session will end and you will be returned to the lobby"),
                _ => unreachable!(),
            }
            None
        }
    }
}
