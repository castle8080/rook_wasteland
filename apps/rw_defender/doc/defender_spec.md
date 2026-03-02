# Defender-Style Arcade Shooter - Functional & Technical Specification

## Executive Summary

This document outlines the design and implementation plan for a classic arcade-style shooter game compiled from Rust to WebAssembly. The game draws inspiration from late 1980s arcade classics like **Space Invaders**, **Galaga**, and **Galaxian**, featuring a player-controlled ship at the bottom of the screen defending against waves of descending alien invaders with progressively increasing difficulty.

## 1. Game Overview

### 1.1 Game Genre & Style
- **Genre**: Fixed shooter / Shoot 'em up
- **Visual Style**: 8-bit arcade aesthetic reminiscent of 1980s arcade games
- **Platform**: Web browser (Rust compiled to WebAssembly)
- **Display**: HTML5 Canvas-based rendering

### 1.2 Core Gameplay Loop
1. Player controls a ship at the bottom of the screen
2. Waves of aliens descend from the top in attack patterns
3. Player moves left/right and fires projectiles upward
4. Destroy all aliens to advance to the next wave
5. Difficulty increases with each wave
6. Game ends when player loses all lives

## 2. Functional Requirements

### 2.1 Player Mechanics

#### 2.1.1 Movement
- **Left/Right Movement**: Smooth horizontal movement along the bottom of the screen
- **Control Inputs**: 
  - Keyboard: Arrow keys or A/D keys
  - Touch: Touch/drag for mobile support (future enhancement)
- **Movement Constraints**: Player cannot leave screen boundaries
- **Base Speed**: 250 pixels/second (at 60 FPS = 4.17 pixels/frame)
- **Acceleration**: Instant (no ramp-up for responsive arcade feel)
- **Deceleration**: Instant stop when key released

#### 2.1.2 Firing Mechanics
- **Fire Input**: Spacebar or up arrow
- **Projectile Type**: Single bullet traveling vertically upward
- **Base Fire Rate**: 250ms cooldown between shots (4 shots/second)
- **Max Bullets On Screen**: 3 simultaneous bullets (base weapon)
- **Bullet Speed**: 450 pixels/second (7.5 pixels/frame at 60 FPS)
- **Bullet Lifetime**: Auto-destroy after 480 pixels traveled (~1.07 seconds)
- **Bullet Size**: 4x8 pixels (hitbox: 3x7 pixels)

**Fire Rate Progression with Power-Ups:**
- Normal: 250ms cooldown
- Rapid Fire: 83ms cooldown (3x rate, 12 shots/second)
- Triple Shot: 250ms cooldown (fires 3 bullets)
- Laser Beam: Continuous fire, 100ms overheat after 3 seconds

#### 2.1.3 Player Lives & Health
- **Starting Lives**: 5 lives
- **Death Condition**: Contact with alien, alien projectile, or ground explosion blast radius
- **Death Animation Duration**: 500ms explosion effect
- **Respawn Delay**: 1 second after death animation
- **Invulnerability Duration**: 2.5 seconds after respawn
- **Visual Feedback**: Sprite flashes at 8 Hz (125ms on/off cycle) during invulnerability
- **Respawn Position**: Center bottom of screen (320px on 640px width canvas)
- **Game Over Condition**: Losing all 5 lives; aliens reaching the ground does NOT immediately end the game

#### 2.1.3a Ground Explosion (Alien Reaches Bottom)
When an enemy descends to the bottom of the screen instead of triggering game over it detonates in a ground explosion:
- **Trigger**: Any enemy (Grunt, Weaver, Diver) reaches Y ≥ CANVAS_H − 32
- **Blast Radius**: 30% of canvas width = 192 pixels
- **Center**: Horizontally centered on the alien's position, anchored to the screen bottom
- **Duration**: 0.8 seconds (visual + damage window)
- **Player Damage**: If player center is within blast radius, player takes fatal damage (loses a life); standard invulnerability applies
- **Friendly Fire on Enemies**: Any other enemy within blast radius is destroyed (no score awarded for collateral kills)
- **Visual**: Large orange/yellow flash expanding from bottom edge, fades out over duration using alpha
- **Entity Type**: `GroundExplosion` — a distinct entity with its own rendering pass

#### 2.1.4 Weapon Power-Ups
Power-ups drop randomly from destroyed enemies and float down the screen. Players collect them by contact.

**Power-Up Types:**

**1. Triple Shot**
- **Effect**: Fires three bullets in a spread pattern (straight, 15° left, 15° right)
- **Duration**: 12 seconds
- **Drop Rate**: 5% from any enemy
- **Visual**: Orange "T" icon
- **Stacking**: Refreshes duration if collected again
- **Bullet Speed**: Same as normal (450 pixels/second)
- **Spread Angle**: ±15° from vertical

**2. Explosive Shot**
- **Effect**: Bullets explode on impact with 48px radius
- **Damage**: Destroys all enemies within blast radius
- **Duration**: 15 seconds
- **Drop Rate**: 3% from enemies, 15% from bosses
- **Visual**: Red "E" icon with spark animation
- **Stacking**: Refreshes duration
- **Explosion Duration**: 300ms visual effect
- **Explosion Damage Check**: Single-frame (no continuous damage)

**3. Rapid Fire**
- **Effect**: Fire rate increased 3x, max 6 bullets on screen
- **Duration**: 12 seconds
- **Drop Rate**: 4% from any enemy
- **Visual**: Yellow lightning bolt icon
- **Stacking**: Refreshes duration
- **Fire Cooldown**: 83ms (reduced from 250ms)
- **Bullet Speed**: Same as normal (450 pixels/second)

**4. Laser Beam**
- **Effect**: Continuous vertical beam that damages all enemies in path
- **Duration**: 10 seconds of available charge
- **Drop Rate**: 2% from enemies, 10% from bosses
- **Visual**: Blue "L" icon
- **Stacking**: Refreshes duration
- **Beam Width**: 8 pixels
- **Damage Rate**: 30 damage/second (500ms to kill basic enemy)
- **Overheat**: After 3 seconds continuous use, 1 second cooldown required
- **Charge Consumption**: Depletes duration while firing

**5. Piercing Shot**
- **Effect**: Bullets pass through enemies, can hit multiple targets
- **Duration**: 12 seconds
- **Drop Rate**: 4% from any enemy
- **Visual**: Green arrow icon
- **Stacking**: Refreshes duration
- **Bullet Speed**: Same as normal (450 pixels/second)
- **Max Pierces**: Unlimited (travels full screen)

**6. Shield**
- **Effect**: Absorbs one hit from enemy or projectile
- **Duration**: Until hit or 30 seconds
- **Drop Rate**: 3% from enemies, 20% from bosses
- **Visual**: Cyan shield icon, blue bubble around player when active
- **Stacking**: Can stack up to 3 shields
- **Bubble Pulse**: Pulsates at 2 Hz (500ms cycle)
- **Warning Blink**: When 5 seconds remain, blinks at 4 Hz

**7. Extra Life**
- **Effect**: Instantly awards +1 life (up to a maximum of 9)
- **Duration**: Instant (no timer)
- **Drop Rate**: 2% from any enemy, 5% from bosses
- **Visual**: Bright white/magenta star icon
- **Stacking**: Each collected adds one life; no weapon slot consumed

**Power-Up Mechanics:**
- Power-ups spawn at enemy death location
- Spawn Delay: Instant on death
- Float Speed: 80 pixels/second (1.33 pixels/frame at 60 FPS)
- Screen Lifetime: 6 seconds before despawn
- Despawn Animation: 500ms fade out
- Collection Range: 16px radius from player center
- Warning Blink: Last 2 seconds, blinks at 4 Hz
- Only one weapon power-up active at a time (Shield can stack with weapons)
- Power-up Switch: Collecting new weapon immediately replaces current weapon
- Visual indicator shows active power-up icon and remaining time bar in UI
- Time Bar Color: Green (>66%), Yellow (33-66%), Red (<33%)

