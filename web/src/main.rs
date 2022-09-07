#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Complile for wasm");
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "the_canvas_id", // hardcode it
        web_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    )
    .expect("failed to start eframe");
}
