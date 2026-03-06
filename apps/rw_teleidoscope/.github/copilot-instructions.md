# rw_teleidoscope — Copilot Instructions

A browser-based interactive kaleidoscope app. Leptos 0.8 (CSR) + Trunk WASM,
WebGL 2 via the `glow` crate for all rendering. Client-side only; no server, no
accounts, no data leaves the browser. Deployed under `/rw_teleidoscope/`.

---

## Project Documentation

All project documentation lives in `doc/`. **Always consult the relevant doc
before implementing a feature.** The docs are the source of truth for design
decisions — do not make architectural choices that contradict them without
updating the doc first.

| File | Purpose | When to read |
|---|---|---|
| `doc/prd.md` | Product Requirements Document — user stories, functional requirements (FR-1 to FR-24), UI layout, steampunk aesthetic, success criteria. All open questions are resolved. | Before implementing any user-facing feature — confirms *what* to build and *why*. |
| `doc/tech_spec.md` | Technical Specification — stack table, full `Cargo.toml`, `Trunk.toml`, module layout, state architecture, rendering pipeline + shader pseudocode, image input pipelines, component responsibilities, `Renderer` struct, camera API, CSS tokens, `make.py` template, testing strategy. All deferred design questions (D1–D3) are resolved. | Before writing any code — confirms *how* to build it. |
| `doc/wireframes.md` | ASCII wireframes for all 7 UI states: landing, main view, panel collapsed, effects detail, export dropdown, camera overlay, mirrors gauge. | When building or modifying any UI component — confirms layout and element placement. |
| `doc/project_plan.md` | Milestone overview — 10 milestones in dependency order, status summary, dependency tree. | To understand overall project progress and what is currently unblocked. |
| `doc/milestones/m1-scaffold.md` | Tasks and manual test checklist for M1: project scaffold. | When working on M1. |
| `doc/milestones/m2-webgl-renderer.md` | Tasks and manual test checklist for M2: WebGL canvas and basic renderer. | When working on M2. |
| `doc/milestones/m3-image-input.md` | Tasks and manual test checklist for M3: file picker, drag-and-drop, texture upload. | When working on M3. |
| `doc/milestones/m4-mirror-symmetry.md` | Tasks and manual test checklist for M4: polar coords, mirror fold, core controls, center drag. | When working on M4. |
| `doc/milestones/m5-visual-effects.md` | Tasks and manual test checklist for M5: spiral, ripple, lens, radial fold, Möbius, recursive reflection. | When working on M5. |
| `doc/milestones/m6-color-transforms.md` | Tasks and manual test checklist for M6: hue, saturation, brightness, posterize, invert. | When working on M6. |
| `doc/milestones/m7-camera-input.md` | Tasks and manual test checklist for M7: getUserMedia, preview overlay, capture, error state. | When working on M7. |
| `doc/milestones/m8-export.md` | Tasks and manual test checklist for M8: canvas.toBlob download, format selector, filename. | When working on M8. |
| `doc/milestones/m9-randomize.md` | Tasks and manual test checklist for M9: Surprise Me button, randomised parameter ranges. | When working on M9. |
| `doc/milestones/m10-steampunk-polish.md` | Tasks and manual test checklist for M10: full steampunk CSS, fonts, icons, collapsible panel. | When working on M10. |
| `doc/lessons.md` | Living record of non-obvious bugs, browser/crate quirks, shader corrections, and hard-won insights discovered during development. A memory aid for future work. | **Before starting work in any area** — check for existing lessons. **After resolving any non-trivial issue** — add a new lesson. |

### How to use the milestone docs

Each milestone doc contains:
- A **task table** with status (`⬜ Pending`, `🔄 In progress`, `✅ Complete`, `🚫 Blocked`).
  Update statuses as work progresses.
- A **manual test checklist** — every checkbox must be ticked before the milestone is
  considered done.
- **Implementation notes** — watch for gotchas specific to that milestone.

When starting a milestone: read the milestone doc first, then read the relevant
sections of `tech_spec.md` (cross-referenced in the notes), then implement.

When a task is complete: mark it `✅` in the milestone doc. When all tasks and
all manual test checklist items pass, mark the milestone `✅` in `project_plan.md`.

If any non-trivial issue was encountered during the milestone, add a lesson to
`doc/lessons.md` before closing out the milestone.

---

## Build, Test, and Lint

Commands run from the **`apps/rw_teleidoscope/` directory**:

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

`Trunk.toml`: `public_url = "/rw_teleidoscope/"` — all asset paths (WASM, JS glue,
CSS, shader files) are injected as absolute paths under this prefix. GLSL shader
files in `assets/shaders/` are declared with `<link data-trunk rel="copy-dir">` in
`index.html` and fetched at WASM startup.

---

## Architecture

```
src/
├── lib.rs              # WASM entry point; #[wasm_bindgen(start)]; lint attrs
├── app.rs              # Root App component; provides KaleidoscopeParams + AppState via context
├── routing.rs          # Hash-based Route enum (no leptos_router)
├── state/
│   ├── params.rs       # KaleidoscopeParams — all RwSignals for renderer uniforms
│   └── app_state.rs    # AppState — image_loaded, camera_open, camera_error, panel_open
├── components/
│   ├── header.rs       # Title + Load Image + Use Camera buttons
│   ├── controls_panel.rs  # Collapsible panel; all sliders, toggles, action buttons
│   ├── canvas_view.rs  # <canvas>; WebGL init; signal→redraw Effect; pointer drag for center
│   ├── camera_overlay.rs  # getUserMedia overlay; capture; error state
│   └── export_menu.rs  # Format picker + canvas.toBlob download
├── renderer/
│   ├── context.rs      # Obtain WebGL2 context from canvas NodeRef; wrap with glow
│   ├── shader.rs       # Fetch .glsl files; compile/link program
│   ├── texture.rs      # Upload ImageData → GL texture
│   ├── uniforms.rs     # Uniform location cache; upload KaleidoscopeParams → GPU
│   └── draw.rs         # draw() — bind texture, upload uniforms, draw quad
├── camera.rs           # request_camera(), capture_frame(), release_camera()
└── utils.rs            # Shared helpers (resize_to_800, math functions)
```