### 2.2 Enemy Mechanics

#### 2.2.1 Enemy Types
**Type 1: Basic Grunt (Invader)**
- Base Speed: 40 pixels/second descent
- Lateral Movement: 30 pixels/second during formation shift
- Movement Pattern: Straight downward with group lateral shifts
- Point Value: 10 points
- Health: 1 hit (15 HP)
- Size: 16x16 pixels (hitbox: 14x14)
- Shooting: None (waves 1-2), 8% chance/second (wave 3+)

**Type 2: Weaver (Zigzag)**
- Base Speed: 50 pixels/second descent
- Zigzag Amplitude: 40 pixels horizontal
- Zigzag Frequency: 1.5 Hz (completes cycle every 0.67 seconds)
- Point Value: 20 points
- Health: 1 hit (15 HP)
- Size: 16x16 pixels (hitbox: 14x14)
- Shooting: 12% chance/second (introduced wave 3+)

**Type 3: Diver (Kamikaze)**
- Base Speed: 80 pixels/second when diving
- Formation Speed: 45 pixels/second before dive
- Dive Trigger: 30% chance when player is within ±80px horizontal range
- Dive Cooldown: 3 seconds between dive attempts
- Point Value: 30 points
- Health: 1 hit (15 HP)
- Size: 16x16 pixels (hitbox: 14x14)
- Return to Formation: 2 seconds after missed dive
- Introduced: Wave 5+

**Type 4: Boss (Special)**
- Appears: Every 5 waves (5, 10, 15, etc.)
- Base Speed: 60 pixels/second lateral movement
- Movement Pattern: Sine wave across top third of screen
- Sine Amplitude: 100 pixels
- Sine Frequency: 0.5 Hz (2 second period)
- Point Value: 100 + (wave_number * 20) points
- Health: 5 hits (75 HP) + 1 hit per 5 waves
- Size: 48x32 pixels (hitbox: 44x28)
- Shooting: 3-shot spread every 1.5 seconds
- Spawn Animation: 2 seconds entrance from top
- Death Animation: 1.5 seconds with multiple explosions

#### 2.2.2 Enemy Behavior
- **Formation**: Start in organized grid formation at top of screen
  - Grid Spacing: 48 pixels horizontal, 40 pixels vertical
  - Initial Y Position: -200 pixels (off-screen)
  - Formation Entry: 2 seconds slide-in animation
- **Movement Patterns**:
  - Wave 1-3: Simple downward descent at base speed
  - Wave 4+: Side-to-side oscillation (±60px, 3 second period) while descending
  - Wave 7+: 20% of enemies perform dive attacks
  - Wave 10+: Formation rotates 15° periodically (every 8 seconds)
- **Descent Speed Progression**: `base_speed * (1 + wave_number * 0.05)`
  - Wave 1: 40 px/s
  - Wave 5: 50 px/s
  - Wave 10: 60 px/s
  - Wave 20: 80 px/s (capped at 2x base speed)
- **Formation Shifting**: 
  - Lateral Speed: 30 px/s as a group
  - Direction Change: Every 2 seconds or at screen edge
  - Edge Buffer: 32 pixels from screen sides
- **Attack Patterns** (wave 3+):
  - Random Shooting Interval: Check every 500ms
  - Base Firing Probability: 5% per check (per enemy)
  - Probability Increase: +0.5% per wave
  - Max Simultaneous Enemy Bullets: 10 + wave_number
  - Target Leading: 30% chance to aim ahead of player position (50px lead)

#### 2.2.3 Enemy Projectiles
- **Appearance**: Simple bullet/bomb sprite (4x4 pixels, red/orange)
- **Base Speed**: 200 pixels/second (3.33 px/frame at 60 FPS)
- **Speed Progression**: `base_speed * (1 + wave_number * 0.03)` up to +50%
  - Wave 1-2: 200 px/s
  - Wave 10: 260 px/s
  - Wave 17+: 300 px/s (capped)
- **Lifetime**: 3 seconds or until off-screen
- **Size**: 4x4 pixels (hitbox: 3x3 pixels)
- **Fire Animation**: 100ms muzzle flash on enemy
- **Pattern Types**:
  - Straight Down: 70% of shots
  - Aimed at Player: 30% of shots (calculated at fire time)
- **Boss Projectiles**:
  - Speed: 250 pixels/second
  - Pattern: 3-shot spread (straight, ±20°)
  - Size: 6x6 pixels
  - Fire Rate: Every 1.5 seconds

### 2.3 Wave/Level Progression

#### 2.3.1 Wave Structure
```
Wave 1:     12 Basic Grunts, no shooting, 45 second duration
Wave 2:     14 Basic Grunts, no shooting, 50 second duration
Wave 3:     16 enemies (12 Grunts, 4 Weavers), basic shooting, 55 seconds
Wave 4:     18 enemies (10 Grunts, 8 Weavers), shooting, 60 seconds
Wave 5:     Boss (5 hits) + 8 Grunts support, 90 second duration
Wave 6-9:   20-26 enemies, mixed types, increased aggression, 60-70 seconds
Wave 10:    Boss (7 hits) + 12 mixed support, 100 second duration
Wave 11+:   Continuous escalation, 70-80 seconds
```

**Wave Timing:**
- Wave Clear Grace Period: 3 seconds before next wave
- Wave Start Notification: 2 second "Wave X" display
- Enemy Spawn Interval: 200ms between each enemy during wave start
- Wave Timeout: If wave takes >3 minutes, enemies speed up by 50%

#### 2.3.2 Difficulty Scaling Formulas

**Enemy Speed Multiplier:**
```
speed_multiplier = min(2.0, 1.0 + wave_number * 0.05)
```
- Wave 1: 1.0x (100%)
- Wave 5: 1.25x (125%)
- Wave 10: 1.5x (150%)
- Wave 20+: 2.0x (200% - capped)

**Enemy Count:**
```
enemy_count = 12 + floor(wave_number * 1.2)
- Capped at 50 enemies per wave
- Boss waves: base_count * 0.6 + boss
```
- Wave 1: 12 enemies
- Wave 5: 18 enemies (or boss + 11)
- Wave 10: 24 enemies (or boss + 14)
- Wave 20: 36 enemies
- Wave 30+: 48 enemies (approaching cap)

**Fire Rate Multiplier:**
```
fire_chance_multiplier = 1.0 + wave_number * 0.1
base_fire_chance = 5%
effective_chance = base_fire_chance * fire_chance_multiplier
```
- Wave 1-2: 0% (no shooting)
- Wave 3: 5% per check (500ms intervals)
- Wave 5: 6% per check
- Wave 10: 10% per check
- Wave 20: 15% per check

**Projectile Speed:**
```
projectile_speed = base_speed * min(1.5, 1.0 + wave_number * 0.03)
```
- Wave 1: 200 px/s
- Wave 5: 230 px/s
- Wave 10: 260 px/s
- Wave 17+: 300 px/s (capped at 1.5x)

**Enemy Composition by Wave:**
```
Wave 1-2:   100% Grunts
Wave 3-4:   75% Grunts, 25% Weavers
Wave 5:     Boss + Grunt support
Wave 6-9:   50% Grunts, 35% Weavers, 15% Divers
Wave 10:    Boss + mixed support
Wave 11-14: 40% Grunts, 35% Weavers, 25% Divers
Wave 15+:   30% Grunts, 40% Weavers, 30% Divers
```

**Power-Up Drop Rate Scaling:**
```
drop_rate = base_drop_rate * min(1.5, 1.0 + wave_number * 0.02)
```
- Higher waves = slightly better drop rates (up to 1.5x)
- Compensates for increased difficulty
- **Formation Complexity**: More intricate patterns after wave 10

