use std::{
    thread,
    sync::{Arc, Mutex},
};

use crate::WAIT;

pub fn begin(players: Arc<Mutex<Vec<super::Player>>>, mut session: super::Session) {
    thread::spawn(move|| {
        session.game = Some(super::Game::TicTacToe);
        let game = session.game.as_ref().unwrap();

        println!("Started {:?} with {} and {}", game, session.player1, session.player2);
        
        loop {
            thread::sleep(WAIT);
            let data = players.lock().unwrap();
            match data.iter().filter(|p| p.addr == session.player1 || p.addr == session.player2).collect::<Vec<&super::Player>>()[..] {
                [player1, player2] => {
                    println!("Found {} and {}", player1.addr, player2.addr);
                    player1.tx.send(format!("You have been matched!\nPlaying {:?} with {}", game, player2.addr)).unwrap();
                    player2.tx.send(format!("You have been matched!\nPlaying {:?} with {}", game, player1.addr)).unwrap();
                    break;
                },
                _ => {
                    println!("Not enough players...");
                    break;
                }
            }
        }
    });
}