mod app;
mod routing;
pub mod state;
pub mod audio;
mod components;
mod canvas;
mod utils;

// These imports are only used by the #[wasm_bindgen(start)] entry point,
// which is excluded during `wasm-pack test` to avoid a duplicate start symbol.
#[cfg(not(test))]
use app::App;
#[cfg(not(test))]
use wasm_bindgen::prelude::wasm_bindgen;

// Configure all #[wasm_bindgen_test] tests in this crate to run in a real browser.
// Place this here (crate root, cfg(test)) so it covers every inline test module.
#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

/// WASM entry point — called when the module is instantiated by the browser.
///
/// Excluded during `wasm-pack test` (`cfg(test)`) because the test harness
/// injects its own start symbol; two start symbols cause a linker error.
#[cfg(not(test))]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
