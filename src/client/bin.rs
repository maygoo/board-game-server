use std::{
    io::{self, prelude::*, stdout},
    thread,
    sync::mpsc::{channel, Receiver},
};

use common::tic_tac_toe::{
    self,
    Message,
    ClientState,
    Piece,
    Board,
    End,
};

fn main() {
    let ip = format!("wss://{}:{}", common::REMOTE_IP, common::REMOTE_PORT);
    let ip_str = ip.to_string();

    match tungstenite::connect(ip) {
        Ok((mut socket, _)) => {
            println!("Successfully connected to {ip_str}.");

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

            // set underlying stream to nonblocking mode
            match socket.get_mut() {
                tungstenite::stream::MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true).unwrap(),
                _ => unimplemented!(),
            }

            // initialise dummy state
            let mut state = ClientState::new(String::new(), Piece::Empty, 0);
            loop {
                match socket.read_message() {
                    Ok(msg) if msg.is_binary() => {
                        let msg: Message = msg.into_data().into();

                        // better way to do this?
                        match play(msg, &mut state, &rx) {
                            Some(msg) => socket.write_message(tungstenite::Message::binary(msg)).unwrap(),
                            None => (),
                        }
                    },
                    Ok(msg) => {
                        if msg.is_close() {
                            break; // exit the thread if close msg received
                        } else if msg.is_text() {
                            println!("text msg received: {msg}");
                        }
                    },
                    Err(tungstenite::Error::Io(_)) => continue, // read timeout
                    Err(e) => {
                        println!("{e}");
                        break;
                    },
                }
            }

            println!("Connection lost");
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
                Piece::Cross => "first",
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
            state.board.place(p, x, y);
            print!("{}", state.board);
            None
        },
        Message::InvalidMove(e) => {
            println!("{e}");
            play(Message::YourTurn, state, rx)
        },
        Message::GameOver(end) => {
            match end {
                End::Disconnect => println!("Opponent has disconnected. Exiting session and returning to lobby"),
                End::Victory(p) if p == state.piece => println!("Congratualtions you have won!\nThe session will end and you will be returned to the lobby"),
                End::Victory(p) if p != state.piece => println!("You lose!\nThe session will end and you will be returned to the lobby"),
                End::Draw => println!("The game has ended in a draw! There are no winners.\nThe session will end and you will be returned to the lobby"),
                _ => unreachable!(),
            }
            None
        }
    }
}
