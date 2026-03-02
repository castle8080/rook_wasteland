# ♜ Rook Wasteland Chess

A browser-based chess game compiled to WebAssembly, part of the [Rook Wasteland](../../README.md) collection of client-side time-wasting apps.

Play against an AI opponent with personality — choose your difficulty and face off against one of three quirky engine personas, each with their own in-game commentary.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2024) |
| UI Framework | [Leptos](https://leptos.dev/) 0.8 (CSR / client-side rendering) |
| Compilation Target | WebAssembly (`wasm32-unknown-unknown`) |
| Build & Dev Server | [Trunk](https://trunkrs.dev/) |
| WASM Bindings | `wasm-bindgen`, `js-sys` |
| Async (timers) | `gloo-timers` |

The entire app runs in the browser with no backend required.

---

## Features

- **Full chess rules** — castling (kingside & queenside), en passant, pawn promotion, check, checkmate, stalemate, and the 50-move draw rule
- **AI engine** — iterative-deepening alpha-beta search with beam pruning and quiescence search
- **Three difficulty levels**:
  - **Easy** — search depth 2, plays as *Pawndrew, The Pawn Who Got Promoted By Accident* ♟
  - **Medium** — search depth 3, plays as *Prof. Pompington III, Author of 47 Books Nobody Has Read* 🎩
  - **Hard** — search depth 4, plays as *Grandmaster Goblin, Ancient Chess Gremlin Escaped From The Machine* 👺
- **Engine evaluation** — material balance, piece-square tables, mobility, pawn structure, king safety, rook file bonuses, bishop pair bonus
- **Live commentary** — each persona reacts to captures, checks, wins, and losses with character-specific quips
- **Move history panel** — scrollable record of all moves in algebraic notation
- **Captured pieces display** — shows material captured by each side
- **Play as either color** — if you choose Black, the engine opens as White

---

## Project Structure

```
src/
├── main.rs           # Entry point — mounts Leptos app to DOM
├── lib.rs            # Module declarations
├── engine/
│   ├── eval.rs       # Board evaluation (material, positional, pawn structure, king safety)
│   ├── movegen.rs    # Pseudo-legal move generation
│   ├── persona.rs    # AI personas and commentary system
│   └── search.rs     # Iterative-deepening alpha-beta + quiescence search
├── rules/
│   ├── validation.rs # Legal move filtering, check/checkmate/stalemate detection
│   └── special_moves.rs # Castling and en passant helpers
├── state/
│   ├── board.rs      # Board representation
│   ├── game.rs       # Reactive game state (Leptos signals)
│   └── piece.rs      # Piece, color, move, and difficulty types
└── ui/
    ├── app.rs        # Root component, engine-move trigger loop
    ├── board.rs      # Interactive board rendering
    ├── commentary_box.rs # Animated persona commentary bubble
    ├── controls.rs   # New game / resign buttons
    ├── game_over.rs  # End-of-game overlay (rematch / new game)
    ├── info_panel.rs # Move history and captured pieces
    ├── piece.rs      # Individual piece rendering
    ├── setup.rs      # Pre-game setup screen (name, color, difficulty)
    └── square.rs     # Board square component
```

---

## Prerequisites

Install Rust and the WebAssembly target:

```sh
rustup target add wasm32-unknown-unknown
```

Install the Trunk build tool:

```sh
cargo install trunk
```

---

## Commands

### Development server

Serves the app at `http://localhost:8080` with live reload:

```sh
trunk serve
```

### Production build

Compiles and bundles everything into the `dist/` directory:

```sh
trunk build --release
```

### Run tests

Runs all unit tests natively (no browser required):

```sh
cargo test
```

### Type check

```sh
cargo check
```

### Lint

```sh
cargo clippy
```

---

## Deployment

After `trunk build --release`, the `dist/` directory contains all static assets (HTML, CSS, WASM, JS glue). It can be served from any static file host — no server-side runtime needed.