### 2.4 Scoring System
- **Basic Enemy Kill**: 10-30 points (by type)
- **Boss Kill**: 100-500 points
- **Wave Clear Bonus**: 100 * wave_number
- **Perfect Wave** (no hits taken): 2x bonus
- **Combo System**: Multiple kills within 2 seconds: 1.5x multiplier
- **Explosive Multi-Kill**: +50 points per additional enemy in explosion
- **Power-Up Collection**: 25 points
- **Extra Life**: Awarded every 10,000 points; also available as a power-up drop

### 2.5 Visual & Audio Design

#### 2.5.1 Graphics Style
- **Resolution**: 640x480 or 800x600 canvas
- **Pixel Style**: Chunky 8-bit sprites (16x16 to 32x32 pixels)
- **Color Palette**: Limited to 16-32 colors, inspired by NES/Commodore 64
- **Animation**: Simple 2-3 frame sprite animations
- **Player Ship Size**: 16×16 sprite rendered at 2× scale = **32×32 visual pixels**; enemies are also 2× so the player ship is visually comparable in size to regular enemies
- **Animation Timing**:
  - Player Ship: 2 frames alternating every 150ms (idle pulse)
  - Enemy Ships: 2-3 frames at 200ms per frame
  - Explosions: 6 frames at 50ms each (300ms total)
  - Power-Up Icons: 4 frames at 150ms (pulse effect)
  - Muzzle Flash: 2 frames at 50ms each
- **Effects**: 
  - Explosion particles on enemy death (larger for explosive shots)
  - Muzzle flash on player fire
  - Real NASA space photography backgrounds (see Background System below)
  - Power-up glow/pulse animation
  - Shield bubble shimmer effect
  - Laser beam with scan-line effect

**Background System - NASA Space Imagery:**

The game uses authentic NASA space photographs as backgrounds, creating a striking contrast between photorealistic cosmic scenery and retro 8-bit pixel art gameplay. This juxtaposition grounds the arcade action in real space exploration imagery.

**Approach**: Static curated set of pre-processed images
- **Source**: NASA public domain imagery (APOD, Hubble, Webb, Mars rovers)
- **Count**: 5-10 carefully selected images
- **Format**: Pre-processed JPEG/PNG (800x600 or smaller)
- **File Size**: ~100KB each after processing (500KB-1MB total)
- **Selection Criteria**:
  - High visual impact (nebulae, planets, deep space)
  - Smooth gradients and minimal busy detail (avoid dense star fields)
  - Color compatibility with game palette
  - Varied mood progression from grounding to cosmic scale

**Image Processing Requirements:**

All NASA images must be processed before integration:

1. **Resize**: Scale to canvas resolution (800x600) to avoid runtime overhead
2. **Gaussian Blur**: 10-12px radius to push background into visual depth
3. **Desaturate**: Reduce to 30-40% saturation to prevent color competition
4. **Darken**: 50-70% brightness reduction (space should be dark)
5. **Color Temperature**: Cool shift toward blue/purple tones
6. **Contrast Reduction**: 80% of original to flatten details

**Processing can be done via:**
- Image editing software (GIMP, Photoshop)
- Command-line tools (ImageMagick)
- Online tools (Photopea)

**Recommended Image Selection by Wave Tier:**

- **Waves 1-5**: Mars surface imagery
  - Examples: Jezero Crater, Valles Marineris, Olympus Mons
  - Mood: Grounding, rust orange/red tones, tangible terrain
  - Effect: Feels like defending Mars colony

- **Waves 6-10**: Close nebulae (Orion, Eagle)
  - Examples: Orion Nebula, Eagle Nebula
  - Mood: Escalation, blues and purples, emerging grandeur
  - Effect: Moving deeper into space

- **Waves 11-15**: Dramatic nebulae (Carina, Pillars of Creation)
  - Examples: Carina Nebula pillars, Horsehead Nebula
  - Mood: Epic scale, dramatic structures, cosmic formations
  - Effect: High-stakes cosmic battle

- **Waves 16-20**: Deep space galaxies
  - Examples: Andromeda Galaxy, Sombrero Galaxy, galaxy clusters
  - Mood: Vast emptiness, ultimate distance, existential scale
  - Effect: Fighting at the edge of known space

- **Boss Waves (5, 10, 15, 20+)**: Ultra deep field imagery
  - Examples: Hubble Ultra Deep Field, Webb Deep Field
  - Mood: Infinite depth, thousands of galaxies, ultimate cosmic scale
  - Effect: Boss battles feel truly epic

**Integration Method:**
- Images embedded as base64 data URLs or loaded from `assets/backgrounds/` directory
- Background selected based on current wave tier
- Smooth fade transition (1 second) when changing background tiers
- Parallax scrolling effect: Background shifts slowly opposite to player movement (0.2x speed)

**CSS Rendering (Applied to background canvas layer):**
```css
#background-canvas {
    filter: blur(10px) brightness(0.5) saturate(0.35) contrast(0.8);
    opacity: 0.7;
    position: absolute;
    z-index: 1;
}

#game-canvas {
    position: absolute;
    z-index: 2;
    image-rendering: pixelated;
    image-rendering: crisp-edges;
}
```

**Alternative**: Apply filters in Rust during rendering if CSS not sufficient:
```rust
ctx.set_global_alpha(0.7);
ctx.draw_image_with_html_image_element(&background_img, 0.0, 0.0)?;
ctx.set_global_alpha(1.0);
```

#### 2.5.2 UI Elements
- **Score Display**: Top-left corner
- **Lives Display**: Top-right corner (ship icons)
- **Wave Number**: Top-center
- **High Score**: Below score
- **Active Power-Up**: Bottom-left, icon + timer bar
- **Shield Counter**: Next to lives display
- **Game Over Screen**: Final score, high score, replay prompt
- **Pause Menu**: ESC key, resume/quit options

#### 2.5.3 Audio (Future Enhancement)
- Retro sound effects for shooting, explosions, and hits
- Background music loops per wave tier
- Web Audio API for synthesis

### 2.6 Game States & Transitions

1. **Main Menu**: Start game, view high scores, instructions
   - Menu Fade-In: 500ms
   - Input Delay: 200ms after display
   
2. **Playing**: Active gameplay
   - Target Frame Rate: 60 FPS (16.67ms per frame)
   - Delta Time: Capped at 100ms (prevents spiral of death)
   
3. **Paused**: Game frozen, menu overlay
   - Pause Fade-In: 200ms
   - Resume Countdown: 3-2-1 over 3 seconds
   
4. **Wave Transition**: Brief screen showing wave number
   - Display Duration: 2 seconds
   - Fade In/Out: 300ms each
   - Wave Number Zoom: 500ms scale animation (0.5x to 1.0x)
   
5. **Game Over**: Display final statistics
   - Death Animation: 500ms
   - Black Screen: 1 second
   - Game Over Text Fade-In: 1 second
   - Stats Display Delay: 500ms stagger per stat line
   - Input Enabled: After 3 seconds total
   
6. **High Score Entry**: Name entry if score qualifies
   - Entry Timeout: 30 seconds
   - Character Blink Rate: 2 Hz (cursor)

## 3. Technical Specification

### 3.1 Technology Stack

#### 3.1.1 Core Technologies
- **Language**: Rust (Edition 2021+)
- **Compilation Target**: WebAssembly (wasm32-unknown-unknown)
- **Binding Layer**: `wasm-bindgen` for JS interop
- **Canvas API**: `web-sys` for DOM and Canvas manipulation

#### 3.1.2 Key Crates
```toml
[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "Window",
    "Document",
    "HtmlCanvasElement",
    "CanvasRenderingContext2d",
    "KeyboardEvent",
    "Performance",
    "console",
]}
js-sys = "0.3"
wee_alloc = "0.4"  # Optional: smaller allocator for WASM
getrandom = { version = "0.2", features = ["js"] }
rand = "0.8"

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "z"  # Optimize for size
lto = true
```

#### 3.1.3 Build Tools
- **wasm-pack**: Build, optimize, and package WASM
- **npm/webpack** (optional): JavaScript bundling and dev server
- **trunk**: Alternative - Rust-native WASM bundler

