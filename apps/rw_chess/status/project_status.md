# Project Status & Decisions

## Current State
- Planning phase complete
- Leptos technical design doc created
- Task list defined (15 tasks)
- No code written yet

## Key Decisions

### Language & Tooling
- **Rust** compiled to **WebAssembly** via `wasm32-unknown-unknown`
- **Leptos 0.8** (latest) with **CSR mode** via Trunk
- No SSR — game is fully client-side; no server needed
- **Trunk** for hot-reload dev server and WASM bundle

### Architecture
- Four decoupled layers: UI, State, Rules, Engine
- Game state shared via `provide_context` / `expect_context`
- Engine runs in `spawn_local` async task to avoid blocking UI
- Pure Rust for rules and engine — no JS interop in logic layer

### Engine
- Alpha-beta minimax with iterative deepening
- Piece-square tables for positional heuristics
- Move ordering: captures first (better pruning)
- Difficulty by search depth: Easy=2, Medium=4, Hard=6

### UI / Visual
- Unicode chess glyphs (♔♕♖♗♘♙ / ♚♛♜♝♞♟)
- Classic board colors: `#f0d9b5` light, `#b58863` dark
- Last move highlighted in yellow
- Engine move highlighted in pulsing blue for 1.5s (prevents missing the move)
- Valid move indicators: semi-transparent dot overlay on squares

### Dependencies (planned)
- `leptos = "0.8"` with `csr` feature
- `leptos_meta = "0.8"`
- `leptos_router = "0.8"`
- `gloo-timers` for animation delays
- `wasm-bindgen` (transitively via leptos)

## Open Questions
- Pawn promotion: auto-queen or show promotion dialog?
  → Plan: show a simple modal with piece choice (Q/R/B/N)
- Draw conditions: implement 50-move rule and threefold repetition?
  → Plan: implement 50-move rule; skip threefold for v0.1
- Sound effects?
  → Plan: out of scope for v0.1

## Next Task to Start
**task-01-scaffold**: Project Scaffold
