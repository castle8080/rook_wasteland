// Clippy lints: warn on unreviewed unwrap/todo usage so new instances are
// visible in CI output. expect() with an explanatory message is acceptable
// for genuinely infallible operations (see doc/rust_code_principles.md §1.1).
#![warn(clippy::unwrap_used)]
#![warn(clippy::todo)]

// When compiling for the native test target (rlib, not wasm32) or for
// wasm-pack test (wasm32 + cfg(test)), rustc cannot trace through the
// #[wasm_bindgen(start)] entry point and flags canvas functions, component
// structs, and audio helpers as dead code.  All of these items ARE reachable
// in the real cdylib/WASM build.
#![cfg_attr(any(not(target_arch = "wasm32"), test), allow(dead_code, unused_imports))]

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
