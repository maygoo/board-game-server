use std::{
    thread::{self, JoinHandle},
    net::SocketAddr,
    sync::{Arc, Mutex},
    sync::mpsc::{Receiver, Sender, SendError},
};

use crate::WAIT;

use self::tic_tac_toe::Message;

pub mod tic_tac_toe;

enum Game {
    TicTacToe,
}

pub struct Session {
    player1: SocketAddr,
    player2: SocketAddr,
    game: Option<Game>,
}

impl Session {
    pub fn new((player1, player2): (SocketAddr, SocketAddr)) -> Self {
        Session {
            player1,
            player2,
            game: None,
        }
    }

    // TODO move these to player
    pub fn send(player: &Player, msg: Message) -> Result<(), SendError<Message>> {
        player.tx.send(msg)?;
        Ok(())
    }

    pub fn broadcast(player1: &Player, player2: &Player, msg: Message) -> Result<(), SendError<Message>> {
        Session::send(player1, msg.clone())?;
        Session::send(player2, msg.clone())?;
        Ok(())
    }
}

pub struct Lobby {
    players: Arc<Mutex<Vec<Player>>>,
}

pub struct Player {
    thread: JoinHandle<()>,
    addr: SocketAddr,
    tx: Sender<tic_tac_toe::Message>,
    rx: Receiver<tic_tac_toe::Message>,
    status: Status,
}

impl Player {
    pub fn new(thread: JoinHandle<()>, addr: SocketAddr, tx: Sender<tic_tac_toe::Message>, rx: Receiver<tic_tac_toe::Message>) -> Self {
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
            loop {
                thread::sleep(WAIT);
                let mut data = players.lock().unwrap();
                let pair = Lobby::find_pair(&mut data);
                if pair.is_some() {
                    // go through some process of selecting a game
                    tic_tac_toe::begin(
                        Arc::clone(&players), 
                        Session::new(pair.unwrap())
                    );
                }
            }
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

