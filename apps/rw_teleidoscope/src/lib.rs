// Clippy lints: warn on unreviewed unwrap/todo usage so new instances are
// visible in CI output. expect() with an explanatory message is acceptable
// for genuinely infallible operations.
#![warn(clippy::unwrap_used)]
#![warn(clippy::todo)]

// When compiling for the native test target (rlib, not wasm32) or for
// wasm-pack test (wasm32 + cfg(test)), rustc cannot trace through the
// #[wasm_bindgen(start)] entry point and flags component structs, renderer
// helpers, and state types as dead code. All of these ARE reachable in the
// real cdylib/WASM build.
#![cfg_attr(any(not(target_arch = "wasm32"), test), allow(dead_code, unused_imports))]

mod camera;
mod routing;
pub mod state;
pub mod utils;

// These modules rely on browser / WebGL APIs that are only available on
// wasm32.  Gating them here allows `cargo test` (native target) to compile
// and run the pure-Rust unit tests in `utils` and `state` without errors.
#[cfg(target_arch = "wasm32")]
pub mod app;
#[cfg(target_arch = "wasm32")]
pub mod components;
#[cfg(target_arch = "wasm32")]
pub mod renderer;

// These imports are only used by the #[wasm_bindgen(start)] entry point.
// Excluded during `wasm-pack test` (feature "wasm-test") and during
// `cargo test` (cfg(test)) to avoid a duplicate `main` symbol conflict with
// the test harness entry point.
#[cfg(all(target_arch = "wasm32", not(test), not(feature = "wasm-test")))]
use app::App;
#[cfg(all(target_arch = "wasm32", not(test), not(feature = "wasm-test")))]
use wasm_bindgen::prelude::wasm_bindgen;

// Configure all #[wasm_bindgen_test] tests in this crate to run in a real browser.
#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

/// WASM entry point — called when the module is instantiated by the browser.
///
/// Excluded when `feature = "wasm-test"` is active (browser integration tests)
/// because the test harness injects its own `main` entry point; two `main`
/// exports cause wasm-ld to discard both, making the test binary unrunnable.
#[cfg(all(target_arch = "wasm32", not(test), not(feature = "wasm-test")))]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
