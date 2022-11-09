use std::{
    thread::{self, JoinHandle},
    net::SocketAddr,
    sync::{Arc, Mutex},
    sync::mpsc::{Receiver, Sender, SendError, TryRecvError},
};

use super::{
    WAIT,
    ChannelBuf,
};

use common::tic_tac_toe::Message;

mod tic_tac_toe;
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
    pub fn send(player: &Player, msg: Message) -> Result<(), SendError<ChannelBuf>> {
        player.tx.send(dbg!(msg).into())?;
        Ok(())
    }

    pub fn broadcast(player1: &Player, player2: &Player, msg: Message) -> Result<(), SendError<ChannelBuf>> {
        Session::send(player1, msg.clone())?;
        Session::send(player2, msg)?;
        Ok(())
    }
}

/* pub fn try_recv<'a, T: serde::de::Deserialize<'a>>(player: &Player) -> Result<T, TryRecvError> {
    //player.rx.try_recv().and_then(|msg| Ok(bincode::deserialize(&msg).unwrap()))
    let res = player.rx.try_recv();
    match res {
        Ok(msg) => Ok(bincode::deserialize::<'a, T>(&msg).unwrap()),
        Err(_) => Err(TryRecvError::Empty),
    }
} */

pub fn try_recv(player: &Player) -> Result<Message, TryRecvError> {
    player.rx.try_recv().map(|msg| dbg!(msg.into()))
}

pub struct Lobby {
    players: Arc<Mutex<Vec<Player>>>,
}

pub struct Player {
    thread: JoinHandle<()>,
    addr: SocketAddr,
    tx: Sender<Vec<u8>>,
    rx: Receiver<Vec<u8>>,
    status: Status,
}

impl Player {
    pub fn new(thread: JoinHandle<()>, addr: SocketAddr, tx: Sender<Vec<u8>>, rx: Receiver<Vec<u8>>) -> Self {
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
                if let Some(pair) = pair {
                    // go through some process of selecting a game
                    let players2 = Arc::clone(&players);
                    thread::spawn(move|| {
                        tic_tac_toe::begin(
                            players2,
                            Session::new(pair)
                        );
                    });
                }
            }
        });
    }

    // return two addrs for both players
    fn find_pair(players: &mut [Player]) -> Option<(SocketAddr, SocketAddr)> {
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
        if players.is_empty() { println!("  None"); }
        for player in players.iter() {
            println!("  {}", player.addr);
        }
    }

    fn add_connection(players: &mut Vec<Player>, new: Player) {
        players.push(new);
    }
}

