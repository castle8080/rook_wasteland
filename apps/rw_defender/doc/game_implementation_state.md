# RW Defender — Implementation State

A living reference describing what is built, how it works, and what remains. Updated as phases complete.

---

## What Is This Game

Classic arcade shooter (Space Invaders / Galaga style). Player ship at screen bottom, waves of descending enemies, 6 power-up types, scaling difficulty. Built in **Rust → WebAssembly**, rendered on an **HTML5 Canvas** at 640×480. No external game engine.

**Tech stack**: Rust 2021 + `wasm-bindgen` + `web-sys`, bundled with Trunk.

---

## Implementation Status

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Core engine (RAF loop, input, entity system, sprites, state machine) | ✅ Done |
| 2 | Boss enemies, parallax starfield, high score persistence | ✅ Done |
| 3 — partial | Ship size 2×, 5 lives, ground explosion, ExtraLife power-up | ✅ Done |
| 3 — remaining | Full power-up mechanics (ExplosiveShot AOE, PiercingShot pass-through) | 🔲 Pending |
| 4 | Polish (effects, audio, mobile touch, balance) | 🔲 Pending |

Tests: **29/29 pass**. Clippy: **0 warnings**.

---

## Module Map

```
src/
├── lib.rs              WASM entry point — RAF game loop, event listeners, thread-locals
├── game.rs             Everything: state machine, update, render, collision (~1163 lines)
├── renderer.rs         Thin Canvas 2D abstraction (fill_rect, draw_pixel, text, etc.)
├── entities/
│   ├── mod.rs
│   └── entity.rs       Entity struct, EntityType enum, PowerUpType enum, ActivePowerUp
├── systems/
│   ├── mod.rs
│   └── input.rs        InputState — keyboard bitmask + one-shot consume_* methods
├── graphics/
│   ├── mod.rs
│   ├── colors.rs       RetroColors palette (16-color NES/C64), color_to_css()
│   ├── sprite.rs       Sprite, AnimatedSprite, SpriteGenerator (procedural, no asset files)
│   └── background.rs   StarField (parallax 3-layer), BackgroundTier enum
├── utils/
│   ├── mod.rs
│   └── math.rs         Vec2, Rect (AABB), Timer, lcg_rand()
└── weapons/
    └── mod.rs          Stub — weapon behavior lives in game.rs for now
```

---

## State Machine

```
MainMenu ──(Enter)──► WaveTransition(2s) ──► Playing
                                                │
                                          ◄─(ESC)──► Paused
                                                │
                                          (lives=0 or
                                          enemy reaches bottom)
                                                │
                                                ▼
                                           GameOver(3s) ──(Enter)──► MainMenu
```

**`GameState` enum** (`game.rs`):
- `MainMenu`
- `Playing`
- `Paused`
- `WaveTransition(f64)` — countdown timer
- `GameOver(f64)` — countdown timer

---

## Core Data Structures

### `Entity` (entity.rs)
Single struct for every game object:

```rust
pub struct Entity {
    pub position: Vec2,
    pub velocity: Vec2,
    pub hitbox: Rect,          // local offset AABB; converted to world space on collision check
    pub entity_type: EntityType,
    pub health: i32,
    pub active: bool,          // set false to deactivate; purged between frames
    pub sprite_index: usize,   // also used as bullet tag (0=normal,1=piercing/laser,2=explosive)
    pub anim_elapsed: f64,
    pub anim_frame: usize,
    pub lifetime: f64,         // bullets/explosions/powerups expire
    pub invuln_time: f64,      // player invulnerability countdown
    pub fire_cooldown: f64,    // enemy firing rate
    pub dive_timer: f64,       // diver state timer
    pub phase: f64,            // oscillation/sine offset (weavers, boss, powerup pseudo-rng)
}
```

### `EntityType` enum
```rust
Player, EnemyGrunt, EnemyWeaver, EnemyDiver, Boss,
PlayerBullet, EnemyBullet, Explosion,
/// Large blast at screen bottom when an enemy lands; damages player + nearby enemies.
GroundExplosion,
PowerUp(PowerUpType)
```