### 3.2 Architecture

#### 3.2.1 Module Structure
```
src/
├── lib.rs                 # WASM entry point, main game loop
├── game.rs                # Game state machine
├── renderer.rs            # Canvas rendering abstraction
├── entities/
│   ├── mod.rs
│   ├── player.rs         # Player ship logic
│   ├── enemy.rs          # Enemy types and behavior
│   ├── projectile.rs     # Bullet entities
│   ├── powerup.rs        # Power-up items
│   └── explosion.rs      # Particle effects
├── systems/
│   ├── mod.rs
│   ├── input.rs          # Keyboard input handling
│   ├── collision.rs      # AABB collision detection
│   ├── physics.rs        # Movement and updates
│   ├── wave_manager.rs   # Wave spawning logic
│   └── powerup_manager.rs # Power-up spawning and effects
├── weapons/
│   ├── mod.rs
│   ├── weapon_types.rs   # Weapon type definitions
│   └── projectile_factory.rs # Creates projectiles based on active weapon
├── utils/
│   ├── mod.rs
│   ├── math.rs           # Vector2D, Rectangle
│   └── timer.rs          # Delta time, cooldowns
└── graphics/
    ├── mod.rs
    ├── sprite.rs         # Sprite data and rendering
    └── colors.rs         # 8-bit color palette
```

#### 3.2.2 Entity Component System (Lightweight)
While not a full ECS, use composition for entities:

```rust
pub struct Entity {
    pub position: Vec2,
    pub velocity: Vec2,
    pub sprite: Sprite,
    pub hitbox: Rectangle,
    pub entity_type: EntityType,
    pub health: i32,
    pub active: bool,
}

pub enum EntityType {
    Player,
    EnemyGrunt,
    EnemyWeaver,
    EnemyDiver,
    Boss,
    PlayerBullet,
    EnemyBullet,
    Explosion,
    PowerUp(PowerUpType),
}

pub enum PowerUpType {
    TripleShot,
    ExplosiveShot,
    RapidFire,
    LaserBeam,
    PiercingShot,
    Shield,
}

pub struct ActivePowerUp {
    pub power_up_type: PowerUpType,
    pub remaining_time: f64,
}
```

#### 3.2.3 Game Loop (Request Animation Frame)
```rust
use wasm_bindgen::prelude::*;
use web_sys::{window, CanvasRenderingContext2d};

#[wasm_bindgen]
pub struct Game {
    context: CanvasRenderingContext2d,
    last_frame_time: f64,
    state: GameState,
    // ... entities, systems
}

#[wasm_bindgen]
impl Game {
    pub fn new(canvas_id: &str) -> Result<Game, JsValue> {
        // Initialize game
    }
    
    pub fn update(&mut self, current_time: f64) {
        // Calculate delta time in seconds
        let mut delta_time = (current_time - self.last_frame_time) / 1000.0;
        self.last_frame_time = current_time;
        
        // Cap delta time to prevent spiral of death (max 100ms frame time)
        if delta_time > 0.1 {
            delta_time = 0.1;
        }
        
        // Target: 60 FPS = 16.67ms per frame
        match self.state {
            GameState::Playing => {
                self.handle_input();              // Target: <1ms
                self.update_entities(delta_time); // Target: <5ms
                self.check_collisions();          // Target: <3ms
                self.spawn_enemies();             // Target: <1ms
                self.render();                    // Target: <6ms
                
                // Total budget: ~16ms for 60 FPS
            }
            // ... other states
        }
    }
}
```

JavaScript side:
```javascript
import init, { Game } from './pkg/defender_game.js';

async function run() {
    await init();
    const game = Game.new('game-canvas');
    
    function gameLoop(timestamp) {
        game.update(timestamp);
        requestAnimationFrame(gameLoop);
    }
    
    requestAnimationFrame(gameLoop);
}

run();
```

### 3.3 Rendering System

#### 3.3.1 Canvas Setup
- **Double-buffering**: Automatically handled by browser
- **Clear Strategy**: Full canvas clear each frame (`clearRect(0, 0, width, height)`)
- **Canvas Layers**: 
  - Layer 1: NASA photo background (separate canvas, updated rarely)
  - Layer 2: Game canvas with pixel art (updated every frame)
  - Layer 3: UI overlay (optional, for performance)
- **Draw Order** (on game canvas): enemies → player projectiles → player → enemy projectiles → UI → effects
- **Canvas Context Settings**:
  ```rust
  // Disable image smoothing for crisp pixel art (game canvas only)
  game_ctx.set_image_smoothing_enabled(false);
  
  // Enable smoothing for NASA background (background canvas)
  bg_ctx.set_image_smoothing_enabled(true);
  ```
- **Coordinate System**: Top-left origin (0,0), Y increases downward
- **Background System**: NASA space photography with parallax scrolling

#### 3.3.2 Sprite Rendering Approaches

There are several approaches to rendering sprites in Rust/WASM with Canvas. Each has trade-offs:

**Approach 1: fillRect Pixel-by-Pixel (Recommended for 8-bit style)**

Best for authentic retro look with minimal dependencies.

```rust
pub struct Sprite {
    pub pixels: Vec<u32>,  // RGBA8888 format (0xAABBGGRR)
    pub width: u32,
    pub height: u32,
}

impl Sprite {
    pub fn draw(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
        // Draw each pixel as a small rectangle
        for py in 0..self.height {
            for px in 0..self.width {
                let pixel = self.pixels[(py * self.width + px) as usize];
                
                // Extract RGBA components
                let a = ((pixel >> 24) & 0xFF) as f64 / 255.0;
                if a < 0.01 { continue; } // Skip transparent pixels
                
                let r = (pixel >> 16) & 0xFF;
                let g = (pixel >> 8) & 0xFF;
                let b = pixel & 0xFF;
                
                let color = format!("rgba({},{},{},{})", r, g, b, a);
                ctx.set_fill_style(&color.into());
                ctx.fill_rect(
                    x + px as f64,
                    y + py as f64,
                    1.0, // 1 pixel width
                    1.0  // 1 pixel height
                );
            }
        }
    }
    
    // Optimized version: batch same-color pixels
    pub fn draw_optimized(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
        let mut last_color = 0u32;
        let mut color_str = String::new();
        
        for py in 0..self.height {
            for px in 0..self.width {
                let pixel = self.pixels[(py * self.width + px) as usize];
                if (pixel >> 24) < 10 { continue; } // Skip transparent
                
                if pixel != last_color {
                    let r = (pixel >> 16) & 0xFF;
                    let g = (pixel >> 8) & 0xFF;
                    let b = pixel & 0xFF;
                    let a = ((pixel >> 24) & 0xFF) as f64 / 255.0;
                    color_str = format!("rgba({},{},{},{})", r, g, b, a);
                    ctx.set_fill_style(&color_str.into());
                    last_color = pixel;
                }
                
                ctx.fill_rect(x + px as f64, y + py as f64, 1.0, 1.0);
            }
        }
    }
}
```

**Pros**: Simple, no image loading, perfect pixel control, retro aesthetic  
**Cons**: Slower for large sprites (optimize by batching colors)  
**Best For**: 8-16px sprites, bullet hell games, maximum retro feel

---

**Approach 2: ImageData + putImageData (Faster for larger sprites)**

Direct pixel buffer manipulation, good performance.

