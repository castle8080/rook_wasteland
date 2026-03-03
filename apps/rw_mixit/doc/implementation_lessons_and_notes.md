# rw_mixit — Implementation Lessons & Notes

Running log of discoveries, gotchas, and patterns worth remembering across milestones.

---

## Lessons from M0 — Project Scaffold

### `view!` macro and dead-code lints

Functions called only from inside `view!` closures are flagged as unused by the compiler because the macro's expansion scope isn't traced by the dead-code analysis. Pattern: inline one-liners directly into the closure. Larger helpers that are also called from non-macro contexts will be fine.

### `RwSignal` requires `Send + Sync` on inner types

Standard `RwSignal::new()` requires `T: Send + Sync`. This matters from M1 onward when `web-sys` types like `AudioBuffer` and `AudioContext` are `!Send`. Those need `RwSignal::new_local()` or must live in `Rc<RefCell<...>>` outside the signal system entirely — as the spec already plans for `AudioDeck`.

### `std::mem::forget` for app-lifetime listeners

`gloo_events::EventListener` removes the browser listener on drop. For listeners that must live forever (hashchange in `App`, keydown in M10), use `std::mem::forget(listener)` rather than storing in a reactive primitive. It is intentional — do not "fix" it.

### `[lib] crate-type = ["cdylib", "rlib"]` — two compilation units

`cargo test` compiles the `rlib` target for the native host. `cargo clippy --target wasm32-unknown-unknown` compiles the `cdylib` for WASM. They can diverge — a warning may appear on one target but not the other. Always run clippy against the wasm32 target specifically, as the task workflow requires.

### `#[wasm_bindgen(start)]` is load-bearing

Without `#[wasm_bindgen(start)]` on `fn main()` in `lib.rs`, the WASM module silently does nothing — the app never starts and there is no error in the console. Easy to accidentally lose if `lib.rs` is refactored. It also suppresses the dead-code warning that `fn main()` in a lib crate would otherwise produce.

### BPM and DSP code can be unit-tested natively

The `rlib` target means all pure-Rust code (`src/audio/bpm.rs`, peak extraction in `src/audio/loader.rs`, etc.) can have `#[test]` functions that run with plain `cargo test` — no browser, no WASM needed. The M4 BPM tasks (`compute_spectral_flux`, `estimate_bpm`) are explicitly designed with this in mind; write the core logic as testable pure functions.