**`Cargo.toml` crate type:**
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```
`rlib` enables `cargo test` on the native host for pure-logic unit tests.
`cdylib` is the WASM output.

---

## State Architecture

`KaleidoscopeParams` and `AppState` are structs of `RwSignal`s, created in `App`
and provided via Leptos context. All UI controls read/write signals directly.

`Renderer` (`glow::Context` is `!Send`) is stored as
`Rc<RefCell<Option<Renderer>>>` in `CanvasView` — **never inside a Leptos signal**.

A single `Effect` in `CanvasView` calls `params.snapshot()` (reads all signals,
registering them as reactive deps) and passes the snapshot to `renderer.draw()`.
This is the only render trigger — no continuous rAF loop unless animation mode
is added later.

---

## Rendering Pipeline

Full-screen quad (two triangles, uploaded once). Fragment shader does all work:

```
polar coords → lens warp → ripple → spiral → mirror fold → Möbius → radial fold
  → texture sample → hue → saturation → brightness → posterize → invert → output
```

Recursive reflection (depth 1–3): render main pass to an FBO, re-bind output
as `u_image`, repeat. Final pass renders to the default framebuffer.

GLSL shaders live in `assets/shaders/vert.glsl` and `assets/shaders/frag.glsl`.
Fetched at startup via `fetch()` (not `include_str!`).

---

## Rust / Error Handling

- **Never `.unwrap()`** in non-test code. Use `.expect("why this cannot fail")`.
  The reason must explain *why* the failure is impossible.
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
let value = RwSignal::new(0);        // Send + Sync types
let local = RwSignal::new_local(x);  // !Send web-sys types only
```

### Components

```rust
#[component]
fn MyComponent(value: i32, #[prop(optional)] label: Option<String>) -> impl IntoView {
    view! { <div>{value}</div> }
}
```

### on:input

```rust
// Uses web_sys::Event (not InputEvent)
on:input=move |ev| {
    let val = ev.target().unchecked_into::<HtmlInputElement>().value();
}
```

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

**Hand-coded hash routing only — do not use `leptos_router`.**
Hash routing (`#/route`) keeps navigation in the URL fragment, which is never
sent to the server. Required for static file hosting.

App-lifetime `gloo_events::EventListener` (e.g. for `hashchange`) must be kept
alive with `std::mem::forget(listener)`.

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

## web-sys API Notes

- `on:input` → `web_sys::Event`, not `InputEvent`. Read with
  `.target().unchecked_into::<HtmlInputElement>().value()`.
- Canvas 2D: `ctx.set_fill_style_str("colour")` — not the deprecated
  `set_fill_style(&JsValue)`.
- `translate()`, `rotate()`, `arc()` return `Result<(), JsValue>` — always
  `.expect()` them.
- `PointerEvent` listeners on `<canvas>` for center drag must call
  `event.prevent_default()` to block text selection / scroll.
- `gloo_events::EventListener` for drag-and-drop and pointer events must be
  kept alive with `std::mem::forget` (app-lifetime listeners).
- `js_sys::Math::random()` returns `f64` in `[0, 1)` — use this for all
  randomization (no `std::time`, not available in WASM).
- For date in filenames: `js_sys::Date::new_0()` — not `std::time`.

---

## Clippy

- `#[allow(clippy::...)]` always requires an explanatory comment.
- State structs with `fn new()` must also `impl Default` delegating to `Self::new()`.
- Run clippy against `wasm32` target — the native and WASM targets can diverge.

---

## Testing

- **Pure functions** (math, fold algorithm, color transforms) → `#[cfg(test)]`
  module in the same file, run with `cargo test`.
- **Browser-dependent** (WebGL init, texture upload, camera) → `#[wasm_bindgen_test]`
  in `tests/`, run with `wasm-pack test --headless --firefox`.
- Implement shader math as pure Rust functions in `utils.rs` first, test them
  natively, then port verbatim to GLSL. Never test math only in the shader.
- `.unwrap()` is acceptable inside `#[test]` functions.

---

## Task Workflow

Before writing code for any non-trivial task:

1. Check `doc/lessons.md` for any recorded lessons relevant to the area being worked on.
2. Read the relevant milestone doc (`doc/milestones/mN-*.md`) and note the task number.
2. Read the relevant sections of `doc/tech_spec.md`.
3. Write a design sketch (data flow, function signatures, edge cases).
4. Implement + write tests.
5. Run `python make.py lint` and `python make.py test` — both must pass.
6. Every public `fn`/`struct`/`trait` needs a `///` doc comment; magic numbers need named constants.
7. Mark the task `✅` in the milestone doc.
8. Commit.

### Commit message format

```
M4.3: implement mirror fold in fragment shader

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Format: `M<milestone>.<task>: <imperative description>`. Co-authored-by trailer on every commit.
