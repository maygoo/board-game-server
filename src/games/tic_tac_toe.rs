use std::{
    thread,
    sync::{Arc, Mutex},
};
use serde::{Serialize, Deserialize};

use crate::{
    WAIT,
    games::{Player, Session}
};

#[derive(PartialEq)]
enum Turn {
    Begin,
    CrossStart,
    CrossWait,
    NoughtStart,
    NoughtWait,
    End,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Message {
    Preamble(Config),
    WaitTurn,
    YourTurn,
    Move((Piece, usize, usize)),
    InvalidMove(String),
    // Nought or Cross piece means they win
    // Empty piece means game is over i.e. disconnect
    GameOver(Piece),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    opponent: String,
    piece: Piece,
    boardsize: usize,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum Piece {
    Nought,
    Cross,
    Empty
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Nought => write!(f, "O"),
            Self::Cross  => write!(f, "X"),
            Self::Empty  => write!(f, " "),
        }
    }
}

struct State {
    turn: Turn,
    board: Vec<Vec<Piece>>,
    winner: Piece,
}

const NAME: &str = "Tic Tac Toe";
const BOARD_SIZE: usize = 3;

pub fn begin(players: Arc<Mutex<Vec<Player>>>, mut session: super::Session) {
    thread::spawn(move|| {
        session.game = Some(super::Game::TicTacToe);

        let mut state = State {
            turn: Turn::Begin,
            board: vec![vec![Piece::Empty; BOARD_SIZE]; BOARD_SIZE],
            winner: Piece::Empty,
        };
        
        println!("Started {:?} with {} and {}", NAME, session.player1, session.player2);
        
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
                            let config1 = Config {
                                opponent: player2.addr.to_string(),
                                piece: Piece::Cross,
                                boardsize: BOARD_SIZE,
                            };

                            let config2 = Config {
                                opponent: player1.addr.to_string(),
                                piece: Piece::Nought,
                                boardsize: BOARD_SIZE,
                            };

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
                            match try_move(player1, Piece::Cross, &mut state) {
                                Ok((x, y)) => {
                                    Session::broadcast(player1, player2, Message::Move((Piece::Cross, x, y))).unwrap();
                                    if check_victory(&state, Piece::Cross) {
                                        state.winner = Piece::Cross;
                                        state.turn = Turn::End;
                                    } else {
                                        state.turn = Turn::NoughtStart;    
                                    }
                                },
                                Err(e) if e.is_empty() => (),
                                Err(e) => Session::send(player1, Message::InvalidMove(e)).unwrap(),
                            };
                        }
                        Turn::NoughtStart => {
                            Session::send(player2, Message::YourTurn).unwrap();
                            Session::send(player1, Message::WaitTurn).unwrap();
                            state.turn = Turn::NoughtWait;
                        },
                        Turn::NoughtWait => {
                            match try_move(player2, Piece::Nought, &mut state) {
                                Ok((x, y)) => {
                                    Session::broadcast(player1, player2, Message::Move((Piece::Nought, x, y))).unwrap();
                                    if check_victory(&state, Piece::Nought) {
                                        state.winner = Piece::Nought;
                                        state.turn = Turn::End;
                                    } else {
                                        state.turn = Turn::CrossStart;    
                                    }
                                },
                                Err(e) if e.is_empty() => (),
                                Err(e) => Session::send(player2, Message::InvalidMove(e)).unwrap(),
                            };
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
    });
}

fn try_move(player: &Player, piece: Piece, state: &mut State) -> Result<(usize, usize), String> {
    match player.rx.try_recv() {
        Ok(Message::Move((_, x, y))) => {
            // check cell is empty then do move
            match &mut state.board[y][x] {
                Piece::Empty => {
                    state.board[y][x] = piece;
                    Ok((x, y))
                }
                p => Err(format!("{} {} already has a {p} on it! Enter another move", (y + 65) as u8 as char, x+1)), // quick convert idxs to game coords
            }
        },
        Ok(m) => Err(format!("Wrong message type {m:?}")),
        Err(_) => Err(String::new()), // nothing received so return empty string
    }
}

// check if piece wins
fn check_victory(state: &State, piece: Piece) -> bool {
    let board = &state.board;

    // check for horizontal victoryD
    let mut win = board.iter().filter(|row| {
        row.iter().filter(|cell| {
            **cell == piece
        }).count() == BOARD_SIZE
    }).count();

    // TODO reduce by using zip function ? maybe
    // check for vertical victory
    for i in 0..BOARD_SIZE {
        let mut flag = true;
        for j in 0..BOARD_SIZE {
            if board[j][i] != piece { flag = false; break; }
        }
        if flag {
            win = 1;
            break;
        }
    }

    // check for diag victory
    let mut flag1 = true;
    let mut flag2 = true;
    for i in 0..BOARD_SIZE {
        if board[i][i] != piece { flag1 = false; }
        if board[i][BOARD_SIZE-i-1] != piece { flag2 = false; }

        if !flag1 && !flag2 { break; }
    }
    if flag1 || flag2 { win = 1; }
    
    win > 0
}

fn _print_board(board: &Vec<Vec<Piece>>) -> String {
    let mut board_str = String::new();

    // create the horizontal separator
    // based on the board size
    let sep = "-".repeat(board.len() * 2 - 1);

    for (i, row) in board.iter().enumerate() {
        if i > 0 { board_str.push_str(format!("{}\n", sep).as_str()); }

        for (j, cell) in row.iter().enumerate() {
            if j > 0 { board_str.push_str("|") }

            board_str.push_str(format!("{}", cell).as_str());
        }

        board_str.push('\n');
    }

    board_str
}