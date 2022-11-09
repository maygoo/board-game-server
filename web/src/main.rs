use wasm_bindgen::prelude::*;

mod app;
pub use app::WebApp;

/// Defines a `println!`-esque macro that binds to js `console.log`
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => ($crate::log_js(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern {
    /// bind to the js function `console.log`
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_js(s: &str);
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Compile for wasm");
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
        "ce7bccc0-da54-48af-a0ee-142ef8570fe5", // hardcode it
        web_options,
        Box::new(|cc| Box::new(WebApp::new(cc))),
    )
    .expect("failed to start eframe");
}