```rust
use web_sys::{ImageData, CanvasRenderingContext2d};
use wasm_bindgen::Clamped;

pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub image_data: ImageData,
}

impl Sprite {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, JsValue> {
        // pixels should be RGBA format: [r,g,b,a, r,g,b,a, ...]
        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&pixels),
            width,
            height,
        )?;
        
        Ok(Sprite {
            width,
            height,
            image_data,
        })
    }
    
    pub fn draw(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) -> Result<(), JsValue> {
        ctx.put_image_data(&self.image_data, x, y)
    }
}

// Helper to create sprite from color data
pub fn create_sprite_from_palette(
    width: u32,
    height: u32,
    indices: &[u8],  // Palette indices
    palette: &[(u8, u8, u8, u8)],  // RGBA palette
) -> Result<Sprite, JsValue> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    
    for &idx in indices {
        let (r, g, b, a) = palette[idx as usize];
        pixels.extend_from_slice(&[r, g, b, a]);
    }
    
    Sprite::new(width, height, pixels)
}
```

**Pros**: Fast, direct pixel manipulation, good for effects  
**Cons**: No rotation/scaling, manual alpha handling  
**Best For**: 16-32px sprites, particle effects, backgrounds

---

**Approach 3: HTML Image Element + drawImage (Most flexible)**

Load images as browser resources, use full Canvas API features.

```rust
use web_sys::{HtmlImageElement, CanvasRenderingContext2d};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

pub struct Sprite {
    pub image: HtmlImageElement,
    pub width: f64,
    pub height: f64,
}

impl Sprite {
    pub async fn load_from_url(url: &str) -> Result<Self, JsValue> {
        let image = HtmlImageElement::new()?;
        
        // Create promise that resolves when image loads
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let img_clone = image.clone();
            let onload = Closure::wrap(Box::new(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut()>);
            
            let onerror = Closure::wrap(Box::new(move |e: web_sys::Event| {
                reject.call1(&JsValue::NULL, &e).unwrap();
            }) as Box<dyn FnMut(_)>);
            
            image.set_onload(Some(onload.as_ref().unchecked_ref()));
            image.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            
            onload.forget();
            onerror.forget();
        });
        
        image.set_src(url);
        JsFuture::from(promise).await?;
        
        let width = image.width() as f64;
        let height = image.height() as f64;
        
        Ok(Sprite { image, width, height })
    }
    
    pub fn draw(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) -> Result<(), JsValue> {
        ctx.draw_image_with_html_image_element(&self.image, x, y)
    }
    
    // Draw with scaling
    pub fn draw_scaled(&self, ctx: &CanvasRenderingContext2d, 
                       x: f64, y: f64, 
                       scale: f64) -> Result<(), JsValue> {
        ctx.draw_image_with_html_image_element_and_dw_and_dh(
            &self.image,
            x, y,
            self.width * scale,
            self.height * scale,
        )
    }
    
    // Draw sprite sheet frame
    pub fn draw_frame(&self, ctx: &CanvasRenderingContext2d,
                      frame_x: u32, frame_y: u32,
                      frame_width: u32, frame_height: u32,
                      dest_x: f64, dest_y: f64) -> Result<(), JsValue> {
        ctx.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &self.image,
            frame_x as f64, frame_y as f64,
            frame_width as f64, frame_height as f64,
            dest_x, dest_y,
            frame_width as f64, frame_height as f64,
        )
    }
}
```

**Pros**: Fast, hardware accelerated, supports rotation/scaling, sprite sheets  
**Cons**: Requires async loading, external files, more setup  
**Best For**: Larger games, sprite sheets, production builds

---

**Approach 4: Procedural Generation (Recommended for prototypes)**

Generate sprites programmatically in code - no assets needed!

```rust
pub struct SpriteGenerator;

impl SpriteGenerator {
    // Generate player ship sprite
    pub fn player_ship() -> Vec<u32> {
        // 16x16 player ship (stored row by row)
        // 0 = transparent, other values = colors
        let pattern = [
            0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,
            0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,0,
            0,0,0,0,0,0,1,2,2,1,0,0,0,0,0,0,
            0,0,0,0,0,1,2,2,2,2,1,0,0,0,0,0,
            0,0,0,0,1,2,2,3,3,2,2,1,0,0,0,0,
            0,0,0,1,2,2,3,3,3,3,2,2,1,0,0,0,
            0,0,1,2,2,2,3,3,3,3,2,2,2,1,0,0,
            0,1,2,2,2,2,2,3,3,2,2,2,2,2,1,0,
            1,2,2,2,2,2,2,2,2,2,2,2,2,2,2,1,
            1,1,2,2,2,2,2,2,2,2,2,2,2,2,1,1,
            0,1,1,2,2,2,2,2,2,2,2,2,2,1,1,0,
            0,0,1,1,2,2,2,2,2,2,2,2,1,1,0,0,
            0,0,0,1,1,2,2,2,2,2,2,1,1,0,0,0,
            0,0,0,0,1,1,1,0,0,1,1,1,0,0,0,0,
            0,0,0,1,1,0,0,0,0,0,0,1,1,0,0,0,
            0,0,1,1,0,0,0,0,0,0,0,0,1,1,0,0,
        ];
        
        let palette = [
            0x00000000, // 0: Transparent
            0xFF606060, // 1: Dark gray
            0xFF00AAFF, // 2: Blue
            0xFF00FFFF, // 3: Cyan (cockpit)
        ];
        
        pattern.iter().map(|&idx| palette[idx as usize]).collect()
    }
    
    // Generate enemy sprite
    pub fn enemy_grunt() -> Vec<u32> {
        let pattern = [
            0,0,1,1,1,1,1,1,1,1,1,1,0,0,
            0,1,1,2,2,2,2,2,2,2,2,1,1,0,
            1,1,2,2,3,2,2,2,2,3,2,2,1,1,
            1,2,2,2,2,2,2,2,2,2,2,2,2,1,
            1,2,2,2,2,2,4,4,2,2,2,2,2,1,
            1,2,2,2,2,4,4,4,4,2,2,2,2,1,
            1,2,2,4,4,4,4,4,4,4,4,2,2,1,
            1,2,4,4,1,4,4,4,4,1,4,4,2,1,
            1,1,4,4,1,4,4,4,4,1,4,4,1,1,
            0,1,1,4,4,4,1,1,4,4,4,1,1,0,
            0,0,1,1,4,1,1,1,1,4,1,1,0,0,
            0,0,0,1,1,1,0,0,1,1,1,0,0,0,
        ];
        
        let palette = [
            0x00000000, // 0: Transparent
            0xFF00FF00, // 1: Green outline
            0xFF00AA00, // 2: Dark green
            0xFFFF0000, // 3: Red eyes
            0xFF00FF00, // 4: Bright green
        ];
        
        // Width = 14, Height = 12
        pattern.iter().map(|&idx| palette[idx as usize]).collect()
    }
    
    // Generate bullet
    pub fn bullet() -> Vec<u32> {
        let yellow = 0xFFFFFF00;
        let white = 0xFFFFFFFF;
        vec![
            yellow, yellow,
            yellow, yellow,
            white, white,
            white, white,
            yellow, yellow,
            yellow, yellow,
        ] // 2x6 pixels
    }
    
    // Generate explosion frame (multiple frames for animation)
    pub fn explosion_frame(frame: u32) -> Vec<u32> {
        match frame {
            0 => Self::explosion_small(),
            1 => Self::explosion_medium(),
            2 => Self::explosion_large(),
            _ => Self::explosion_fade(),
        }
    }
    
    fn explosion_small() -> Vec<u32> {
        // 8x8 initial explosion
        let orange = 0xFFFF8800;
        let yellow = 0xFFFFFF00;
        // ... pattern ...
        vec![orange; 64]  // Simplified
    }
}

// Usage
let sprite = Sprite {
    pixels: SpriteGenerator::player_ship(),
    width: 16,
    height: 16,
};
```

**Pros**: No assets, instant startup, easy to modify, version control friendly  
**Cons**: Manual sprite design, limited detail  
**Best For**: Prototyping, game jams, minimalist games, this project!

---

**Approach 5: Embedded Base64 Data URLs**

Embed small PNG sprites directly in code.

