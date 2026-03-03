mod app;
mod routing;
mod state;
mod audio;
mod components;
mod canvas;
mod utils;

use app::App;
use wasm_bindgen::prelude::wasm_bindgen;

/// WASM entry point — called when the module is instantiated by the browser.
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
