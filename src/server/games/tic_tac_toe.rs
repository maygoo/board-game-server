use std::{
    thread,
    sync::{Arc, Mutex},
};

use crate::{
    WAIT,
    games::{Player, Session}
};

use common::tic_tac_toe::{
    self,
    Message,
    ServerState,
    Piece,
    Turn, ClientState,
};

pub fn begin(players: Arc<Mutex<Vec<Player>>>, mut session: super::Session) {
    session.game = Some(super::Game::TicTacToe);

    let mut state = ServerState::new(tic_tac_toe::BOARD_SIZE);
    
    println!("Started {:?} with {} and {}", tic_tac_toe::NAME, session.player1, session.player2);
    
    loop {
        thread::sleep(WAIT);
        let mut data = players.lock().unwrap();
        // check that both players are still connected
        let mut players: Vec<&mut Player> = data
            .iter_mut()
            .filter(|p| p.addr == session.player1 || p.addr == session.player2)
            .collect();
        
        match players.len() {
            2 => {
                let player1 = &players[0]; // player 1; crosses
                let player2 = &players[1]; // player 2; noughts
                match state.turn {
                    Turn::Begin => {
                        let config1 = ClientState::new(player2.addr.to_string(), Piece::Cross, state.board.size);
                        let config2 = ClientState::new(player1.addr.to_string(), Piece::Nought, state.board.size);

                        Session::send(player1, Message::Preamble(config1)).unwrap();
                        Session::send(player2, Message::Preamble(config2)).unwrap();
                        println!("Found {} and {}", player1.addr, player2.addr);
                        
                        state.turn = Turn::CrossStart;
                    },
                    
                    Turn::CrossStart => {
                        Session::send(player1, Message::YourTurn).unwrap();
                        Session::send(player2, Message::WaitTurn).unwrap();
                        state.turn = Turn::CrossWait;
                    },
                    Turn::CrossWait => {
                        match super::try_recv(player1) {
                            Ok(Message::Move((_, x, y))) => {
                                match state.board.try_place(x, y, Piece::Cross) {
                                    Ok((x, y)) => {
                                        Session::broadcast(player1, player2, Message::Move((Piece::Cross, x, y))).unwrap();
                                        if state.board.check_victory(Piece::Cross) {
                                            state.winner = Piece::Cross;
                                            state.turn = Turn::End;
                                        } else {
                                            state.turn = Turn::NoughtStart;    
                                        }
                                    },
                                    Err(e) => Session::send(player1, Message::InvalidMove(e)).unwrap(),
                                }
                            },
                            Ok(m) => Session::send(player1, Message::InvalidMove(format!("Wrong message type {m:?}"))).unwrap(),
                            Err(_) => (), // nothing received
                        }
                    }
                    Turn::NoughtStart => {
                        Session::send(player2, Message::YourTurn).unwrap();
                        Session::send(player1, Message::WaitTurn).unwrap();
                        state.turn = Turn::NoughtWait;
                    },
                    Turn::NoughtWait => {
                        match super::try_recv(player2) {
                            Ok(Message::Move((_, x, y))) => {
                                match state.board.try_place(x, y, Piece::Nought) {
                                    Ok((x, y)) => {
                                        Session::broadcast(player1, player2, Message::Move((Piece::Nought, x, y))).unwrap();
                                        if state.board.check_victory(Piece::Nought) {
                                            state.winner = Piece::Nought;
                                            state.turn = Turn::End;
                                        } else {
                                            state.turn = Turn::CrossStart;    
                                        }
                                    },
                                    Err(e) => Session::send(player2, Message::InvalidMove(e)).unwrap(),
                                }
                            },
                            Ok(m) => Session::send(player2, Message::InvalidMove(format!("Wrong message type {m:?}"))).unwrap(),
                            Err(_) => (), // nothing received
                        }
                    },
                    Turn::End => {
                        Session::broadcast(player1, player2, Message::GameOver(state.winner.clone())).unwrap();
                        players[0].status = super::Status::Waiting;
                        players[1].status = super::Status::Waiting;
                        println!("Game over, {} has won", state.winner);
                        break;
                    }
                }
            },
            1 => {
                let player = &mut players[0];
                println!("One player dropped");
                Session::send(player, Message::GameOver(Piece::Empty)).unwrap();
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