```rust
pub struct SpriteCache {
    sprites: HashMap<String, HtmlImageElement>,
}

impl SpriteCache {
    pub fn new() -> Self {
        let mut cache = Self {
            sprites: HashMap::new(),
        };
        cache.load_embedded_sprites();
        cache
    }
    
    fn load_embedded_sprites(&mut self) {
        // Embed small sprites as base64 data URLs
        // Created with: base64 -i sprite.png
        const PLAYER_SPRITE: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAA...";
        
        let img = HtmlImageElement::new().unwrap();
        img.set_src(PLAYER_SPRITE);
        self.sprites.insert("player".to_string(), img);
    }
    
    pub fn get(&self, name: &str) -> Option<&HtmlImageElement> {
        self.sprites.get(name)
    }
}
```

**Pros**: No external files, good performance, use real art tools  
**Cons**: Increases WASM size, base64 encoding overhead  
**Best For**: Small games, avoiding HTTP requests, single-file distribution

---

#### 3.3.3 Recommended Approach for This Game

For the defender game, use **Approach 4 (Procedural)** + **Approach 1 (fillRect)**:

```rust
pub struct SpriteAtlas {
    pub player: Sprite,
    pub enemies: Vec<Sprite>,
    pub bullet: Sprite,
    pub powerups: Vec<Sprite>,
}

impl SpriteAtlas {
    pub fn new() -> Self {
        Self {
            player: Sprite {
                pixels: SpriteGenerator::player_ship(),
                width: 16,
                height: 16,
            },
            enemies: vec![
                Sprite {
                    pixels: SpriteGenerator::enemy_grunt(),
                    width: 14,
                    height: 12,
                },
                // ... other enemy types
            ],
            bullet: Sprite {
                pixels: SpriteGenerator::bullet(),
                width: 2,
                height: 6,
            },
            powerups: Self::generate_powerup_sprites(),
        }
    }
    
    fn generate_powerup_sprites() -> Vec<Sprite> {
        // Generate 6 powerup icons (T, E, R, L, P, S)
        // ... 
        vec![]
    }
}

// In game initialization
let atlas = SpriteAtlas::new();

// In render loop
atlas.player.draw_optimized(&ctx, player_x, player_y);
```

**Why this approach?**
- ✅ Zero asset dependencies
- ✅ Authentic 8-bit pixel art aesthetic
- ✅ Easy to tweak during development
- ✅ Small WASM binary size
- ✅ Instant startup (no loading)
- ✅ Version control friendly (no binary files)

#### 3.3.4 Sprite Animation System

```rust
pub struct AnimatedSprite {
    pub frames: Vec<Sprite>,
    pub frame_duration: f64,  // Seconds per frame
    pub current_frame: usize,
    pub elapsed_time: f64,
    pub looping: bool,
}

impl AnimatedSprite {
    pub fn update(&mut self, delta_time: f64) {
        self.elapsed_time += delta_time;
        
        if self.elapsed_time >= self.frame_duration {
            self.elapsed_time -= self.frame_duration;
            self.current_frame += 1;
            
            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                }
            }
        }
    }
    
    pub fn draw(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
        self.frames[self.current_frame].draw_optimized(ctx, x, y);
    }
}

// Usage for explosions
let explosion = AnimatedSprite {
    frames: vec![
        Sprite { pixels: SpriteGenerator::explosion_frame(0), width: 16, height: 16 },
        Sprite { pixels: SpriteGenerator::explosion_frame(1), width: 16, height: 16 },
        Sprite { pixels: SpriteGenerator::explosion_frame(2), width: 16, height: 16 },
        // ...
    ],
    frame_duration: 0.05,  // 50ms per frame
    current_frame: 0,
    elapsed_time: 0.0,
    looping: false,
};
```

#### 3.3.5 Pixel Art Best Practices

**CSS for Canvas Element:**
```css
canvas {
    /* Disable anti-aliasing for crisp pixels */
    image-rendering: pixelated;
    image-rendering: crisp-edges;
    image-rendering: -moz-crisp-edges;
    
    /* Scale up for modern displays */
    width: 1280px;   /* 2x scale */
    height: 960px;
    
    /* Keep original resolution */
    /* Canvas will be 640x480 internally */
}
```

**Color Palette (8-bit style):**
```rust
pub struct RetroColors;

impl RetroColors {
    // Classic arcade palette (16 colors)
    pub const BLACK: u32       = 0xFF000000;
    pub const DARK_BLUE: u32   = 0xFF0000AA;
    pub const DARK_GREEN: u32  = 0xFF00AA00;
    pub const DARK_CYAN: u32   = 0xFF00AAAA;
    pub const DARK_RED: u32    = 0xFFAA0000;
    pub const DARK_MAGENTA: u32= 0xFFAA00AA;
    pub const BROWN: u32       = 0xFFAA5500;
    pub const GRAY: u32        = 0xFFAAAAAA;
    pub const DARK_GRAY: u32   = 0xFF555555;
    pub const BLUE: u32        = 0xFF5555FF;
    pub const GREEN: u32       = 0xFF55FF55;
    pub const CYAN: u32        = 0xFF55FFFF;
    pub const RED: u32         = 0xFFFF5555;
    pub const MAGENTA: u32     = 0xFFFF55FF;
    pub const YELLOW: u32      = 0xFFFFFF55;
    pub const WHITE: u32       = 0xFFFFFFFF;
}
```

**Performance Tips:**
- Cache color strings to avoid repeated format! calls
- Batch fillRect calls with same color
- Use web-sys bindings directly (faster than js-sys)
- Pre-generate all sprites at startup
- Use separate canvas for NASA background (updated only on wave transitions)

#### 3.3.6 NASA Background System Implementation

```rust
use web_sys::{HtmlImageElement, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::JsCast;

pub struct BackgroundManager {
    images: Vec<HtmlImageElement>,
    current_tier: usize,
    background_canvas: HtmlCanvasElement,
    background_ctx: CanvasRenderingContext2d,
    parallax_offset: f64,
    transition_progress: f64,
}

impl BackgroundManager {
    pub fn new(canvas_width: u32, canvas_height: u32) -> Result<Self, JsValue> {
        // Create dedicated background canvas
        let document = web_sys::window().unwrap().document().unwrap();
        let bg_canvas = document.create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        bg_canvas.set_width(canvas_width);
        bg_canvas.set_height(canvas_height);
        
        let bg_ctx = bg_canvas.get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;
        
        // Enable smoothing for photo backgrounds
        bg_ctx.set_image_smoothing_enabled(true);
        
        Ok(Self {
            images: vec![],
            current_tier: 0,
            background_canvas: bg_canvas,
            background_ctx: bg_ctx,
            parallax_offset: 0.0,
            transition_progress: 0.0,
        })
    }
    
    pub async fn load_backgrounds() -> Result<Vec<HtmlImageElement>, JsValue> {
        let background_urls = vec![
            "assets/backgrounds/mars_surface.jpg",
            "assets/backgrounds/orion_nebula.jpg",
            "assets/backgrounds/carina_nebula.jpg",
            "assets/backgrounds/deep_space.jpg",
            "assets/backgrounds/ultra_deep_field.jpg",
        ];
        
        let mut images = Vec::new();
        for url in background_urls {
            let img = Self::load_image(url).await?;
            images.push(img);
        }
        
        Ok(images)
    }
    
    async fn load_image(url: &str) -> Result<HtmlImageElement, JsValue> {
        let image = HtmlImageElement::new()?;
        
        // Create promise for image loading
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            let img_clone = image.clone();
            let onload = Closure::wrap(Box::new(move || {
                resolve.call0(&JsValue::NULL).unwrap();
            }) as Box<dyn FnMut()>);
            
            let onerror = Closure::wrap(Box::new(move |_: web_sys::Event| {
                reject.call1(&JsValue::NULL, &JsValue::from_str("Image load failed")).unwrap();
            }) as Box<dyn FnMut(_)>);
            
            image.set_onload(Some(onload.as_ref().unchecked_ref()));
            image.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            
            onload.forget();
            onerror.forget();
        });
        
        image.set_src(url);
        wasm_bindgen_futures::JsFuture::from(promise).await?;
        
        Ok(image)
    }
    
    pub fn select_background_for_wave(&mut self, wave: u32) {
        let new_tier = match wave {
            1..=5 => 0,   // Mars surface
            6..=10 => 1,  // Orion nebula
            11..=15 => 2, // Carina nebula
            16..=20 => 3, // Deep space galaxies
            _ => 4,       // Ultra deep field for 20+
        };
        
        if new_tier != self.current_tier {
            self.current_tier = new_tier;
            self.transition_progress = 0.0;
            self.render_background();
        }
    }
    
    pub fn update_parallax(&mut self, player_velocity: f64, delta_time: f64) {
        // Move background opposite to player at 20% speed
        self.parallax_offset -= player_velocity * 0.2 * delta_time;
        
        // Wrap offset to avoid floating point drift
        let canvas_width = self.background_canvas.width() as f64;
        if self.parallax_offset.abs() > canvas_width {
            self.parallax_offset = 0.0;
        }
    }
    
    fn render_background(&self) {
        if self.current_tier >= self.images.len() {
            return;
        }
        
        let img = &self.images[self.current_tier];
        let width = self.background_canvas.width() as f64;
        let height = self.background_canvas.height() as f64;
        
        // Clear
        self.background_ctx.clear_rect(0.0, 0.0, width, height);
        
        // Draw with parallax offset
        self.background_ctx.set_global_alpha(0.7);
        self.background_ctx.draw_image_with_html_image_element_and_dw_and_dh(
            img,
            self.parallax_offset,
            0.0,
            width,
            height,
        ).ok();
        self.background_ctx.set_global_alpha(1.0);
    }
    
    pub fn get_canvas(&self) -> &HtmlCanvasElement {
        &self.background_canvas
    }
}

// In HTML setup, position canvases:
// <div style="position: relative;">
//   <canvas id="background-canvas" style="position: absolute; z-index: 1;"></canvas>
//   <canvas id="game-canvas" style="position: absolute; z-index: 2;"></canvas>
// </div>
```

