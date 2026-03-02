# Task 03: Ship Size + Lives System Overhaul

## Goals

1. **Double player ship visual size** — ship was 16×16 at scale=1 (tiny vs 40×32 enemies); now 16×16 at scale=2 = 32×32 visual
2. **5 starting lives** instead of 3
3. **Ground explosion mechanic** — enemy reaching bottom triggers a blast (radius = 30% screen width = 192px) that can kill the player and nearby enemies; no instant game over
4. **ExtraLife power-up** — new 7th power-up type; drops at 2% rate; grants +1 life (max 9)

## Spec Changes

- `doc/defender_spec.md` section 2.1.3: lives=5, new "Ground Explosion" subsection
- `doc/defender_spec.md` section 2.1.4: added ExtraLife as power-up type 7
- `doc/defender_spec.md` section 2.5.1: documented 2× ship scale
- `doc/defender_spec.md` section 3.5.2: updated collision pairs

## Implementation Checklist

### A. Player ship size (2×)
- [x] `src/graphics/sprite.rs` — `SpriteAtlas::new()`: add `.with_scale(2)` to `player` and `player_thrust`
- [x] `src/game.rs` — `Player::new()`: hitbox updated to `Rect::new(2.0, 2.0, 28.0, 28.0)` (inset on 32×32 visual)
- [x] `src/game.rs` — `PLAYER_START_X = CANVAS_W / 2.0 - 16.0` (center on 32px wide sprite)
- [x] `src/game.rs` — `PLAYER_START_Y = CANVAS_H - 48.0` (keeps 16px gap from bottom)
- [x] `src/game.rs` — position clamp updated to `CANVAS_W - 32.0`
- [x] `src/game.rs` — bullet spawn X offset updated to `+14.0` (center of 32px sprite)

### B. Entity types
- [x] `src/entities/entity.rs` — `EntityType::GroundExplosion` added
- [x] `src/entities/entity.rs` — `PowerUpType::ExtraLife` added

### C. Starting lives
- [x] `src/game.rs` — `Player::new()`: `lives: 5`

### D. Ground explosion
- [x] `src/game.rs` — `GROUND_EXPLOSION_RADIUS = 192.0` constant
- [x] `src/game.rs` — `GROUND_EXPLOSION_DURATION = 0.8` constant
- [x] `src/game.rs` — `spawn_ground_explosion_at(pos)` method
- [x] `src/game.rs` — `update_enemies()`: replace 3× `GameState::GameOver` with `spawn_ground_explosion_at()`
- [x] `src/game.rs` — `update_ground_explosions()`: tick lifetime, deactivate at 0
- [x] `src/game.rs` — `check_collisions()`: GroundExplosion branch — player radius check + enemy radius check
- [x] `src/game.rs` — `render_game()`: render GroundExplosion as large orange/yellow flash with alpha fade

### E. ExtraLife power-up
- [x] `src/game.rs` — `spawn_powerup_at()`: 7-way roll including ExtraLife (at index 6)
- [x] `src/game.rs` — `check_collisions()`: ExtraLife collected → `player.lives = (player.lives + 1).min(9)`
- [x] `src/game.rs` — `powerup_index()` & `powerup_colors`: entry for ExtraLife (bright magenta)
- [x] `src/game.rs` — `powerup_duration()`: ExtraLife returns 0.0 (instant)
- [x] `src/graphics/sprite.rs` — powerup color for ExtraLife added to atlas

### F. Tests
- [x] `src/entities/entity.rs` — test: GroundExplosion entity type exists
- [x] `src/game.rs` (or math.rs) — test: ground explosion radius constant = 192
- [x] `src/entities/entity.rs` — test: ExtraLife powerup type exists

### G. Lint & test
- [x] `cargo clippy --target wasm32-unknown-unknown` — 0 warnings
- [x] `cargo test` — all tests pass

## Constants Added

| Constant | Value | Notes |
|----------|-------|-------|
| `GROUND_EXPLOSION_RADIUS` | 192.0 | 30% of 640px canvas width |
| `GROUND_EXPLOSION_DURATION` | 0.8 s | Blast window + visual effect |
| `PLAYER_START_X` | `CANVAS_W/2 - 16` | Centers 32px-wide ship (was -8) |
| `PLAYER_START_Y` | `CANVAS_H - 48` | 16px gap from bottom (was -40) |

## Design Decisions

- **No score for collateral kills** — enemies caught in a ground explosion award 0 points; the explosion is a penalty/hazard, not a reward
- **Player must be alive to be hit** — ground explosion collision check respects existing invulnerability timer
- **GroundExplosion uses entity.lifetime** — `lifetime` counts down from `GROUND_EXPLOSION_DURATION`; collision + rendering active while `lifetime > 0`
- **ExtraLife is NOT a weapon slot** — collected like Shield (instant apply, no `active_powerup` slot consumed)
- **HUD lives cap** — display is `self.player.lives.min(9)` so icons don't overflow
