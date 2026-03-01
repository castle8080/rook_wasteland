# Task: Phase 1 - Core Engine

## Status: COMPLETE ✅

## Goal
Bootstrap the full project scaffold and implement the core game loop, rendering system, input handling, entity system, and sprite generation — enough to display a playable first wave.

## Scope
- [x] Cargo.toml, Trunk.toml, index.html, .gitignore
- [x] src/utils/math.rs — Vec2, Rect (AABB), Timer
- [x] src/graphics/colors.rs — RetroColors palette, color_to_css
- [x] src/graphics/sprite.rs — Sprite, AnimatedSprite, SpriteGenerator
- [x] src/systems/input.rs — InputState with keydown/keyup
- [x] src/entities/entity.rs — Entity, EntityType, PowerUpType, ActivePowerUp
- [x] src/renderer.rs — Renderer wrapping CanvasRenderingContext2d
- [x] src/game.rs — Game state machine, update + render loop
- [x] src/lib.rs — WASM entry point, RAF loop, event listeners
- [x] doc/principles_and_lessons.md — Rust/WASM best practices
- [x] 19/19 unit tests pass (cargo test)
- [x] 0 clippy errors/warnings (cargo clippy --target wasm32-unknown-unknown)
- [x] trunk build succeeds (391KB debug WASM)
- [x] Git committed

## Design Decisions
- Procedural sprites (no external assets) for Phase 1
- Single Entity struct with EntityType enum (not full ECS)
- Thread-local RefCell pattern for WASM game state
- InputState::consume_* pattern for one-shot keypresses
- Colors stored as 0xAARRGGBB u32
- Enemy fire uses deterministic pseudo-random (phase * golden_ratio) to avoid borrow conflicts with self.rng

## Notes
- Weapons module stubbed; full weapon mechanics in Phase 4.5
- Background system deferred to later phase (NASA images require asset pipeline)
- Boss entity type defined but not yet spawned (Phase 3+)