**Asset Directory Structure:**
```
assets/
└── backgrounds/
    ├── mars_surface.jpg          (~100KB, processed)
    ├── orion_nebula.jpg          (~100KB, processed)
    ├── carina_nebula.jpg         (~100KB, processed)
    ├── deep_space.jpg            (~100KB, processed)
    └── ultra_deep_field.jpg      (~100KB, processed)
```

### 3.4 Input System

#### 3.4.1 Keyboard Input
```rust
use web_sys::KeyboardEvent;

pub struct InputState {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub fire_pressed: bool,
    pub pause_pressed: bool,
}

pub fn setup_input_listeners(window: &Window) -> Result<(), JsValue> {
    let keydown_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        match event.key().as_str() {
            "ArrowLeft" | "a" => /* set left_pressed */,
            "ArrowRight" | "d" => /* set right_pressed */,
            " " | "ArrowUp" => /* set fire_pressed */,
            "Escape" => /* toggle pause */,
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    
    window.add_event_listener_with_callback(
        "keydown", 
        keydown_closure.as_ref().unchecked_ref()
    )?;
    
    keydown_closure.forget();
    Ok(())
}
```

### 3.5 Collision Detection

**Performance Target**: <3ms per frame for all collision checks

**Optimization Strategy**:
- Spatial partitioning: 4x4 grid (16 cells of 160x120px each on 640x480 canvas)
- Early exit on bounding box checks
- Check only active entities
- Process collision pairs in specific order (most likely first)

**Collision Check Frequency**:
- Player vs Enemy Bullets: Every frame
- Player vs Enemies: Every frame
- Player vs Power-Ups: Every frame
- Player Bullets vs Enemies: Every frame
- Enemies vs Screen Bottom: Every 5 frames (not time-critical)

#### 3.5.1 AABB (Axis-Aligned Bounding Box)
```rust
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rectangle {
    pub fn intersects(&self, other: &Rectangle) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}
```

#### 3.5.2 Collision Pairs
- Player vs Enemy Bullets
- Player vs Enemies (collision damage)
- Player vs Power-Ups (collection)
- Player Bullets vs Enemies
- Explosive Bullets vs Enemies (radius check)
- Enemies vs Screen Bottom (ground explosion — NOT game over)
- Ground Explosion vs Player (radius check)
- Ground Explosion vs Enemies (radius check — friendly fire)

#### 3.5.3 Explosion Radius Detection
```rust
pub fn check_explosion_collision(explosion_center: Vec2, radius: f64, entities: &[Entity]) -> Vec<usize> {
    entities.iter().enumerate()
        .filter_map(|(idx, entity)| {
            let distance = (entity.position - explosion_center).length();
            if distance <= radius {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}
```

### 3.6 Wave Management

```rust
pub struct WaveManager {
    current_wave: u32,
    enemies_remaining: u32,
    spawn_timer: f64,
    wave_config: WaveConfig,
}

pub struct WaveConfig {
    pub enemy_count: u32,
    pub enemy_types: Vec<(EntityType, f32)>,  // (type, spawn_chance)
    pub enemy_speed_multiplier: f64,
    pub fire_rate_multiplier: f64,
}

impl WaveManager {
    pub fn generate_wave_config(wave_number: u32) -> WaveConfig {
        let base_count = 12;
        let enemy_count = base_count + (wave_number / 2) * 2;
        let speed_mult = 1.0 + (wave_number as f64 * 0.05).min(1.0);
        
        // Determine enemy composition based on wave number
        // ...
    }
}

pub struct PowerUpManager {
    active_power_ups: Vec<ActivePowerUp>,
    drop_rng: Rng,
}

impl PowerUpManager {
    pub fn try_spawn_powerup(&mut self, enemy_type: EntityType, position: Vec2) -> Option<Entity> {
        let drop_chance = match enemy_type {
            EntityType::Boss => 0.45,  // 45% from bosses
            _ => 0.08,  // 8% from regular enemies
        };
        
        if self.drop_rng.gen::<f64>() < drop_chance {
            let powerup_type = self.random_powerup_type();
            Some(Entity::new_powerup(powerup_type, position))
        } else {
            None
        }
    }
    
    pub fn activate_powerup(&mut self, powerup_type: PowerUpType) {
        // Add or refresh power-up timer
    }
}
```

### 3.7 Performance Considerations

#### 3.7.1 Target Performance Metrics
- **Frame Rate**: Solid 60 FPS (16.67ms frame budget)
- **Frame Time Budget Breakdown**:
  - Input Processing: <1ms
  - Game Logic Update: <5ms
  - Collision Detection: <3ms
  - Rendering: <6ms
  - Browser Overhead: ~1-2ms
- **Maximum Entity Count**: 150 active entities
  - 1 Player
  - 50 Enemies (max)
  - 13 Player Bullets (max with rapid fire + triple shot)
  - 30 Enemy Bullets (max)
  - 20 Explosions/Effects
  - 10 Power-Ups floating
  - 26 entities buffer
- **Input Latency**: <50ms from keypress to visual response
- **Memory Usage Target**: <50MB total
- **WASM Binary Size**: <500KB (uncompressed)

#### 3.7.2 WASM Optimization
- Use `opt-level = "z"` for smaller binary
- Enable LTO (Link Time Optimization)
- Use `wee_alloc` for smaller heap allocator
- Pool/reuse entity objects instead of allocating
  - Pre-allocate entity pool of 150 objects at startup
  - Mark entities as active/inactive instead of allocation
  - Reduces GC pressure and allocation overhead
- Avoid heap allocations in hot paths (game loop)
- Use fixed-size arrays where possible

#### 3.7.3 Rendering Optimization
- **Target Draw Calls**: <200 per frame
- **Culling**: Skip entities outside viewport ±50px buffer
- **Integer Coordinates**: Use integer pixel positions to avoid subpixel artifacts
- **Background Layers**:
  - Static background: Render once, reuse
  - Scrolling stars: Update only Y position, redraw every 3 frames
