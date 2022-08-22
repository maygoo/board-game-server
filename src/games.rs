use std::{
    thread::{self, JoinHandle},
    time::Duration,
    net::SocketAddr,
    sync::{Arc, Mutex},
    sync::mpsc::{Receiver, Sender},
};

pub mod tic_tac_toe;

enum Game {
    TicTacToe,
}

struct Session {
    Player1: Player,
    Player2: Player,
    Game: Game,
}

impl Session {
    pub fn new(p1: Player, p2: Player, game: Game) -> Self {
        Session {
            Player1: p1,
            Player2: p2,
            Game: game,
        }
    }
}

pub fn test_connection(connections: Arc<Mutex<Vec<Player>>>) {

}

pub struct Lobby {
    players: Arc<Mutex<Vec<Player>>>,
}

pub struct Player {
    pub thread: JoinHandle<()>,
    pub addr: SocketAddr,
    pub tx: Sender<String>,
    pub rx: Receiver<String>,
}

impl Lobby {
    pub fn new() -> Self {
        Lobby {
            players:Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn test_channel(&self) {
        let players = Arc::clone(&self.players);
        thread::spawn(move|| {
            loop {
                thread::sleep(Duration::from_secs(2));
                let data = players.lock().unwrap();
                
                if data.len() > 0 {
                    let msg = data[0].rx.recv().unwrap();
                    println!("other side of channel");
                    println!("message received: {}", msg.trim());
                    data[0].tx.send(msg.repeat(2)).unwrap();
                }
            }
        });
    }

    // maintains a list of active connections
    // by spawning a new thread and removing
    // finished TcpStream threads
    pub fn monitor(&self) {
        let players = Arc::clone(&self.players);
        thread::spawn(move|| {
            loop {
                thread::sleep(Duration::from_secs(2));
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

