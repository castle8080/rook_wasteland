# RW Defender

A classic arcade-style vertical shooter (Space Invaders / Galaga inspired) built in **Rust**, compiled to **WebAssembly**, and rendered in the browser via **HTML5 Canvas**. No JavaScript game framework — just raw Rust, `wasm-bindgen`, and `web-sys`.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (2021 edition) |
| Compile target | `wasm32-unknown-unknown` |
| WASM bindings | [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) |
| Browser APIs | [`web-sys`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/) (Canvas 2D, keyboard events, localStorage, requestAnimationFrame) |
| JS interop | [`js-sys`](https://rustwasm.github.io/wasm-bindgen/api/js_sys/) |
| RNG | [`rand`](https://docs.rs/rand) with `SmallRng` |
| Bundler | [Trunk](https://trunkrs.dev/) |
| Release profile | `opt-level = "z"` + LTO for minimal WASM binary size |

## Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) — Rust WASM bundler

```bash
# Add the WASM compile target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk
```

## Commands

### Development server

```bash
trunk serve
```

Opens a live-reload dev server at **http://localhost:8081**. Trunk watches for source changes and rebuilds automatically.

### Production build

```bash
trunk build --release
```

Outputs optimised static assets to `dist/`. Serve that directory with any static file server (e.g. `npx serve dist`).

### Run tests

```bash
cargo test
```

Unit tests run natively (no browser required). WASM integration tests can be run with `wasm-pack test --headless --chrome` if `wasm-bindgen-test` targets are added.

### Lint

```bash
cargo clippy --target wasm32-unknown-unknown
```

### Format

```bash
cargo fmt
```

## Gameplay

Waves of enemies descend in formation. Clear all enemies to advance. Every **5th wave** spawns a Boss. Enemies that reach the bottom trigger a **ground explosion** — a large area-of-effect blast that damages the player and nearby enemies.

### Enemy Types

| Enemy | Behaviour |
|-------|-----------|
| Grunt | Descends in formation, fires downward |
| Weaver | Sine-wave lateral movement |
| Diver | Dives straight down when the player is within range |
| Boss | Sine-wave horizontal patrol, 3-shot spread fire; appears every 5 waves |

Enemy speed scales +5% per wave, capped at 2×.

### Controls

| Key | Action |
|-----|--------|
| `←` / `A` | Move left |
| `→` / `D` | Move right |
| `Space` / `W` / `↑` | Fire |
| `Escape` / `P` | Pause / unpause |
| `Enter` | Start / restart |

### Power-Ups

Dropped randomly on enemy death. Only one weapon power-up is active at a time.

| Colour | Power-Up | Effect | Duration |
|--------|----------|--------|----------|
| Orange | Triple Shot | Fires 3 bullets in a spread | 12 s |
| Red | Explosive Shot | Bullets explode on impact | 15 s |
| Yellow | Rapid Fire | 3× fire rate | 12 s |
| Blue | Laser Beam | Continuous piercing beam | 10 s |
| Green | Piercing Shot | Bullets pass through enemies | 12 s |
| Cyan | Shield | Absorbs one hit (stacks up to 3) | 30 s |
| Magenta | Extra Life | +1 life instantly (max 9) | instant |

## Project Structure

```
apps/rw_defender/
├── src/
│   ├── lib.rs              # WASM entry point: canvas setup, input listeners, rAF game loop
│   ├── renderer.rs         # Canvas 2D drawing abstraction
│   ├── game/
│   │   ├── mod.rs          # Game struct, state machine (MainMenu → Playing → GameOver…)
│   │   ├── constants.rs    # All tuning constants + localStorage high-score helpers
│   │   ├── player.rs       # Player state, SpriteAtlas
│   │   ├── update.rs       # Per-frame game logic (movement, spawning, collisions)
│   │   └── render.rs       # HUD, entities, overlays rendering
│   ├── entities/
│   │   └── entity.rs       # Entity, EntityType, PowerUpType, ActivePowerUp
│   ├── graphics/
│   │   ├── sprite.rs       # Sprite, SpriteGenerator (procedural pixel art)
│   │   ├── background.rs   # StarField parallax layers, BackgroundTier per wave range
│   │   └── colors.rs       # RetroColors palette
│   ├── systems/
│   │   └── input.rs        # InputState — keyboard event → game actions
│   ├── utils/              # Vec2, Rect (AABB), Timer, LCG RNG helper
│   └── weapons/            # Weapon type stubs (future expansion)
├── assets/                 # Background space images
├── index.html              # HTML shell (two canvases: background + game)
├── Trunk.toml              # Trunk config (port 8081, dist/)
└── Cargo.toml
```

## Architecture Notes

- **Two-canvas compositing** — a background `<canvas>` renders the parallax starfield + space images; the game `<canvas>` sits on top for all entities and HUD. This avoids clearing the starfield every frame.
- **Entity-component lite** — all game objects are flat `Entity` structs with an `EntityType` discriminant. No ECS framework.
- **No heap allocations in hot path** — `Vec<Entity>` is pre-allocated with capacity 200; dead entities are flagged `active = false` and drained once per frame.
- **High score persistence** — saved/loaded via the browser's `localStorage` under the key `rw_defender_high_score`.
- **Deterministic RNG** — `SmallRng::seed_from_u64(42)` is used for enemy spawning; `getrandom` with the `js` feature satisfies WASM entropy requirements.

## Development Docs

- [`doc/defender_spec.md`](doc/defender_spec.md) — Full game design specification
- [`doc/principles_and_lessons.md`](doc/principles_and_lessons.md) — Rust/WASM best practices and lessons learned
- [`tasks/`](tasks/) — Phased task breakdowns and implementation notes

