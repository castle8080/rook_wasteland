# Task 2: Boss Enemies, Parallax Starfield, High Score Persistence

## Status: COMPLETE

## Goals
- [x] Boss enemy spawning (every 5 waves)
- [x] Boss sine-wave movement + 3-shot spread
- [x] Diver dive-attack behavior  
- [x] Procedural parallax starfield background
- [x] High score persistence via localStorage
- [x] 23/23 tests pass, 0 clippy warnings, trunk build succeeds

## Implementation

### Boss Enemies (game.rs)
- `begin_wave()` checks `self.wave.is_multiple_of(5)` → spawns 1 Boss
- Boss spawned at `(CANVAS_W/2 - 24, BOSS_Y=50)` — top-center
- Boss movement: sine wave, `x = center + 100 * sin(phase * 2π * 0.5Hz)`, fixed `y = 50`
- Boss fires 3-shot spread (±20°) at `BOSS_BULLET_SPEED = 250px/s` every `BOSS_FIRE_INTERVAL = 1.5s`
- Boss HP = `75 + (wave/5) * 15` (spec formula)
- Boss score = `100 + wave * 20` (spec formula)
- `is_regular_enemy()` helper excludes Boss from formation shift logic

### Diver Dive Attack (game.rs)
- `dive_timer > 0`: actively diving straight down at `DIVER_SPEED = 80px/s`
- Dive trigger: player within `±80px` horizontal AND `lifetime <= 0` (dive cooldown)
- Dive duration: `DIVER_RETURN_TIME = 2.0s`; after missed dive, entity deactivated
- Dive cooldown: `DIVER_DIVE_COOLDOWN = 3.0s` stored in `entity.lifetime`

### Parallax Starfield (src/graphics/background.rs)
- 3 depth layers: 80 slow/dim stars, 50 medium, 25 fast/bright
- Each layer scrolls downward at different speed; wraps at CANVAS_H
- `BackgroundTier` enum: Warm (waves 1-5), Nebula (6-10), Deep (11-15), UltraDeep (16+)
- Each tier has distinct star color temperature (warm red → cool blue → purple → icy)
- LCG deterministic RNG for star placement (`crate::utils::math::lcg_rand`)
- Initialized in `lib.rs` as `STARFIELD` thread-local; tier updated from `game.wave` each frame

### High Score Persistence (game.rs)
- `load_high_score()` / `save_high_score()` use `web_sys::window().local_storage()`
- Key: `"rw_defender_high_score"`
- Loaded in `Game::new()`, saved whenever `player.score > self.high_score`

## Constants Added
- `BOSS_AMPLITUDE = 100.0`, `BOSS_FREQ = 0.5`, `BOSS_Y = 50.0`
- `BOSS_FIRE_INTERVAL = 1.5`, `BOSS_BULLET_SPEED = 250.0`
- `DIVER_SPEED = 80.0`, `DIVER_TRIGGER_RANGE = 80.0`
- `DIVER_RETURN_TIME = 2.0`, `DIVER_DIVE_COOLDOWN = 3.0`
- `HIGH_SCORE_KEY = "rw_defender_high_score"`

## Files Changed
- `src/graphics/background.rs` — new: StarField + BackgroundTier
- `src/graphics/mod.rs` — added background module exports
- `src/utils/math.rs` — added `lcg_rand()` LCG helper
- `src/lib.rs` — STARFIELD thread-local, bg canvas init, per-frame update/render
- `src/game.rs` — boss spawn/movement/fire, diver dive, localStorage, score formula
