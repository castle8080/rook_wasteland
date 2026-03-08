# rw_sixzee

A solitaire 6-column Sixzee game for the browser. Built with **Rust**, **Leptos 0.8 (CSR)**, and compiled to WebAssembly via **Trunk**. Part of the [Rook Wasteland](../../README.md) monorepo.

All game logic, state persistence, and advisor computation run entirely client-side. No server is required beyond static file hosting. The game is deployed under `/rw_sixzee/`.

---

## What It Is

Standard Sixzee uses a single scorecard column — every decision is a permanent commitment with no room to recover. rw_sixzee plays the same game across **6 columns simultaneously**. Players optimize placement across 78 cells, balancing short-term opportunity against long-term column structure. Each column earns its own upper-section bonus independently. Bonus Sixzee rolls award points without consuming a cell, so games with strong Sixzee luck run longer than 78 turns.

Grandma watches every game. She has opinions.

---

## Project Layout

```
apps/rw_sixzee/
├── Cargo.toml              # crate root (cdylib + rlib)
├── Trunk.toml              # Trunk build config (public_url = /rw_sixzee/)
├── make.py                 # build / test / dist / lint helpers
├── index.html              # Trunk entry point
├── src/
│   ├── lib.rs              # WASM entry, #[wasm_bindgen(start)], cfg gates
│   ├── app.rs              # root App component, hash-router dispatch
│   ├── error.rs            # AppError, AppResult
│   ├── router.rs           # Route enum + hash-change listener
│   ├── state/
│   │   ├── game.rs         # GameState struct + turn lifecycle
│   │   ├── scoring.rs      # pure scoring functions (no Leptos)
│   │   ├── storage.rs      # localStorage read/write
│   │   └── quotes.rs       # Grandma quote bank + tier selection
│   ├── components/         # Leptos UI components
│   ├── dice_svg/           # themed SVG die face components
│   └── worker/             # Web Worker bridge for Ask Grandma (M7)
├── assets/
│   └── grandma_quotes.json # Grandma's quotes, served as a static file
├── style/
│   └── main.css            # flat BEM stylesheet + CSS theme variables
├── tests/
│   └── integration.rs      # WASM browser integration tests (wasm-pack)
├── generated/
│   └── v_col.rs            # committed auto-generated DP value table
├── offline/                # standalone DP precomputation tool (not shipped)
│   └── src/main.rs
└── doc/                    # project documentation (PRD, tech spec, milestones)
```

---

## Building and Running

All commands run from `apps/rw_sixzee/`. Python 3 and the Rust toolchain with `wasm32-unknown-unknown` target are required. Install [Trunk](https://trunkrs.dev/) and [wasm-pack](https://rustwasm.github.io/wasm-pack/) before starting.

| Command | What it does |
|---|---|
| `python make.py build` | Debug WASM build via `trunk build`. Output in `dist/`. |
| `python make.py dist` | Release WASM build via `trunk build --release`. Optimised for size (`opt-level = "z"`, LTO). |
| `python make.py lint` | `cargo clippy --target wasm32-unknown-unknown -- -D warnings`. Zero warnings enforced; `unwrap()` is banned. |
| `python make.py test` | `cargo test` (native unit tests) + `wasm-pack test --headless --firefox` (browser integration tests). |
| `python make.py help` | Print available targets. |

To serve locally after building:

```
trunk serve
```

The app is available at `http://localhost:8080/rw_sixzee/`.

### Running a single test

```bash
cargo test <test_name>
```

For example:

```bash
cargo test upper_bonus_boundary
cargo test detect_bonus_sixzee
```

---

## Linting

Clippy is configured to target `wasm32-unknown-unknown` (the actual compilation target) and denies all warnings, including `clippy::unwrap_used`. The lint config lives in `Cargo.toml`:

```toml
[lints.clippy]
unwrap_used = "deny"
```

`expect()` is permitted only at validated call sites — `use_context` panics, DP table `include!`, and checked dice arithmetic. See `doc/tech_spec.md §15` for the full policy.

---

## Grandma's Quotes

Grandma's dialogue is driven by `assets/grandma_quotes.json`, which is served as a static file at `/rw_sixzee/assets/grandma_quotes.json` (fetched at runtime, not embedded in the WASM binary). It is copied into `dist/` by Trunk via the `[[copy-dir]]` rule in `Trunk.toml`.

Before writing or reviewing any Grandma quote, read `doc/grandma_soul.md`. It is the authoritative voice guide — her personality, register, vocabulary, what she won't say, and the emotional tier system.

To add or edit quotes, edit `assets/grandma_quotes.json` directly. No build step is required; the file is fetched fresh on each page load. Do not regenerate or move this file without also updating the fetch path in `src/state/quotes.rs`.

---

## DP Precomputation (`offline/`)

The **Ask Grandma** advisor uses a precomputed expected-value table (`V_COL`) to evaluate board positions and recommend moves. This table is produced by a standalone Rust binary in `offline/` and committed to the repo as `generated/v_col.rs`. It is included at compile time in the Worker crate via Rust's `include!` macro — no file I/O at runtime.

### When to regenerate

Only regenerate `generated/v_col.rs` if the scoring rules change (i.e., if `src/state/scoring.rs` is modified). The committed file is valid for the current rules and there is no reason to re-run the solver otherwise.

### How to regenerate

```bash
cd apps/rw_sixzee/offline
cargo run --release
```

The solver runs in approximately 2–3 seconds and writes `generated/v_col.rs` directly. It prints a summary to stdout:

```
v_col[0]    = 229.638504  (expected ~230–280)
v_col[8191] = 0.000000    (must be 0.0)
v_col min   = 0.000000
v_col max   = 229.638504
YZ_BONUS_CORRECTION[0]  = 90.0 (n=6 open, not forfeited)
YZ_BONUS_CORRECTION[6]  = 0.0  (n=0 open, not forfeited)
YZ_BONUS_CORRECTION[7]  = 0.0  (forfeited)
All assertions passed. generated/v_col.rs is ready.
```

After regenerating, commit `generated/v_col.rs` along with the scoring rule change.

### What the solver produces

`generated/v_col.rs` contains two `pub const` arrays:

```rust
// Expected future score for an optimal single-column game starting with
// the given 13-bit fill pattern (bit i = 1 → row i is already scored).
pub const V_COL: [f32; 8192] = [ ... ];

// Additive correction for the Sixzee bonus pool.
// Index: (6 - n_sixzee_open) for forfeited=false, +7 for forfeited=true.
pub const YZ_BONUS_CORRECTION: [f32; 14] = [ ... ];
```

The solver also has its own test suite:

```bash
cd apps/rw_sixzee/offline
cargo test
```

---

## Documentation

Full project documentation lives in `doc/`:

| File | Contents |
|---|---|
| `doc/prd.md` | Product Requirements Document — user stories and acceptance criteria |
| `doc/tech_spec.md` | Technical Specification — stack, module layout, state architecture |
| `doc/wireframes.md` | Screen-by-screen UI wireframes and navigation flows |
| `doc/project_plan.md` | Milestone overview and status |
| `doc/milestones/` | Per-milestone deliverables, checklists, and implementation notes |
| `doc/lessons.md` | Hard-won bugs, browser quirks, and non-obvious solutions |
| `doc/grandma_soul.md` | Grandma's character voice guide |
