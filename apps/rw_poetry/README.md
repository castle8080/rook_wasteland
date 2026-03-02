# RW Poetry

A local-first poetry reader and voice-journal app built with Rust, Leptos, and WebAssembly.

## What It Does

- **Random poem discovery** — browse a curated collection of public-domain poems
- **Voice recording** — record yourself reading a poem directly in the browser
- **Local library** — save and revisit past recordings with poem metadata and timestamps
- **Playback and export** — replay recordings and download audio files
- **Offline-friendly** — runs entirely in the browser with no backend required at runtime

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (2024 edition) |
| Frontend Framework | [Leptos](https://leptos.dev) 0.8 (CSR / client-side rendering) |
| Runtime Target | WebAssembly (WASM) |
| Build Tool | [Trunk](https://trunkrs.dev) |
| Storage | IndexedDB (via [`idb`](https://crates.io/crates/idb)) |
| Audio | Browser Media APIs via [`web-sys`](https://crates.io/crates/web-sys) |
| HTTP | [`gloo-net`](https://crates.io/crates/gloo-net) |
| Routing | Hand-rolled hash routing (no `leptos_router`) |
| Poem corpus | Static JSON files under `public/poems/` |

## Project Layout

```
src/
  main.rs                  # Entry point — mounts the Leptos app
  app.rs                   # Root App component; hash routing setup
  routing.rs               # Route enum and hash parsing/formatting
  poem_repository/         # Fetch poem index and individual poem JSON
  recording_store/         # IndexedDB persistence for recordings
  audio_capture/           # Browser microphone and MediaRecorder bindings
  ui/
    components/            # Shared UI components (TopBar, etc.)
    reader.rs              # Poem reader view
    recording_controls.rs  # Record / stop / discard controls
    recordings_list.rs     # Browse saved recordings
    recording_detail.rs    # Single recording playback and export
    audio_player.rs        # Reusable audio player component

public/poems/
  poems_index.json         # Generated index of all poems (do not edit by hand)
  authors/<name>/<slug>.json  # Individual poem JSON files

scripts/
  build_poems_index.py     # Regenerates poems_index.json from the authors/ tree

tasks/                     # Per-task design documents (living log of decisions)
doc/                       # Architecture, spec, and Leptos usage guidance
style/
  main.css                 # Global stylesheet
```

## Prerequisites

- **Rust** (stable, 2024 edition) — install via [rustup](https://rustup.rs)
- **wasm32 target**: `rustup target add wasm32-unknown-unknown`
- **Trunk**: `cargo install trunk`
- **Python 3** (for the poem index script)
- **wasm-pack** (optional, for browser-targeted tests): `cargo install wasm-pack`

## Commands

### Development

```bash
# Start the dev server with live reload (default: http://localhost:8080)
trunk serve

# Fast compile check (run constantly while developing)
cargo check
```

### Building

```bash
# Production WASM build — output goes to dist/
trunk build --release

# Debug build
trunk build
```

### Testing

```bash
# Run all native unit tests
cargo test

# Run WASM-targeted tests in a headless browser
wasm-pack test --headless --chrome
# or
wasm-pack test --headless --firefox
```

### Linting and Formatting

```bash
# Lint (warnings are treated as errors)
cargo clippy -- -D warnings

# Auto-format
cargo fmt

# Check formatting without modifying files
cargo fmt --check
```

### Poem Corpus

```bash
# Rebuild poems_index.json after adding, renaming, or removing poem files
python3 scripts/build_poems_index.py
```

## Adding Poems

Poem files live under `public/poems/authors/<author_slug>/`. Each file is a JSON object:

```json
{
  "id": "unique-slug",
  "title": "Poem Title",
  "author": "Author Name",
  "content": "Full poem text...",
  "date": "1917",
  "source": "Optional source note",
  "tags": ["optional", "tags"]
}
```

After adding or modifying poem files, regenerate the index:

```bash
python3 scripts/build_poems_index.py
```

## Routes

The app uses hash-based routing so it works on any static file server:

| Hash | View |
|---|---|
| `#/` or `#/?poem_id=<id>` | Poem reader (random or specific poem) |
| `#/readings` | Recordings list |
| `#/readings/<id>` | Recording detail / playback |

## Pre-commit Checklist

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
trunk build
```

## Documentation

- `doc/poetry_spec.md` — full product and technical specification
- `doc/building.md` — development standards and workflow guide
- `doc/leptos_technical_design_principles_and_api_practices.md` — Leptos API usage patterns
- `tasks/` — per-task design documents
