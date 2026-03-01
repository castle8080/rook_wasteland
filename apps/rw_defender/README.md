# RW Defender

A classic arcade-style shooter (Space Invaders / Galaga inspired) built in Rust compiled to WebAssembly, running in the browser via HTML5 Canvas.

## Prerequisites

- [Rust](https://rustup.rs/) with the `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) — Rust WASM bundler

```bash
# Install Rust target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk
```

## Running (Development)

```bash
cd apps/rw_defender
trunk serve
```

Then open **http://localhost:8081** in your browser.

Trunk watches for changes and rebuilds automatically.

## Building (Production)

```bash
cd apps/rw_defender
trunk build --release
```

Output goes to `dist/`. Serve the `dist/` folder with any static file server.

## Testing

```bash
cd apps/rw_defender
cargo test
```

## Linting

```bash
cd apps/rw_defender
cargo clippy --target wasm32-unknown-unknown
```

## Controls

| Key | Action |
|-----|--------|
| `←` / `A` | Move left |
| `→` / `D` | Move right |
| `Space` / `W` / `↑` | Fire |
| `Escape` / `P` | Pause |
| `Enter` | Start / restart |

## Project Structure

```
src/
├── lib.rs              # WASM entry point, game loop
├── game.rs             # Game state machine + main logic
├── renderer.rs         # Canvas 2D abstraction
├── entities/           # Entity, EntityType, PowerUpType
├── graphics/           # Sprite, SpriteGenerator, color palette
├── systems/            # Input handling
├── utils/              # Vec2, Rect (AABB), Timer
└── weapons/            # Weapon types (stub, Phase 4+)
```
