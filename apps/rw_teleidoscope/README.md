# rw_teleidoscope

A browser-based interactive kaleidoscope app built with **Rust + Leptos + WebGL**, compiled to **WebAssembly**. Upload a photo (or capture from your camera), then explore symmetrical, psychedelic patterns in real time using mirror segments, visual effects, and color transforms — all running entirely client-side with no server, no uploads, and no accounts. The UI has a steampunk aesthetic.

## Prerequisites

| Tool | Purpose |
|---|---|
| [Rust toolchain](https://rustup.rs/) | Compiler + cargo |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| [Trunk](https://trunkrs.dev/) | Dev server + WASM bundler — `cargo install trunk` |
| [wasm-pack](https://rustwasm.github.io/wasm-pack/) | WASM browser tests — `cargo install wasm-pack` |
| Firefox or Chrome (headless) | Required for `wasm-pack test` |

## Common Commands

All commands run from this directory.

### Run the dev server

```sh
trunk serve
```

Opens at `http://localhost:8080/rw_teleidoscope/`. Hot-reloads on Rust source or shader changes.

### Build commands (via make.py)

```sh
python make.py build    # debug WASM build
python make.py dist     # release build → dist/
python make.py test     # cargo test (native) + wasm-pack test --headless --firefox
python make.py lint     # cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

### Run a single unit test

```sh
cargo test <test_name>
```

## Project Layout

```
src/
  lib.rs              # WASM entry point (#[wasm_bindgen(start)])
  app.rs              # Root Leptos component; provides global state via context
  routing.rs          # Hash-based client-side routing (no leptos_router)
  state/              # KaleidoscopeParams + AppState (all RwSignals)
  components/         # Leptos components (Header, ControlsPanel, CanvasView, …)
  renderer/           # WebGL 2 via glow: context, shaders, texture, uniforms, draw
  camera.rs           # getUserMedia, capture frame, release camera
  utils.rs            # Shared helpers (resize_to_800, math functions)
assets/
  shaders/
    vert.glsl         # Full-screen quad vertex shader
    frag.glsl         # Kaleidoscope fragment shader (all effects in one pass)
style/
  main.css            # Steampunk CSS custom properties and component styles
doc/
  prd.md              # Product requirements document
  tech_spec.md        # Technical specification
  wireframes.md       # ASCII wireframes for all UI states
  project_plan.md     # Milestone overview and dependency order
  lessons.md          # Lessons learned during development
  milestones/         # Per-milestone task lists and manual test checklists
```

## Technology Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2021) |
| UI framework | [Leptos 0.8](https://leptos.dev/) (CSR) |
| Compilation target | WebAssembly (`wasm32-unknown-unknown`) |
| Bundler | [Trunk](https://trunkrs.dev/) |
| GPU rendering | WebGL 2 via the [`glow`](https://crates.io/crates/glow) crate |
| Deployment base | `/rw_teleidoscope/` |

## Documentation

See the `doc/` directory for full design and implementation docs:

- **[PRD](doc/prd.md)** — what the app does and why
- **[Tech Spec](doc/tech_spec.md)** — how it's built
- **[Wireframes](doc/wireframes.md)** — UI layout reference
- **[Project Plan](doc/project_plan.md)** — milestones and progress
- **[Lessons Learned](doc/lessons.md)** — gotchas and hard-won insights
