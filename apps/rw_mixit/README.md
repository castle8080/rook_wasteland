# rw_mixit

A browser-based DJ mixing tool built with **Rust + Leptos**, compiled to **WebAssembly**. Dual-deck audio mixing runs entirely client-side via the Web Audio API — no server, no uploads, no accounts. The aesthetic is old-school hip-hop turntablism rendered as a chunky cartoon: animated vinyl platters, glowing VU meters, and tactile-feeling controls.

## Prerequisites

| Tool | Purpose |
|---|---|
| [Rust toolchain](https://rustup.rs/) | Compiler + cargo |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| [Trunk](https://trunkrs.dev/) | Dev server + WASM bundler — `cargo install trunk` |
| [wasm-pack](https://rustwasm.github.io/wasm-pack/) | WASM integration tests — `cargo install wasm-pack` |
| Chrome or Firefox (headless) | Required for `wasm-pack test` |

## Common Commands

### Run the dev server

```sh
trunk serve
```

Opens at `http://localhost:8080/rw_mixit/`. The page hot-reloads on Rust source changes.

### Build for production

```sh
trunk build --release
```

Output goes to `dist/`. The `public_url` is set to `/rw_mixit/` in `Trunk.toml` — adjust there if deploying to a different path.

### Unit tests (native, fast)

Runs all pure-Rust logic tests (BPM detection, peak extraction, crossfader math, routing, etc.) on the host without a browser.

```sh
cargo test
```

### WASM integration tests (browser)

Runs tests tagged `#[wasm_bindgen_test]` in a real browser via WebDriver. Requires Chrome or Firefox with a matching WebDriver binary on `PATH`.

```sh
# Chrome (headless)
wasm-pack test --headless --chrome

# Firefox (headless)
wasm-pack test --headless --firefox
```

### Lint

```sh
cargo clippy --target wasm32-unknown-unknown
```

The crate is configured with `#![warn(clippy::unwrap_used)]` and `#![warn(clippy::todo)]` — new `unwrap()` calls outside test code will produce warnings.

## Project Layout

```
src/
  lib.rs               # WASM entry point (#[wasm_bindgen(start)])
  audio/               # Web Audio API wrappers (context, deck, mixer, loader, BPM)
  canvas/              # rAF loop, waveform and platter canvas drawing
  components/          # Leptos components (App, Deck, Mixer, Controls, …)
  state/               # Reactive signals (DeckState, MixerState)
  routing.rs           # Hash-based client-side routing
doc/                   # Design docs and specs
tasks/                 # In-progress and completed implementation task notes
tests/                 # Native integration tests (BPM against real audio files)
```

## Technology Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| UI framework | [Leptos 0.8](https://leptos.dev/) (CSR / client-side rendering) |
| Compilation target | WebAssembly (`wasm32-unknown-unknown`) |
| Bundler | [Trunk](https://trunkrs.dev/) |
| Audio | Web Audio API via `web-sys` |
| Canvas | 2D Canvas API via `web-sys` |
| BPM detection | Spectral flux + autocorrelation (`rustfft`) |
