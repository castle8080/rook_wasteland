// When compiling for the native test target, web_sys / gloo / Leptos DOM
// components cannot compile. Gating the browser-only modules here lets
// `cargo test` compile and run pure-Rust unit tests (router, state, scoring)
// without errors. See doc/lessons.md L4.
#![cfg_attr(any(not(target_arch = "wasm32"), test), allow(dead_code, unused_imports))]

pub mod error;
pub mod router;
pub mod state;

#[cfg(target_arch = "wasm32")]
pub mod app;
#[cfg(target_arch = "wasm32")]
pub mod components;
#[cfg(target_arch = "wasm32")]
pub mod dice_svg;
#[cfg(target_arch = "wasm32")]
pub mod worker;

// These imports are only used by the #[wasm_bindgen(start)] entry point.
// Excluded during `wasm-pack test` (feature "wasm-test"), `cargo test` (cfg(test)),
// and when building the worker binary (feature "worker").
// See doc/lessons.md L5.
#[cfg(all(target_arch = "wasm32", not(test), not(feature = "wasm-test"), not(feature = "worker")))]
use app::App;
#[cfg(all(target_arch = "wasm32", not(test), not(feature = "wasm-test"), not(feature = "worker")))]
use wasm_bindgen::prelude::wasm_bindgen;

// Configure all #[wasm_bindgen_test] tests in this crate to run in a real browser.
#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

/// WASM entry point — called when the module is instantiated by the browser.
///
/// Excluded when `feature = "wasm-test"` is active (browser integration tests),
/// when `feature = "worker"` is active (grandma worker binary uses its own start),
/// and during `cargo test` (native tests use a separate test harness).
#[cfg(all(target_arch = "wasm32", not(test), not(feature = "wasm-test"), not(feature = "worker")))]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
