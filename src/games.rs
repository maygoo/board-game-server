use std::{
    thread,
    time::Duration,
    sync::{Arc, Mutex},
};

pub mod tic_tac_toe;

// change to struct if we need more features later
type Player = super::Connection;

enum Game {
    TicTacToe,
}

struct Session {
    Player1: Player,
    Player2: Player,
    Game: Game,
}

impl Session {
    pub fn new(p1: Player, p2: Player, game: Game) -> Session {
        Session {
            Player1: p1,
            Player2: p2,
            Game: game,
        }
    }
}

pub fn test_connection(connections: Arc<Mutex<Vec<Player>>>) {
    thread::spawn(move|| {
        loop {
            thread::sleep(Duration::from_secs(2));
            let data = connections.lock().unwrap();
            
            if data.len() > 0 {
                let msg = data[0].rx.recv().unwrap();
                println!("other side of channel");
                println!("message received: {}", msg.trim());
                data[0].tx.send(msg.repeat(2)).unwrap();
            }
        }
    });
}