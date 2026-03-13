# rw_mixit — Copilot Instructions

A Leptos 0.8 (CSR) + Trunk WASM app. Client-side only; deployed as static files
with no server-side rendering or backend API. Part of the Rook Wasteland monorepo
but developed independently. Deployed under `/rw_mixit/`.

Browser-based DJ mixing tool: dual-deck turntable UI, Web Audio API audio graph,
Canvas 2D animations (spinning platters, scrolling waveforms), spectral-flux BPM
detection. Cartoon hip-hop aesthetic — bold outlines, saturated colors, keyframe
animations.

---

## Project Documentation

All project documentation lives in `doc/`. **Always consult the relevant doc
before implementing a feature.** The docs are the source of truth for design
decisions — do not make architectural choices that contradict them without
updating the doc first.

| File | Purpose | When to read |
|---|---|---|
| `doc/rw_mixit_spec.md` | Product Requirements — features, controls, audio architecture diagram, visual design, keyboard shortcuts, resolved decisions. | Before implementing any user-facing feature. |
| `doc/rw_mixit_tech_spec.md` | Technical Specification — stack, module layout, state architecture, Web Audio API graph, all DSP algorithms (BPM, reverb IR, echo, flanger, stutter, scratch, VU meter), canvas rendering, component tree. | Before writing any code. |
| `doc/ascii_wireframes.md` | Screen-by-screen ASCII wireframes — deck layout, mixer panel, component hierarchy. | Before implementing any UI component. |
| `doc/implementation_plan.md` | Milestone overview and task checklist. | To understand overall project progress; update task status as you go. |
| `doc/implementation_lessons_and_notes.md` | Living record of non-obvious bugs, web-sys quirks, Web Audio API gotchas, and hard-won insights discovered during development. | **Before starting work in any area.** **After resolving any non-trivial issue** — add a new lesson. |
| `doc/leptos_technical_design_principles_and_api_practices.md` | Leptos-specific patterns and best practices for this app. | Before implementing any Leptos component or reactive logic. |
| `doc/rust_code_principles.md` | Rust coding standards and error handling patterns used in this codebase. | Before writing any Rust code. |
| `doc/task_workflow.md` | Step-by-step process for each development task. | Before starting any non-trivial task. |

---

## Build, Test, and Lint

Commands run from the **`apps/rw_mixit/` directory**:

```bash
python make.py build    # debug WASM build (trunk build)
python make.py test     # cargo test (native) + wasm-pack test --headless --firefox
python make.py dist     # release build → dist/
python make.py lint     # cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

Run a single unit test by name:
```bash
cargo test <test_name>
```

Lint with zero-warnings policy:
```bash
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

Required toolchain: `rustup target add wasm32-unknown-unknown`, `cargo install trunk`,
`cargo install wasm-pack` (for browser tests).

`Trunk.toml`: `public_url = "/rw_mixit/"` — all asset paths (WASM, JS glue, CSS)
are injected as absolute paths under this prefix.

---

## Architecture

```
src/
├── lib.rs              # WASM entry point; #[wasm_bindgen(start)]; lint attrs
├── app.rs              # Root App component; provides route + audio context
├── routing.rs          # Hash-based Route enum (Main / Settings / About)
├── state/
│   ├── deck.rs         # DeckState — all RwSignals for one deck
│   └── mixer.rs        # MixerState — crossfader, master volume, BPM, sync
├── audio/
│   ├── context.rs      # AudioContext lazy init (requires user gesture)
│   ├── deck_audio.rs   # AudioDeck — Web Audio node graph per deck
│   ├── mixer_audio.rs  # MixerAudio — crossfader GainNodes + master GainNode
│   ├── bpm.rs          # BPM detection: spectral flux + autocorrelation
│   └── loader.rs       # File → ArrayBuffer → AudioBuffer; waveform peak extraction
├── canvas/
│   ├── raf_loop.rs     # requestAnimationFrame driver — shared loop for both decks
│   ├── platter_draw.rs # Vinyl platter drawing (rotation, grooves, tonearm)
│   └── waveform_draw.rs # Waveform + playhead + loop region + hot cue markers
├── components/
│   ├── deck.rs         # <Deck> — left or right deck shell
│   ├── mixer.rs        # <Mixer> — center panel
│   ├── controls.rs     # Play/Cue/Stop/Nudge buttons
│   ├── pitch_fader.rs  # Tempo slider (vinyl-mode only)
│   ├── eq.rs           # 3-band EQ knobs
│   ├── hot_cues.rs     # 4 hot cue buttons
│   ├── loop_controls.rs # Loop In/Out/Toggle/bar shortcuts
│   ├── fx_panel.rs     # Echo/Reverb/Flanger/Stutter/Scratch toggles
│   └── header.rs       # Logo + nav
└── utils/
    └── mod.rs          # Keyboard shortcuts, misc helpers
```

