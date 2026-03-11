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

---

## Lessons from M3 — Platter Animation & Speed Control

### `ctx.save()` and `ctx.restore()` return `()`

`CanvasRenderingContext2d::save()` and `restore()` in web-sys return `()`, not
`Result`. Do not call `.expect()` on them. Only `translate()`, `rotate()`, and
`arc()` return `Result<(), JsValue>` and need `.expect()`.

### `#[allow(clippy::too_many_arguments)]` on `start_raf_loop`

Clippy enforces a 7-argument limit by default. `start_raf_loop` now has 8
parameters (2 state, 2 audio, 2 waveform, 2 platter). Suppress with
`#[allow(clippy::too_many_arguments)]` and a comment explaining the count.
If the parameter count grows further, wrap per-deck arguments into a struct.

### Groove rotation angle is stateless — no accumulated float issue

The rotation angle is computed fresh each frame as
`current_secs * RPM_33_RPS * playback_rate * TAU`. For tracks up to 2 hours,
`current_secs` stays below 7200, giving an angle below ~25 000 rad — well
within f64 precision. No angle accumulator needed.

### Canvas text_baseline, set_font, set_text_align take `&str` directly

Unlike `fill_style`/`stroke_style` (which needed `_str` suffix variants),
`set_text_baseline`, `set_font`, `set_text_align`, and `set_line_cap` all
already accept `&str` in web-sys 0.3 with no deprecated JsValue variant.
Use them directly without the `_str` suffix.

---

## Lessons from M4 — BPM Detection & Sync

### Autocorrelation double-period bias requires a sub-lag check

For integer-lag autocorrelation on a flux impulse train, the lag corresponding
to double the true period (half the true BPM) can score equally high or higher
than the true period lag. This happens because of two effects:

1. **Integer alignment**: the true beat period (e.g. 40.37 frames) is not an
   integer. The lag `round(2*T)` = 81 frames can align more cleanly than
   `round(T)` = 40 frames when the per-beat drift is small over many beats.

2. **Fewer-but-cleaner pairs**: lag `2T` has half the pairs of lag `T`, but
   each pair drifts half as much per beat, so the signal does not de-correlate
   as quickly.

**Fix**: After finding the best lag, check `half_lag = best_lag / 2`.  If
`half_lag >= lag_min` and its correlation is ≥ 50% of the best lag's
correlation, prefer `half_lag`. For a true-period lag (e.g. lag 40 for 128 BPM),
`half_lag` = 20 falls below `lag_min` = 26, so the check never fires and the
correct BPM is preserved. For a double-period lag (e.g. lag 81 for 64 BPM),
`half_lag` = 40 is in range and has a strong correlation → BPM corrects to 128.

### `DeckId` needs `Copy` to be used as a component prop and in closures

`DeckId` is an enum used inside move closures for SYNC/MASTER handlers.
Add `#[derive(Copy)]` so it can be captured by value without `.clone()` calls.

### `MixerState` must be created once in `DeckView` and signals distributed

`bpm_a`, `bpm_b`, and `sync_master` live on `MixerState` but each `Deck`
only receives the signals it needs (own BPM, other BPM, master).
Both decks share the same `sync_master: RwSignal<Option<DeckId>>` — a signal
is `Copy` in Leptos so it can be passed to both components directly.

### TAP BPM tap-time storage uses `Rc<RefCell<Vec<f64>>>`

The TAP button handler needs mutable access to a `Vec<f64>` of tap timestamps.
Because Leptos `on:click` closures require `Fn + 'static`, use
`Rc<RefCell<Vec<f64>>>` and `.clone()` the Rc into the closure. The Vec is
capped at 9 entries (8 intervals) by removing the oldest on overflow.

### `window.performance().now()` requires the `"Performance"` web-sys feature

`web_sys::Window::performance()` returns `Option<web_sys::Performance>`, and
`Performance::now()` returns `f64` milliseconds. Both are already enabled via
the `"Performance"` entry in `Cargo.toml`'s web-sys feature list.

---

## Lessons from M5 — Mixer Panel

