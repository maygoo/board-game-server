use std::fmt::Display;
use std::sync::mpsc::{channel, Sender, Receiver};
use wasm_bindgen::prelude::*;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message as WsMessage;
use wasm_bindgen_futures::spawn_local;
use futures::{SinkExt, StreamExt};

use common::{WAIT_MS, ChannelBuf};
use common::tic_tac_toe::{
    ClientState,
    Piece,
    Message,
    Board,
};

/// Defines a `println!`-esque macro that binds to js `console.log`
macro_rules! log {
    ($($t:tt)*) => (log_js(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern {
    /// bind to the js function `console.log`
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_js(s: &str);
}

#[cfg(debug_assertions)]
const REMOTE_IP: &str = "ws://127.0.0.1:3334";
#[cfg(not(debug_assertions))]
const REMOTE_IP: &str = "ws://ec2-3-25-98-214.ap-southeast-2.compute.amazonaws.com:3334";

pub struct TemplateApp {
    // Example stuff:
    remote_ip: String,
    state: ClientState,
    worker: Option<Worker>,
    turn: bool,
}

struct Worker {
    tx: Sender<ChannelBuf>,
    rx: Receiver<ChannelBuf>,
}

impl Worker {
    pub fn new(mut ws: WebSocket) -> Self {
        let (tx_t, rx) = channel::<ChannelBuf>();
        let (tx, rx_t) = channel::<ChannelBuf>();
        
        spawn_local(async move {
            log!("Connected to websocket");

            loop {
                // should equate to a thread::sleep
                gloo_timers::future::TimeoutFuture::new(WAIT_MS).await;

                // check for any incoming messages on the websocket
                match futures::poll!(ws.next()) {
                    futures::task::Poll::Ready(
                        Some(
                        Ok(
                        WsMessage::Bytes(bytes)
                    ))) => {
                        // forward message through the channel
                        log!("msg: {bytes:?}");
                        tx_t.send(bytes).unwrap();
                    },
                    _ => (),
                }

                // check for any incoming messages on the channel
                match rx_t.try_recv() {
                    Ok(msg) => {
                        log!("woop woop msg received on channel");
                        // forward message through the websocket
                        ws.send(WsMessage::Bytes(msg)).await.unwrap();
                    },
                    _ => (),
                }
            }
        });

        Worker {
            tx,
            rx,
        }
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            remote_ip: REMOTE_IP.to_owned(),
            state: ClientState::new(String::new(), Piece::Empty, 0),
            worker: None,
            turn: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Lobby");

            ui.horizontal(|ui| {
                ui.label("Server: ");
                ui.text_edit_singleline(&mut self.remote_ip);
            });

            if ui.button("join").clicked() && self.worker.is_none() {
                match WebSocket::open(&self.remote_ip) {
                    Ok(ws) => {
                        self.worker = Some(Worker::new(ws));
                    },
                    Err(_) => (),
                }
            }

            if self.worker.is_some() {
                ui.label("connected");

                if ui.button("send YouTurn message").clicked() {
                    self.worker
                        .as_ref()
                        .unwrap()
                        .tx.send(
                            Message::YourTurn.into()).unwrap();
                }

                if ui.button("send WaitTurn message").clicked() {
                    self.worker
                        .as_ref()
                        .unwrap()
                        .tx.send(
                            Message::YourTurn.into()).unwrap();
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Board Games");
            ui.add(egui::github_link_file!(
                "https://github.com/maygoo/board-games-rust/tree/gui/web/",
                "Project source"
            ));
            egui::warn_if_debug_build(ui);

            ui.separator();

            if self.worker.is_none() {
                ui.heading("Join a game");
            } else {
                ui.heading(common::tic_tac_toe::NAME);
                ui.label(common::tic_tac_toe::INSTRUCTIONS);

                ui.label(format!{"State: {:?}", self.state});

                // consume messages from the channel
                match self.worker.as_ref().unwrap().rx.try_recv() {
                    Ok(msg) => {
                        match msg.into() {
                            Message::Preamble(config) => {
                                self.state = config;
                                self.state.board = Board::new(self.state.board.size);
                            },
                            Message::WaitTurn => {
                                log!("waiting for your turn");
                                self.turn = false;
                            },
                            Message::YourTurn => {
                                self.turn = true;
                            },
                            Message::Move((p, x, y)) => {
                                self.state.board.place(p, x, y)
                            },
                            Message::InvalidMove(e) => {
                                log!("{e}");
                                self.turn = true;
                            },
                            Message::GameOver(end) => {
                            },
                        }
                    },
                    Err(_) => (),
                }

                // display board
                match display_board(ui, &self.state.board, self.turn) {
                    Some((x, y)) => {
                        self.turn = false;
    
                        self.worker
                            .as_ref()
                            .unwrap()
                            .tx.send(
                                Message::Move(
                                    (self.state.piece.clone(),
                                    x,
                                    y))
                                .into())
                            .unwrap();
                    },
                    None => (),
                }
            }
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