- **Sprite Batching**: Group similar sprites when possible
- **Canvas Size**: Fixed 640x480 or 800x600 (avoid dynamic resizing)
- **Dirty Rectangle** (future): Only redraw changed regions

#### 3.7.4 Game Logic Optimization
- **Collision Detection Optimization**:
  - Spatial partitioning: Divide screen into 4x4 grid (16 cells)
  - Only check collisions within same/adjacent cells
  - Reduces checks from O(n²) to O(n*k) where k is avg entities per cell
  - Target: <3ms for all collision checks
- **Entity Update Order**:
  1. Update positions (vectorized when possible)
  2. Check bounds (remove off-screen entities)
  3. Collision detection (spatial hash)
  4. Apply game logic (state machines)
- **Fixed Time Step**: 16.67ms updates, interpolate rendering if needed
- **Update Frequency**: Some systems can run at lower frequency:
  - AI decisions: Every 100ms
  - Power-up spawns: Every 500ms check
  - UI updates: Every 33ms (30 Hz acceptable)

### 3.8 Data Persistence

#### 3.8.1 Local Storage
Use `web-sys` to save high scores:
```rust
use web_sys::window;

pub fn save_high_score(score: u32) -> Result<(), JsValue> {
    let storage = window()
        .unwrap()
        .local_storage()?
        .unwrap();
    
    storage.set_item("high_score", &score.to_string())?;
    Ok(())
}
```

### 3.9 Testing Strategy

#### 3.9.1 Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_detection() {
        let rect1 = Rectangle { x: 0.0, y: 0.0, width: 10.0, height: 10.0 };
        let rect2 = Rectangle { x: 5.0, y: 5.0, width: 10.0, height: 10.0 };
        assert!(rect1.intersects(&rect2));
    }
}
```

#### 3.9.2 WASM Tests
```rust
#[cfg(test)]
mod wasm_tests {
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_entity_movement() {
        // Test in WASM environment
    }
}
```

Run with: `wasm-pack test --chrome --headless`

### 3.10 Build & Deployment

#### 3.10.1 Development Build
```bash
# Using wasm-pack
wasm-pack build --target web --dev

# Using trunk (simpler)
trunk serve
```

#### 3.10.2 Production Build
```bash
wasm-pack build --target web --release

# Outputs to pkg/:
# - defender_game_bg.wasm
# - defender_game.js
# - defender_game.d.ts
```

#### 3.10.3 HTML Template
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Defender - Arcade Shooter</title>
    <style>
        body {
            margin: 0;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            background: #000;
        }
        canvas {
            border: 2px solid #0f0;
            image-rendering: pixelated;
            image-rendering: crisp-edges;
        }
    </style>
</head>
<body>
    <canvas id="game-canvas" width="640" height="480"></canvas>
    <script type="module">
        import init, { Game } from './pkg/defender_game.js';
        
        async function run() {
            await init();
            const game = Game.new('game-canvas');
            
            function gameLoop(timestamp) {
                game.update(timestamp);
                requestAnimationFrame(gameLoop);
            }
            
            requestAnimationFrame(gameLoop);
        }
        
        run();
    </script>
</body>
</html>
```

## 4. Development Roadmap

### Phase 1: Core Engine (Week 1-2)
- [ ] Setup Rust/WASM project structure
- [ ] Implement game loop with RAF
- [ ] Canvas rendering abstraction
- [ ] Basic input handling
- [ ] Entity system foundation
- [ ] **Source and process NASA background images**:
  - [ ] Research and select 5-10 high-quality NASA images
  - [ ] Download high-resolution versions from NASA sources
  - [ ] Process images (resize, blur, desaturate, darken)
  - [ ] Optimize file sizes (target ~100KB each)
  - [ ] Create `assets/backgrounds/` directory
  - [ ] Test visual compatibility with pixel art sprites
- [ ] Implement background layer system with parallax

### Phase 2: Player Mechanics (Week 2-3)
- [ ] Player ship movement
- [ ] Player shooting mechanics
- [ ] Player bullet collision
- [ ] Player death and respawn

### Phase 3: Enemy System (Week 3-4)
- [ ] Enemy spawn system
- [ ] Basic enemy type (Grunt)
- [ ] Enemy movement patterns
- [ ] Enemy collision with player bullets

### Phase 4: Combat & Waves (Week 4-5)
- [ ] Wave manager implementation
- [ ] Additional enemy types
- [ ] Enemy shooting mechanics
- [ ] Scoring system

### Phase 4.5: Power-Up System (Week 5)
- [ ] Power-up entity and drop system
- [ ] Weapon types implementation (triple shot, explosive)
- [ ] Power-up timer and UI display
- [ ] Shield system
- [ ] Laser beam mechanics
- [ ] Explosion radius collision detection

### Phase 5: Polish & Features (Week 5-6)
- [ ] Visual effects (explosions, particles)
- [ ] UI elements and game states
- [ ] High score persistence
- [ ] Balance and difficulty tuning
- [ ] Fine-tune NASA background transitions and parallax
- [ ] Verify background visibility across all wave tiers

### Phase 6: Enhancement (Week 6+)
- [ ] Audio system
- [ ] Boss enemies
- [ ] Power-ups
- [ ] Mobile touch controls
- [ ] Performance optimization

## 5. Alternative Technology Considerations

### 5.1 Graphics Libraries
- **ggez**: Game framework (native + WASM support)
- **quicksilver**: 2D game framework for WASM
- **macroquad**: Simple game framework with WASM target
- **bevy**: Full ECS game engine (larger, more complex)

**Recommendation**: Start with raw Canvas API via `web-sys` for learning and control. Consider a framework if project expands significantly.

### 5.2 Asset Pipeline
- **Aseprite**: Create pixel art sprites
- **GraphicsGale**: Free pixel art tool
- **Piskel**: Browser-based pixel editor
- **GIMP**: General image editing

### 5.3 Sound Design
- **Bfxr/ChipTone**: Browser-based 8-bit sound generators
- **FamiTracker**: NES-style music composition
- **Web Audio API**: Real-time audio synthesis in browser

## 6. Success Metrics

### 6.1 Technical Metrics
- WASM binary size < 500KB
- 60 FPS gameplay on modern browsers
- Input latency < 50ms
- Initial load time < 2 seconds

### 6.2 Gameplay Metrics
- Average session length: 5-10 minutes
- Wave progression: 50% of players reach wave 5
- Replay rate: Players attempt 3+ games per session

## 7. References & Resources

### 7.1 Learning Resources
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [web-sys Documentation](https://rustwasm.github.io/wasm-bindgen/api/web_sys/)

### 7.2 Game Design References
- **Space Invaders** (1978): Formation movement, shields
- **Galaga** (1981): Dive-bomb attacks, challenge stages
- **Galaxian** (1979): Smooth movement patterns
- **Phoenix** (1980): Boss battles, multi-stage enemies

### 7.3 Similar Projects
- [rustwasm/wasm-pack-template](https://github.com/rustwasm/wasm-pack-template)
- [Games Made with Rust + WASM](https://github.com/topics/rust-wasm-game)

## 8. Conclusion

This specification provides a comprehensive blueprint for developing a classic arcade-style shooter game using Rust and WebAssembly. The design prioritizes:

1. **Simplicity**: Clear, focused gameplay mechanics
2. **Performance**: Optimized for browser execution
3. **Nostalgia**: Authentic 8-bit aesthetic
4. **Progression**: Satisfying difficulty curve
5. **Maintainability**: Clean architecture and code organization

The modular architecture allows for iterative development, starting with a minimal playable product and expanding features over time. The technology stack leverages modern web capabilities while maintaining the charm of retro gaming.

---

**Document Version**: 1.0  
**Last Updated**: March 1, 2026  
**Author**: Game Design Team  
**Status**: Ready for Implementation