### `PowerUpType` enum
```rust
TripleShot, ExplosiveShot, RapidFire, LaserBeam, PiercingShot, Shield,
/// Instantly grants +1 life (max 9).
ExtraLife,
```

### `ActivePowerUp` (entity.rs)
Tracks the single active weapon power-up on the player (shields are a separate counter):
```rust
pub struct ActivePowerUp {
    pub ptype: PowerUpType,
    pub remaining: f64,   // seconds left
}
```

### `Player` (game.rs)
Separate struct (not an Entity) — holds canonical player state:
```rust
pub struct Player {
    pub entity: Entity,
    pub lives: u32,        // 5 starting (max 9 with ExtraLife pickups)
    pub score: u32,
    pub shields: u32,      // 0–3; each absorbs one hit
    pub fire_cooldown: f64,
    pub active_powerup: Option<ActivePowerUp>,
}
```

### `SpriteAtlas` (game.rs)
Pre-generated at startup — holds all `Sprite` objects for every entity type:
```rust
pub struct SpriteAtlas {
    pub player: Sprite,         // 16×16 at scale=2 → 32×32 visual
    pub player_thrust: Sprite,  // thrust animation variant, same size
    pub grunt: Sprite,
    pub weaver: Sprite,
    pub diver: Sprite,
    pub boss: Sprite,
    pub player_bullet: Sprite,
    pub enemy_bullet: Sprite,
    pub explosion_frames: Vec<Sprite>,
    pub powerup_colors: [u32; 7],   // indexed by powerup_index()
}
```

### `Game` (game.rs)
Top-level game state:
```rust
pub struct Game {
    pub state: GameState,
    pub player: Player,
    pub entities: Vec<Entity>,   // all non-player entities
    pub atlas: SpriteAtlas,
    pub wave: u32,
    pub high_score: u32,
    pub spawn_queue: Vec<EntityType>,
    pub spawn_timer: f64,
    pub formation_dir: f64,      // +1/-1 for left-right bounce
    pub formation_x: f64,        // current formation offset
    pub combo_timer: f64,
    pub combo_count: u32,
}
```

---

## Key Constants (game.rs)

| Constant | Value | Notes |
|----------|-------|-------|
| `PLAYER_SPEED` | 250.0 px/s | Horizontal movement |
| `PLAYER_FIRE_COOLDOWN` | 0.25 s | 4 shots/sec base |
| `PLAYER_MAX_BULLETS` | 3 | 6 with RapidFire |
| `BULLET_SPEED` | 450.0 px/s | Upward |
| `BULLET_LIFETIME` | 1.07 s | ~480px travel |
| `INVULN_DURATION` | 2.5 s | After taking a hit |
| `INVULN_FLASH_HZ` | 8.0 Hz | Player blink rate |
| `GRUNT_SPEED` | 30.0 px/s | Descent speed |
| `ENEMY_BULLET_SPEED` | 160.0 px/s | Scales with wave |
| `ENEMY_FIRE_INTERVAL` | 0.5 s | Fire-check cadence |
| `FORMATION_COLS` | 7 | Enemies per row |
| `FORMATION_ROWS` | 3 | Rows per wave |
| `FORMATION_SPACING_X` | 62.0 px | Horizontal gap |
| `FORMATION_SPACING_Y` | 50.0 px | Vertical gap |
| `WAVE_TRANSITION_DURATION` | 2.0 s | Between waves |
| `BOSS_AMPLITUDE` | 100.0 px | Sine-wave X range |
| `BOSS_FREQ` | 0.5 Hz | Sine-wave frequency |
| `BOSS_Y` | 50.0 | Fixed vertical position |
| `BOSS_FIRE_INTERVAL` | 1.5 s | 3-shot spread |
| `BOSS_BULLET_SPEED` | 250.0 px/s | |
| `DIVER_SPEED` | 80.0 px/s | During dive |
| `DIVER_TRIGGER_RANGE` | ±80.0 px | Player proximity trigger |
| `DIVER_RETURN_TIME` | 2.0 s | Time in dive before deactivate |
| `DIVER_DIVE_COOLDOWN` | 3.0 s | Between dives |
| `GROUND_EXPLOSION_RADIUS` | 192.0 px | 30% of 640px canvas; centered at screen bottom |
| `GROUND_EXPLOSION_DURATION` | 0.8 s | Blast window + visual fade |
| `HIGH_SCORE_KEY` | `"rw_defender_high_score"` | localStorage key |

