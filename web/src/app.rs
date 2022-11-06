use gloo_net::websocket::futures::WebSocket;

use common::tic_tac_toe::{
    ClientState,
    Piece,
    Message,
    Board,
    Turn,
};
use crate::log;

mod style;
use style::Style;

mod worker;
use worker::Worker;

struct Info {
    pub text: String,
    locked: bool,
}

impl Info {
    pub fn new() -> Self {
        Info {
            text: String::new(),
            locked: false,
        }
    }

    pub fn update(&mut self, new: String) -> &mut Self {
        if !self.locked {
            self.text = new;
        }
        self
    }

    pub fn unlock(&mut self) -> &mut Self {
        self.locked = false;
        self
    }

    pub fn lock(&mut self) -> &mut Self {
        self.locked = true;
        self
    }
}

pub struct WebApp {
    // Example stuff:
    remote_ip: String,
    state: ClientState,
    worker: Option<Worker>,
    info: Info,
}

impl Default for WebApp {
    fn default() -> Self {
        Self {
            remote_ip: common::REMOTE_IP.to_owned(),
            state: ClientState::new(String::new(), Piece::Empty, 0),
            worker: None,
            info: Info::new(),
        }
    }
}

impl WebApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }
}

impl eframe::App for WebApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_style(Style::build_style((*ctx.style()).clone()));

        egui::CentralPanel::default().show(ctx, |ui| {

            ui.vertical_centered(|ui| {

                ui.heading("Board Games");
                ui.add(egui::Hyperlink::from_label_and_url(
                    egui::RichText::new("Github").size(14.0),
                    "https://github.com/maygoo/board-games-rust"
                ));
                egui::warn_if_debug_build(ui);

                ui.separator();

                if self.worker.is_none() {

                    ui.add(egui::widgets::TextEdit::singleline(&mut self.remote_ip)
                        .text_color(Style::CORAL));

                    if ui.button("Connect to the server").clicked() && self.worker.is_none() {
                        let ip = format!("wss://{}:{}", &self.remote_ip, common::REMOTE_PORT);
                        match WebSocket::open(&ip) {
                            Ok(ws) => {
                                self.worker = Some(Worker::new(ws));
                            },
                            Err(e) => log!("can't connect to websocket: {e}"),
                        }
                    }

                } else {
                    ui.heading(common::tic_tac_toe::NAME);

                    #[cfg(debug_assertions)]
                    ui.label(format!{"State: {:?}", self.state});

                    ui.label(format!("You are player: {}", self.state.piece));
                    ui.label(&self.info.text);

                    match self.state.turn {
                        Turn::Begin => self.info.update("Wait for another player to appear".to_string()),
                        Turn::TurnStart => self.info.update("It is your turn!".to_string()),
                        Turn::TurnWait => self.info.update("Wait for your opponent to make their turn".to_string()),
                        Turn::End => {
                            // prompt player to play again
                            &mut self.info
                        },
                    };

                    // consume messages from the channel
                    match self.worker.as_ref().unwrap().rx.try_recv() {
                        Ok(msg) => {
                            match msg.into() {
                                Message::Preamble(config) => {
                                    self.state = config;
                                    self.state.board = Board::new(self.state.board.size);
                                },
                                Message::WaitTurn => self.state.turn = Turn::TurnWait,
                                Message::YourTurn => self.state.turn =Turn::TurnStart,
                                Message::Move((p, x, y)) => {
                                    self.state.board.place(p, x, y);
                                    self.info.unlock();
                                },
                                Message::InvalidMove(err) => {
                                    self.info.unlock().update(err).lock();
                                    self.state.turn = Turn::TurnStart;
                                },
                                Message::GameOver(end) => {
                                    self.info.unlock().update(format!("{end:?}")).lock();

                                    self.state.turn = Turn::End;
                                    // display window popup
                                },
                            }
                        },
                        Err(_) => (),
                    }

                    // display board
                    match display_board(ui, &self.state.board, self.state.turn == Turn::TurnStart) {
                        Some((x, y)) => {
                            self.state.turn = Turn::TurnWait;
        
                            self.worker
                                .as_ref()
                                .unwrap()
                                .tx.send(
                                    Message::Move(
                                        (self.state.piece.clone(),
                                        x,
                                        y
                                    )).into()
                                ).unwrap();
                        },
                        None => (),
                    }
                }
            });
        });
    }
}

fn display_board(ui: &mut egui::Ui, board: &Board, clickable: bool) -> Option<(usize, usize)> {
    let mut turn = None;
    for (y, row) in board.iter().enumerate() {
        ui.horizontal(|ui| {
            for (x, cell) in row.iter().enumerate() {
                if ui.add_sized([20.0, 20.0], egui::Button::new(cell.to_string())).clicked() && clickable {
                    log!("clicked pos: {x},{y}");
                    turn = Some((x, y));
                }
            }
        });
    }

    // validate turn here then return turn if valid
    
    turn
}
