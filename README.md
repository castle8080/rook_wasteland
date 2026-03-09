# Rook Wasteland

A collection of small, silly, client-side web apps. No accounts. No backend. No purpose beyond wasting time in interesting ways.

**▶ [Play them here](https://rookwasteland.z5.web.core.windows.net/)**

---

## The Apps

### ♜ Chess
*Play chess against an AI with a personality problem.*

Three difficulty levels, each backed by a ridiculous persona delivering live in-game commentary. Easy, medium, or hard — they're all weird. Fully client-side game engine with move validation, piece rules, and board state managed entirely in the browser.

---

### 👾 Defender
*Shoot things. They shoot back.*

Classic arcade vertical shooter with wave-based enemies, boss fights every 5 rounds, and a power-up system (rapid fire, triple shot, laser). Enemies are procedurally drawn in pixel art. High scores persist locally. Built with no UI framework — just raw Canvas 2D rendering from Rust.

---

### 📜 Poetry
*Read poems. Record yourself. Cringe later.*

Browse public-domain poetry and record yourself reading aloud. Recordings are saved entirely in your browser via IndexedDB — no uploads, no servers, no audience. Replay them later. Export them if you dare.

---

### 🎚️ Mixit
*Spin records. Mix decks. Pretend you're a DJ.*

A dual-deck audio mixer running entirely in your browser. Load tracks, nudge the pitch, ride the crossfader. Features per-deck 3-band EQ, sweep filter, reverb, delay, flanger, and echo effects. BPM detection runs on file load so you can pretend you knew what you were doing all along.

---

### 🔭 Teleidoscope
*A hand-cranked engine of impossible symmetry.*

Load a photograph into this fine Victorian computational apparatus, adjust the brass fittings, and watch it fold your image into patterns no Euclidean geometry can explain. Toggle the Möbius mirror. Engage recursive reflection. Hit Randomize and pretend you meant for it to look like that.

---

### 🎲 Sixzee
*Stop trying to force luck on the dice.*

Six-column solitaire Sixzee. Seventy-eight cells, each one permanent. The dice don't negotiate. When you're completely stuck, Ask Grandma — she's backed by a precomputed dynamic programming table and knows exactly what you should do. She always has.

---

## Technology

All apps are written in **Rust** and compiled to **WebAssembly**, built with [Trunk](https://trunkrs.dev/). Every app is pure static files — no server-side rendering, no API, deployable anywhere.

### Common patterns across all apps

- **Rust → WASM**: All apps target `wasm32-unknown-unknown` via `wasm-bindgen`. Release builds use `opt-level = "z"` + LTO for minimal binary size.
- **Leptos 0.8 (CSR)**: Most apps use Leptos as the reactive UI framework in client-side rendering mode. Signals drive UI updates with no virtual DOM.
- **Web APIs via `web-sys`**: Browser capabilities (audio, canvas, storage, camera, workers) are accessed through `web-sys` bindings — no JavaScript glue code.
- **Hash-based routing**: Apps use hash routing for navigation, keeping them fully functional as static files without a server rewrite rule.
- **Local-first storage**: Persistent state lives in the browser. `localStorage` for lightweight key/value data, `IndexedDB` for binary blobs (recordings).
- **`make.py` build scripts**: Each app has a standardized `make.py` with consistent `build`, `serve`, `test`, and `dist` targets.

### Interesting per-app highlights

**Defender** skips Leptos entirely — it's pure `web-sys` against a Canvas 2D context. No reactive framework, just a game loop. This makes it the leanest binary in the repo.

**Mixit** does real-time audio signal processing through the Web Audio API graph. BPM detection uses spectral flux analysis + autocorrelation implemented in Rust with `rustfft`, running on the audio buffer at load time.

**Sixzee** uses a **Web Worker** to offload its "Ask Grandma" AI hint computation off the main thread. The worker is a separate WASM binary compiled from the same crate with a `worker` feature flag. The optimal strategy is backed by an 8,192-entry dynamic programming table precomputed by a standalone offline Rust solver and baked into the binary at compile time.

**Teleidoscope** renders through **WebGL 2** via the `glow` crate, using a two-texture FBO ping-pong scheme for recursive reflection. Depth `N` issues `N+1` draw calls. The canvas is created with `preserveDrawingBuffer: true` so image export captures the actual rendered frame.

**Poetry** is the most local-first app in the set — after the initial page load, everything happens in the browser. Audio is captured with `MediaRecorder`, stored as blobs in IndexedDB, and played back with a custom audio player. Nothing leaves the device.

---

## Structure

```
rook_wasteland/
├── apps/
│   ├── rw_chess/          # Chess vs. AI personas
│   ├── rw_defender/       # Arcade vertical shooter
│   ├── rw_index/          # Landing page / app launcher
│   ├── rw_mixit/          # Dual-deck DJ mixer
│   ├── rw_poetry/         # Poetry reader + voice journal
│   ├── rw_sixzee/         # Solitaire dice game
│   └── rw_teleidoscope/   # WebGL symmetry engine
├── rw_serve/              # Axum-based static file server (HTTPS, structured logging)
└── make.py                # Top-level build orchestration
```

`rw_index` is the landing page and links to everything else. `rw_serve` is a minimal Rust/Axum server used for production hosting — it just serves the built `dist/` directories as static files.

