use std::{
    thread,
    io::prelude::*,
    net::TcpStream,
    sync::mpsc::{channel, Sender, Receiver},
    time::Duration,
};

use wasm_bindgen::prelude::*;

use common::tic_tac_toe::{
    ClientState,
    Piece,
    Message,
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
const REMOTE_IP: &str = "127.0.0.1:3334";
#[cfg(not(debug_assertions))]
const REMOTE_IP: &str = "ec2-3-25-98-214.ap-southeast-2.compute.amazonaws.com:3334";

pub struct TemplateApp {
    // Example stuff:
    remote_ip: String,
    value: f32,
    state: ClientState,
    stream: Option<TcpStream>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            remote_ip: REMOTE_IP.to_owned(),
            value: 2.7,
            state: ClientState::new(String::new(), Piece::Empty, 0),
            stream: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.stream.is_some() {
            let mut stream = self.stream.as_ref().unwrap();

            // non-blocking read
            let mut recv = [0 as u8; common::BUFF_SIZE];
            match stream.read(&mut recv) {
                Ok(size) if size > 0 => {
                    let msg: Message = bincode::deserialize(&recv).unwrap();

                    // send message
                    /* match play(msg, &mut state, &rx) {
                        Some(msg) => { stream.write(&bincode::serialize(&msg).unwrap()).unwrap(); }
                        None => (),
                    } */
                    log!("Incoming message: {:?}", msg);
                },
                Ok(_) => {
                    log!("Connectedion closed");
                    self.stream = None;
                }, // connection is closed if size == 0
                Err(_) => (), // ignore the timeout
            }
        }

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

            if ui.button("join").clicked() && self.stream.is_none() {
                self.stream = match TcpStream::connect(&self.remote_ip) {
                //self.stream = match TcpStream::connect_timeout(&self.remote_ip.parse().unwrap(), Duration::from_millis(100)) {
                    Ok(mut stream) => {
                        log!("Connected!");
                        //stream.set_nonblocking(true);
                        Some(stream)
                    }
                    Err(e) => {
                        log!("Unable to connect: {e}");
                        None
                    },
                };
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

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);

            ui.add_space(10.0);
            ui.heading(common::tic_tac_toe::NAME);
            ui.small(common::tic_tac_toe::INSTRUCTIONS);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