**`Cargo.toml` crate type:**
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```
`rlib` enables `cargo test` on the native host for pure-logic unit tests (BPM math,
peak extraction, EQ formulas, routing). `cdylib` is the WASM output.

---

## State Architecture

### Two Worlds: Reactive vs. Imperative

| World | Technology | Examples |
|---|---|---|
| **Reactive UI** | Leptos signals + `view!` macro | Volume fader, play button, EQ knobs |
| **Imperative Canvas Loop** | `requestAnimationFrame` closure | Platter rotation, waveform playhead scroll |

The rAF loop reads signals with **`.get_untracked()`** — using `.get()` inside a
`Closure<dyn FnMut()>` creates spurious reactive subscriptions and is wrong.

### DeckState

All per-deck reactive state lives in `DeckState` (a struct of `RwSignal`s, created
once per deck and distributed via props). `web-sys` audio node objects are NOT
stored in signals — see `AudioDeck` below.

### AudioDeck (Web Audio nodes)

`AudioDeck` holds the actual `web-sys` audio graph. It is stored in
`Rc<RefCell<AudioDeck>>` — **not** a Leptos signal. Leptos `Effect`s bridge
reactive signals to imperative node updates:

```rust
Effect::new(move |_| {
    let vol = deck_state.volume.get();
    audio_deck.borrow().channel_gain.gain().set_value(vol);
});
```

### AudioContext Lazy Init

`AudioContext` requires a user gesture. Create it lazily on the first file-load or
play button click. Store in `Rc<RefCell<Option<AudioContext>>>` provided as context.
Apply current signal values immediately after creation — `Effect`s won't re-fire
because the signals did not change.

### Leptos Context Collisions

Leptos resolves `provide_context` / `use_context` by `TypeId`. Providing multiple
`RwSignal<bool>` values overwrites each other. Wrap each in a unique newtype:
```rust
struct IsPlayingA(pub RwSignal<bool>);
struct IsPlayingB(pub RwSignal<bool>);
```

---

## Rust / Error Handling

- **Never `.unwrap()`** in non-test code. Use `.expect("why this cannot fail")`.
  The reason must explain *why* the failure is impossible, not just what failed.
- Inside `spawn_local` callbacks where errors cannot be propagated, log to console:
  ```rust
  web_sys::console::error_1(&format!("Failed: {:?}", e).into())
  ```
- Enable in `lib.rs`:
  ```rust
  #![warn(clippy::unwrap_used)]
  #![warn(clippy::todo)]
  ```
- No `unsafe`. No `todo!()` or `unimplemented!()` in committed code.

---

## Leptos 0.8 Patterns

### Signals

```rust
let value = RwSignal::new(0);           // Send + Sync types only
let local = RwSignal::new_local(x);     // !Send web-sys types (AudioBuffer, etc.)
```

### Components

```rust
#[component]
fn MyComponent(value: i32, #[prop(optional)] label: Option<String>) -> impl IntoView {
    view! { <div>{value}</div> }
}
```

### Reactive attributes — must use `move ||` closures

Any attribute expression that reads a signal **must** be a `move ||` closure.
A plain `format!()` or conditional runs outside reactive tracking; Leptos warns at
runtime and the attribute goes permanently stale.

### Context

```rust
provide_context(my_signal);                        // ancestor
let s = expect_context::<RwSignal<MyType>>();      // descendant
```

### Async

```rust
use leptos::task::spawn_local;
spawn_local(async move { ... });   // WASM is single-threaded; no tokio::spawn
```

---

## Routing

**Hash-based routing only — do not use `leptos_router`.**

Routes: `#/` (Main deck view), `#/settings`, `#/about`.