---

## Game Loop (lib.rs)

RAF-based, delta-time capped at 100ms:

```
Each frame:
  1. Compute dt (capped at 0.1s)
  2. game.update(input, dt)
       → handle state transitions
       → update_playing():
           update_player (move, fire, invuln tick)
           update_spawning (200ms queue drain)
           update_enemies (movement, fire AI)
           update_bullets (move, expire)
           update_explosions (animate, expire)
           update_ground_explosions (tick lifetime, deactivate)
           update_powerups (move, expire, tick active powerup)
           check_collisions
           purge inactive entities
           check_wave_complete
  3. game.render(renderer)
       → clear canvas
       → render_game (entities, player)
       → render_hud
       → render overlay (menu / pause / wave / game over)
  4. starfield.update(dt) + starfield.render(bg_renderer)
  5. background image update (every 5 waves, CSS img swap)
```

---

## Systems Detail

### Input (systems/input.rs)
- Tracks held keys as booleans: `left`, `right`, `fire`, `pause`, `start`
- One-shot flags: `consume_pause()`, `consume_start()` — cleared after read (prevents key repeat)
- Mapped keys: Arrows + WASD for move, Space/W/↑ for fire, ESC/P for pause, Enter for start

### Enemy Movement

| Type | Behavior |
|------|----------|
| **Grunt** | Descends at `GRUNT_SPEED × speed_multiplier`; formation shifts left/right, bounces off walls |
| **Weaver** | Descends + sinusoidal X oscillation (40px amplitude, 1.5 Hz); `phase` tracks offset |
| **Diver** | Formation movement until player within ±80px → dives straight down at 80px/s → deactivates after 2s |
| **Boss** | Fixed Y=50; X = `center + BOSS_AMPLITUDE × sin(phase × π × BOSS_FREQ)` |

### Enemy Fire AI
- Checked every `ENEMY_FIRE_INTERVAL` (0.5s) per enemy
- Fire probability scales with wave: `3% + wave×0.5%` (capped ~15%)
- Aimed-shot probability scales with wave: `15% + wave×3%` (capped ~50%)
- Pseudo-random via `entity.phase * golden_ratio` (avoids borrow conflict with `self.rng`)
- Boss fires 3-shot spread (±20°) every `BOSS_FIRE_INTERVAL`

### Collision Detection (check_collisions, game.rs)
- **PlayerBullet vs Enemy**: both deactivate → explosion spawn, score, 8% powerup drop
- **EnemyBullet vs Player**: if shield → consume shield; else → damage + invuln reset
- **Enemy vs Player**: same as above + enemy deactivates
- **PowerUp vs Player**: 16px radius check → apply effect (weapon/shield → as before; ExtraLife → lives +1)
- **GroundExplosion vs Player**: radial distance ≤ `GROUND_EXPLOSION_RADIUS` → player takes fatal hit
- **GroundExplosion vs Enemies**: radial distance ≤ `GROUND_EXPLOSION_RADIUS` → enemy deactivates (no score)
- **Enemy reaches bottom** (y > CANVAS_H − 32): spawns GroundExplosion, enemy deactivated — no instant game over
- Hitbox: `entity.hitbox` is local-space Rect; add `entity.position` to get world AABB

### Scoring
- Grunt: 10 pts, Weaver: 20 pts, Diver: 30 pts, Boss: `100 × (wave/5)` pts
- Wave clear bonus: `100 × wave`
- Combo (2+ kills within 2s): `1.5×` multiplier, tracked by `combo_timer` + `combo_count`
- Extra life: every 10,000 points
- High score saved to `localStorage` immediately on beat

### Wave Spawning
- `begin_wave()` fills `spawn_queue: Vec<EntityType>` with wave's enemies
- Enemy count: `5 + wave × 1.5` (capped at 49)
- Composition by wave range (see spec); Boss spawns alone on every 5th wave
- `update_spawning()` drains queue at 200ms intervals, calling `spawn_enemy()`
- Formation placement: `col = index % FORMATION_COLS`, `row = index / FORMATION_COLS`
- `speed_multiplier()`: `min(2.0, 1.0 + wave × 0.05)`

