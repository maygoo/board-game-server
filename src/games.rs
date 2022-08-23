use std::{
    thread::{self, JoinHandle},
    time::Duration,
    net::SocketAddr,
    sync::{Arc, Mutex},
    sync::mpsc::{Receiver, Sender},
};

//pub mod tic_tac_toe;

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

    // test forwarding messages between tcp connections
    // through this central thread
    pub fn pair_players(&self) {
        let players = Arc::clone(&self.players);
        thread::spawn(move|| {
            // wait til we have 2 players
            let player1;
            let player2;

            loop {
                thread::sleep(Duration::from_secs(2));
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
                thread::sleep(Duration::from_secs(2));
                let data = players.lock().unwrap();

                // check that player 1 and player 2 are both connected
                if data.len() < 2 || data[0].addr != player1 || data[1].addr != player2 {
                    break;
                }
                
                // test piping msgs between both players
                // start with player 1
                match data[0].rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(msg) => {
                        println!("Player 1 sent: {msg}");
                        data[1].tx.send(msg).unwrap();
                    },
                    _ => (),
                }

                match data[1].rx.recv_timeout(Duration::from_millis(100)) {
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

