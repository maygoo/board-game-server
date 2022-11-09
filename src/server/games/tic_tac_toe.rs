use std::{
    thread,
    sync::{Arc, Mutex},
};

use crate::games::{Player, Session};

use common::THREAD_SLEEP;
use common::tic_tac_toe::{
    self,
    Message,
    ServerState,
    Piece,
    Turn,
    ClientState,
    End,
};

pub fn begin(players: Arc<Mutex<Vec<Player>>>, mut session: super::Session) {
    session.game = Some(super::Game::TicTacToe);

    let mut state = ServerState::new(tic_tac_toe::BOARD_SIZE);
    
    println!("Started {:?} with {} and {}", tic_tac_toe::NAME, session.player1, session.player2);
    
    loop {
        thread::sleep(THREAD_SLEEP);
        let mut data = players.lock().unwrap();
        // check that both players are still connected
        let mut players: Vec<&mut Player> = data
            .iter_mut()
            .filter(|p| p.addr == session.player1 || p.addr == session.player2)
            .collect();
        
        match players.len() {
            2 => {
                let current_player;
                let next_player;
                if state.current_player == Piece::Cross {
                    current_player = &players[state.crosses_player];
                    next_player = &players[state.noughts_player];
                } else {
                    current_player = &players[state.noughts_player];
                    next_player = &players[state.crosses_player];
                }

                match state.turn {
                    Turn::Begin => {
                        let config1 = ClientState::new(next_player.addr.to_string(), Piece::Cross, state.board.size);
                        let config2 = ClientState::new(current_player.addr.to_string(), Piece::Nought, state.board.size);
                        Session::send(current_player, Message::Preamble(config1)).unwrap();
                        Session::send(next_player, Message::Preamble(config2)).unwrap();
                        println!("Found {} and {}", current_player.addr, next_player.addr);
                        state.turn = Turn::TurnStart;
                    },
                    Turn::TurnStart => {
                        Session::send(current_player, Message::YourTurn).unwrap();
                        Session::send(next_player, Message::WaitTurn).unwrap();
                        state.turn = Turn::TurnWait;
                    },
                    Turn::TurnWait => {
                        match super::try_recv(current_player) {
                            Ok(Message::Move((_, x, y))) => {
                                match state.board.try_place(dbg!(state.current_player.clone()), x, y) {
                                    Ok(m) => {
                                        Session::broadcast(current_player, next_player, Message::Move(m)).unwrap();
                                        match state.board.check_victory(state.current_player.clone()) {
                                            Some(end) => {
                                                state.winner = end;
                                                state.turn = Turn::End;
                                            },
                                            None => {
                                                state.turn = Turn::TurnStart;
                                                state.current_player = state.current_player.next();
                                            },
                                        }
                                    },
                                    Err(e) => Session::send(current_player, Message::InvalidMove(e)).unwrap(),
                                }
                            },
                            Ok(m) => Session::send(current_player, Message::InvalidMove(format!("Wrong message type {m:?}"))).unwrap(),
                            Err(_) => (), // nothing received
                        }
                    },
                    Turn::End => {
                        Session::broadcast(current_player, next_player, Message::GameOver(state.winner.clone())).unwrap();
                        players[0].status = super::Status::Waiting;
                        players[1].status = super::Status::Waiting;
                        println!("Game over, winner: {:?}", state.winner);
                        break;
                    }
                }
            },
            1 => {
                let player = &mut players[0];
                println!("One player dropped");
                Session::send(player, Message::GameOver(End::Disconnect)).unwrap();
                player.status = super::Status::Waiting;
                break;
            },
            _ => {
                println!("Both players dropped");
                break;
            }
        }
    }
}
