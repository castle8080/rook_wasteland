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


---

## Lessons from M2 — Playback & Waveform

### `AudioBufferSourceNode` is one-shot

An `AudioBufferSourceNode` can only be started once. After `stop()` it is inert and must be discarded. Store in `Option<AudioBufferSourceNode>` and create a fresh node on every `play()` call.

### `stop()` / `stop_with_when()` deprecation in web-sys 0.3.91

Both `AudioBufferSourceNode::stop()` (no args) and `stop_with_when(f64)` are marked deprecated in current web-sys. There is no non-deprecated replacement in this version — use `#[allow(deprecated)]` on the wrapping helper function and `stop_with_when(0.0)` until a newer web-sys updates the bindings.

### `linear_ramp_to_value_at_time` argument types

In web-sys the signature is `(value: f32, end_time: f64)` — the target value is `f32`, the time is `f64`. Passing two `f64`s causes a type error that is not always obvious from the message.

### Canvas draw method naming (3-arg form)

`draw_image_with_html_canvas_element(canvas, dx, dy)` is the 3-argument form in web-sys that positions the blit. Do NOT use `draw_image_with_html_canvas_element_and_dx_and_dy` — that overload does not exist with that name and will fail to compile.

### `set_fill_style_str` / `set_stroke_style_str`

Use `ctx.set_fill_style_str("colour")` (and the stroke equivalent) instead of the deprecated `set_fill_style(&JsValue)`. Both are available in web-sys 0.3.91.

### rAF closure memory pattern in WASM

The recursive `requestAnimationFrame` loop requires `Rc<RefCell<Option<Closure<dyn FnMut()>>>>` with the `g = f.clone(); *g.borrow_mut() = Some(Closure::new(move || { ... }))` idiom. The whole setup must be wrapped in `spawn_local(async move { ... })` so it runs after the first render — otherwise `NodeRef`s for canvas elements are still `None`.

### Sharing a closure across two `on:` handlers

Two event listeners on the same element (e.g. `on:mouseup` + `on:mouseleave` for nudge-end) cannot share a single Rust `Fn` closure — the borrow checker prevents moving it into both. Solution: wrap in `Rc<dyn Fn()>` and `.clone()` for each handler.

### Leptos trait imports for signals in non-component modules

`get_untracked()`, `set()`, `get()` on `RwSignal` are trait methods from `leptos::prelude`. Canvas/rAF modules that don't already pull in `use leptos::prelude::*` must add it explicitly or the methods appear to not exist.

### Clippy `new_without_default` on state structs

State structs with a `new()` constructor should also implement `Default` (delegating to `Self::new()`). Clippy's `-D warnings` mode will refuse to compile without it. Add a trivial `impl Default` for every state struct that has `new()`.
