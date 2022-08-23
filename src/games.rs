use std::{
    thread::{self, JoinHandle},
    net::SocketAddr,
    sync::{Arc, Mutex},
    sync::mpsc::{Receiver, Sender},
};

use crate::WAIT;

pub mod tic_tac_toe;

#[derive(Debug)]
enum Game {
    TicTacToe,
}

pub struct Session {
    player1: SocketAddr,
    player2: SocketAddr,
    game: Option<Game>,
}

impl Session {
    pub fn new(player1: SocketAddr, player2: SocketAddr) -> Self {
        Session {
            player1,
            player2,
            game: None,
        }
    }
}

pub struct Lobby {
    players: Arc<Mutex<Vec<Player>>>,
}

pub struct Player {
    thread: JoinHandle<()>,
    addr: SocketAddr,
    tx: Sender<String>,
    rx: Receiver<String>,
    status: Status,
}

impl Player {
    pub fn new(thread: JoinHandle<()>, addr: SocketAddr, tx: Sender<String>, rx: Receiver<String>) -> Self {
        Player {
            thread,
            addr,
            tx,
            rx,
            status: Status::Waiting,
        }
    }
}

#[derive(PartialEq)]
enum Status {
    Waiting,
    Playing,
}

impl Lobby {
    pub fn new() -> Self {
        Lobby {
            players:Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn begin_game(&self) {
        let players = Arc::clone(&self.players);

        thread::spawn(move|| {
            let mut pair;
            while {
                let mut data = players.lock().unwrap();
                pair = Lobby::find_pair(&mut data);
                pair.is_none()
            } {
                thread::sleep(WAIT);
            }
            let (player1, player2) = pair.unwrap();

            let session = Session::new(player1, player2);
            tic_tac_toe::begin(players, session);
        });
    }

    // return two addrs for both players
    fn find_pair(players: &mut Vec<Player>) -> Option<(SocketAddr, SocketAddr)> {
        let mut waiting: Vec<&mut Player> = players.iter_mut().filter(|player| player.status == Status::Waiting).collect();

        if waiting.len() < 2 { 
            None
        } else {
            waiting[0].status = Status::Playing;
            waiting[1].status = Status::Playing;
            
            Some((waiting[0].addr, waiting[1].addr))
        }
    }

    // test forwarding messages between tcp connections
    // through this central thread
    pub fn _pair_players(&self) {
        let players = Arc::clone(&self.players);
        thread::spawn(move|| {
            // wait til we have 2 players
            let player1;
            let player2;

            loop {
                thread::sleep(WAIT);
                let data = players.lock().unwrap();
                if data.len() > 1 {
                    player1 = data[0].addr;
                    player2 = data[1].addr;

                    println!("Player 1 selected: {}", player1);
                    println!("Player 2 selected: {}", player2);
                    break;
                }
            }

            loop {
                let data = players.lock().unwrap();

                // check that player 1 and player 2 are both connected
                if data.len() < 2 || data[0].addr != player1 || data[1].addr != player2 {
                    break;
                }
                
                // test piping msgs between both players
                // start with player 1
                match data[0].rx.try_recv() {
                    Ok(msg) => {
                        println!("Player 1 sent: {msg}");
                        data[1].tx.send(msg).unwrap();
                    },
                    _ => (),
                }

                match data[1].rx.try_recv() {
                    Ok(msg) => {
                        println!("Player 2 sent: {msg}");
                        data[0].tx.send(msg).unwrap();
                    },
                    _ => (),
                }
            }

            println!("Pair broken.");
        });
    }

    // maintains a list of active connections
    // by spawning a new thread and removing
    // finished TcpStream threads
    pub fn monitor(&self) {
        let players = Arc::clone(&self.players);
        thread::spawn(move|| {
            loop {
                thread::sleep(WAIT);
                let mut data = players.lock().unwrap();
                
                // print active connections only if connections change
                let initial_len = data.len();
                data.retain(|player| !player.thread.is_finished());
                if data.len() != initial_len { Lobby::print_connections(&data); }
            }
        });
    }

    pub fn add_and_print_connections(&self, new: Player) {
        let mut data = self.players.lock().unwrap();
        Lobby::print_connections(&data);
        println!("  {}  <--  new", new.addr);
        Lobby::add_connection(&mut data, new)
    }

    fn print_connections(players: &Vec<Player>) {
        println!("Active players:");
        for player in players.iter() {
            println!("  {}", player.addr);
        }
    }

    fn add_connection(players: &mut Vec<Player>, new: Player) {
        players.push(new);
    }
}