App-lifetime listeners (`hashchange`, `keydown`) must be kept alive with
`std::mem::forget(listener)` — gloo `EventListener` removes the browser listener on
drop.

---

## WASM Entry Point

```rust
#[cfg(not(test))]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
```

Without `#[wasm_bindgen(start)]` the module starts silently — no error, no app.

---

## Web Audio API Notes

- `AudioBufferSourceNode` is **one-shot** — it cannot be restarted. Create a fresh
  node on every `play()` call. Store in `Option<AudioBufferSourceNode>`.
- `stop()` / `stop_with_when()` are deprecated in web-sys 0.3.91 with no
  non-deprecated replacement. Use `#[allow(deprecated)]` on the wrapping helper.
- `linear_ramp_to_value_at_time(value: f32, end_time: f64)` — value is `f32`,
  time is `f64`. Mixing them up causes a type error.
- **Echo feedback gain must stay strictly below 1.0** — values ≥ 1.0 cause
  exponential amplitude growth and browser AudioContext suspension.
- Use **`AudioParam.setValueAtTime`** (not `setInterval`) for the stutter gate —
  sample-accurate scheduling only.
- The flanger LFO must be started with `lfo.start()` before audio plays, or the
  delay time modulation is silent.
- For the VU meter, `AnalyserNode` should be placed post-EQ/post-FX, before the
  channel `GainNode`. Use `get_float_time_domain_data` for RMS level.
- Dry/wet bypass for reverb and echo: use parallel `GainNode`s with
  `linearRampToValueAtTime` over 20 ms — never disconnect the `ConvolverNode`
  mid-playback (causes clicks).

### Audio Graph Topology

```
[AudioBufferSourceNode]  ← .playbackRate (vinyl speed)
         │
    [GainNode]           ← pre-FX / stutter gate
         │
  [BiquadFilter HIGH]    ← highshelf @ 8 kHz
  [BiquadFilter MID]     ← peaking @ 1 kHz, Q=0.7
  [BiquadFilter LOW]     ← lowshelf @ 200 Hz
  [BiquadFilter SWEEP]   ← LP/HP sweep filter
         │
  [Reverb: ConvolverNode + dry/wet GainNodes]
  [Echo: DelayNode + feedback GainNode + wet/dry GainNodes]
  [Flanger: short DelayNode + OscillatorNode LFO + depth GainNode]
         │
    [GainNode]           ← channel volume fader
    [AnalyserNode]       ← VU meter tap (fftSize=256)
         │
    [GainNode (xfade)]   ← crossfader blend (equal-power cos/sin)
    [GainNode]           ← master volume
         │
  [AudioContext.destination]
```

---

## web-sys API Notes

- `on:input` → `web_sys::Event`, not `InputEvent`. Read value with
  `.target().unchecked_into::<HtmlInputElement>().value()`.
- Canvas 2D: use `ctx.set_fill_style_str("colour")` / `set_stroke_style_str(...)` —
  not the deprecated `set_fill_style(&JsValue)`.
- `ctx.save()` and `ctx.restore()` return `()` — do NOT call `.expect()` on them.
  Only `translate()`, `rotate()`, `arc()` return `Result<(), JsValue>` and need
  `.expect()`.
- `set_text_baseline`, `set_font`, `set_text_align`, `set_line_cap` accept `&str`
  directly — no `_str` suffix needed (unlike fill/stroke style).
- `draw_image_with_html_canvas_element(canvas, dx, dy)` is the correct 3-arg blit
  form. Do NOT use `..._and_dx_and_dy` — that variant does not exist.
- `gloo_events::EventListenerOptions` defaults to `passive: true`. Any listener
  calling `prevent_default()` must use `EventListenerOptions::enable_prevent_default()`.
- `js_sys::Math::random()` returns `f64` in `[0, 1)` — use for randomization
  (no `std::time` in WASM).
- `window().performance().unwrap().now()` → `f64` milliseconds. Requires
  `"Performance"` in web-sys features.

---

## Canvas / requestAnimationFrame