---

## Rendering

### Sprite System (graphics/sprite.rs)
- **`Sprite`**: `Vec<u32>` pixel buffer (0xAARRGGBB), width, height
- **`AnimatedSprite`**: Vec of `Sprite` frames + frame duration
- **`SpriteGenerator`**: Methods for each entity type — procedurally draws pixels using `RetroColors` palette
- Rendered per-pixel with last-color cache (avoids redundant `set_fill_style` calls)
- Alpha < 10 → transparent pixel (skipped)
- Scale factor applied during draw (enemies at 2× = 40×32 visual; **player now also at 2× = 32×32 visual**)

### Layers
1. **Background canvas** (CSS z-index 0): NASA JPEG (CSS filtered) + StarField overlay
2. **Game canvas** (CSS z-index 1): Entities, HUD, overlays

### Background System (graphics/background.rs)
- `BackgroundTier` enum: `NebulaeWarm` (1-5), `NebulaeDetail` (6-11), `Galaxies` (12-17), `DeepSpace` (18+)
- `background_by_index(idx)` returns filename string; tier cycles through its image list
- `StarField`: 3 parallax layers (different speeds + star sizes), updates per frame
- Star colors shift warmer at lower tiers, cooler/bluer at deep space tiers

### HUD
- Score: top-left, white 12px
- High score: below score, yellow 10px  
- Wave: top-center, cyan 12px
- Lives: top-right (ship icons)
- Shields: `S` indicators, cyan
- Active power-up: colored box + countdown bar, bottom-left

---

## Power-Up System

| Type | Color | Duration | Effect |
|------|-------|----------|--------|
| TripleShot | Orange | 12s | 3 bullets: center + ±15° angled |
| ExplosiveShot | Red | 15s | Bullets tagged `sprite_index=2`; splash AOE pending |
| RapidFire | Yellow | 12s | Fire cooldown 0.083s, max 6 bullets |
| LaserBeam | Blue | 10s | Bullets tagged `sprite_index=1` (laser visual) |
| PiercingShot | Green | 12s | Bullets tagged `sprite_index=1`; pass-through pending |
| Shield | Cyan | Until hit | +1 shield (stacks to 3), absorbed before health |
| **ExtraLife** | **Magenta** | **Instant** | **+1 life (max 9); no weapon slot consumed** |

**Drop**: 8% chance on any enemy death. Type weights: ExtraLife 4%, all others ~16% each.  
**Collection**: 16px radius from player center.  
**Only one weapon active at a time** (new pickup replaces old); Shield and ExtraLife never occupy the weapon slot.  
**⚠ Remaining Phase 3 TODO**: ExplosiveShot splash AOE and PiercingShot pass-through logic not yet implemented — `sprite_index` tags are set but collision handling treats them like normal bullets.

---

## Patterns & Conventions

- **Thread-local RefCell**: `thread_local! { static GAME: RefCell<Game> }` — required for WASM (no threads)
- **Composition over inheritance**: single `Entity` struct + `EntityType` discriminant
- **One-shot input**: `consume_pause()` / `consume_start()` cleared after first read each frame
- **Delta time**: always in seconds (JS `performance.now()` ms ÷ 1000)
- **Entity deactivation**: set `active = false` mid-frame; purge via `retain()` between frames (never remove during iteration)
- **Pseudo-random for enemies**: `entity.phase * 1.618...` to avoid borrowing `self.rng` while iterating `self.entities`
- **Color format**: `0xAARRGGBB` u32; palette indices in sprite source arrays
- **`#[rustfmt::skip]`** on sprite pixel arrays to preserve visual layout

---

## File Sizes (approximate, debug build)

| File | Lines |
|------|-------|
| `game.rs` | ~1163 |
| `graphics/sprite.rs` | ~400+ |
| `graphics/background.rs` | ~200 |
| `entities/entity.rs` | ~150 |
| `utils/math.rs` | ~200 (incl. tests) |
| WASM binary | ~391 KB (debug) |