### Shared audio nodes need lazy init co-located with the first-user-gesture path

The Web Audio API requires a user gesture before `AudioContext` can be created.
`MixerAudio` (xfade gains + master gain) is shared across both decks and must
use the same `AudioContext`, so it cannot be constructed at `DeckView` render
time. The pattern: store `Rc<RefCell<Option<MixerAudio>>>`, create it inside the
`on_file_change` handler on first call (same place `AudioContext` is lazily
initialised), then connect each deck's `analyser` node immediately after its
`AudioDeck` is created. Both deck closures race to create `MixerAudio`; the
`is_none()` guard ensures only the first wins.

### Apply current signal values immediately after lazy node creation

Leptos `Effect`s track reactive signals — they fire when a signal changes, not
when the audio node they target comes into existence. If the user moves the
crossfader or master volume slider *before* loading any track, the Effects fire
but `MixerAudio` is still `None` (no-op). When `MixerAudio` is finally created,
those Effects won't re-fire because the signals didn't change. Fix: read the
current signal values with `.get_untracked()` and apply them to the newly
created nodes at construction time.

### Connect audio graph topology exactly once — guard with the same `is_none()` check

`AudioNode::connect()` in the Web Audio API is idempotent: repeated calls
between the same source→destination pair are silently ignored. However, calling
`connect_to_mixer_output()` on every file reload is a logic error. The correct
pattern: co-locate the wiring inside the `if deck_opt.is_none()` block that
creates the `AudioDeck`. Since `MixerAudio` is always created in the block just
before, it is guaranteed to exist at that point. This ensures the audio graph
topology is established exactly once per deck lifetime.

### Extract pure math helpers for testability (same pattern as `pitch_to_rate`)

The equal-power crossfader formula (`cos(val·π/2)`, `sin(val·π/2)`) cannot be
tested natively if it lives inside a method that takes `&GainNode` (a `web-sys`
type requiring a browser). Extract it as a `pub(crate) fn crossfader_gains(val:
f32) -> (f32, f32)` so the math properties (boundary values, constant-power
invariant, symmetry) can all be covered with plain `#[test]` functions. Reserve
`#[wasm_bindgen_test]` for node construction and actual `AudioParam` updates.

---

## Lessons from Feature 001 — Scratch Realism Improvement

### Track actual buffer state, not gesture direction, in position integrators

When integrating a running position estimate during scratch, branch on the actual
playback state (`scratch_in_reverse`) not on the user's gesture direction
(`d_angle < 0`). If a reversed buffer is unavailable, the forward buffer
continues playing forward even during a backward drag — using gesture direction
would subtract from position while audio moves forward. This pattern applies to
any stateful audio gesture that may have a "no-op fallback path".

### Reversed AudioBuffer seek offset is `duration - position`, not `position`

When swapping from a forward buffer at position T to a reversed buffer, the
correct start offset into the reversed buffer is `duration - T`. The reversed
buffer plays index 0 (= the last original frame) first. Starting at `T` would
land at a completely unrelated point in the track. Easy to get backwards.

### AudioBuffer.clone() in web-sys is a cheap JS reference copy

`AudioBuffer` is a JS wrapper type. `.clone()` copies the JS object handle, not
the PCM audio data (~50 MB per 5-minute stereo track). Use `.clone()` freely to
satisfy the borrow checker when passing buffers out of `self` fields into
expressions that also need `&mut self`. No performance concern.

---

## Lessons from Feature 002 — Compact Responsive Layout

### `Closure::forget()` must be conditional on successful `set_timeout`

When creating a `Closure::<dyn FnMut()>` to pass to
`set_timeout_with_callback_and_timeout_and_arguments_0`, the `.forget()` call
**must** be inside the `if let Ok(handle) = ...` success branch — not after it
unconditionally. If `set_timeout` fails (returns `Err`) and `.forget()` is
still called, the closure leaks permanently: the browser never invokes it, so
the allocation is never reclaimed, and on every subsequent resize event another
closure leaks. The rule: only forget a closure when the browser has taken
ownership of it by accepting the timeout registration.