The rAF loop is started once on app mount via `spawn_local` (so `NodeRef` canvas
elements are available after first render). Pattern:

```rust
spawn_local(async move {
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        // use .get_untracked() on signals — .get() creates spurious subscriptions
        window().request_animation_frame(
            f.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap();
    }));
    window().request_animation_frame(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref()
    ).unwrap();
});
```

Two `on:` handlers on the same element (e.g. `on:mouseup` + `on:mouseleave` for
nudge-end) cannot share a single Rust closure. Wrap in `Rc<dyn Fn()>` and `.clone()`
for each handler.

### Platter Rotation

Rotation angle is stateless — computed fresh each frame:
```rust
let angle = current_secs * 0.55 * playback_rate * TAU;  // 33 RPM ≈ 0.55 rot/sec
```
No angle accumulator needed; f64 precision is fine for tracks up to 2 hours.

---

## BPM Detection

BPM detection runs once on file load. The algorithm:
1. **Spectral flux** on the bass frequency band (bins 1–32, ~43–1400 Hz) using
   `rustfft` with 1024-sample Hanning windows, 512-sample hop.
2. **Autocorrelation** over lags for BPM 60–200.
3. **Sub-lag correction**: if `r[lag/2] ≥ 0.90 × r[best_lag]`, prefer half the
   lag to correct double-period bias from integer rounding.

The core math (`compute_spectral_flux`, `estimate_bpm`) lives in `src/audio/bpm.rs`
as pure Rust functions testable with plain `cargo test`.

Integration tests in `tests/bpm_real_tracks.rs` decode MP3 files natively via
`symphonia` (dev-dep) and feed PCM into the same functions.

---

## Clippy

- `#[allow(clippy::...)]` always requires an explanatory comment.
- State structs with `fn new()` must also `impl Default` delegating to `Self::new()`.
- `start_raf_loop` takes >7 arguments; suppress `clippy::too_many_arguments` with a
  comment explaining the count. Refactor into a struct if count grows further.
- Run clippy against `wasm32` target — native and WASM targets can diverge.

---

## Testing

Three tiers. Every non-trivial task must include tests from the appropriate tier(s).

- **Tier 1 — native `#[test]`:** pure math (BPM formulas, `pitch_to_rate`,
  `crossfader_gains`, peak extraction, routing logic). No browser needed.
  Run with `cargo test`.
- **Tier 2 — `#[wasm_bindgen_test]` (low-level):** isolated web-sys API calls
  (AudioContext node construction, AudioParam updates). Files in `tests/`. Run with
  `wasm-pack test --headless --firefox`.
- **Tier 3 — `#[wasm_bindgen_test]` (integration):** full component trees mounted
  in headless Firefox; tests signal → DOM reactive wiring.

Each file under `tests/` needs its own `wasm_bindgen_test_configure!(run_in_browser);`.

`.unwrap()` is fine inside any `#[test]` function — a panic is a test failure.

Extract pure math helpers as standalone functions specifically so they can be
unit-tested natively (see `crossfader_gains`, `pitch_to_rate`, `estimate_bpm`).

---

## Task Workflow

Before writing code for any non-trivial task:

1. Check `doc/implementation_lessons_and_notes.md` for relevant lessons.
2. Read the relevant section of `doc/rw_mixit_tech_spec.md` and `doc/rw_mixit_spec.md`.
3. Write a design sketch (data flow, function signatures, edge cases).
4. Implement + write tests (see Testing section above for tier guidance).
5. **Coverage audit:** for every public function or component added or modified,
   list each meaningful behaviour and confirm it is tested or explicitly waived.
6. Run `python make.py lint` and `python make.py test` — both must pass.
7. Every public `fn`/`struct`/`trait` needs a `///` doc comment; magic numbers need
   named constants.
8. Stage changes and run the `code-review` agent — fix every flagged bug/logic error.
9. Mark the task complete in `doc/implementation_plan.md`.
10. Add any new lessons to `doc/implementation_lessons_and_notes.md`.
11. Commit.
12. Suggest a set of basic smoke tests to run.

### Commit message format

```
M2.3: implement crossfader equal-power blend

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Format: `M<milestone>.<task>: <imperative description>`. Co-authored-by trailer on
every commit.
