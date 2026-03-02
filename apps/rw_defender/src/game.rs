use crate::entities::{ActivePowerUp, Entity, EntityType, PowerUpType};
use crate::graphics::{background_by_index, RetroColors, Sprite, SpriteGenerator, ALL_BACKGROUNDS};
use crate::renderer::Renderer;
use crate::systems::InputState;
use crate::utils::{Rect, Vec2};
use rand::Rng;

// ── Constants ────────────────────────────────────────────────────────────────

pub const CANVAS_W: f64 = 640.0;
pub const CANVAS_H: f64 = 480.0;

const PLAYER_SPEED: f64 = 250.0;
const PLAYER_START_X: f64 = CANVAS_W / 2.0 - 8.0;
const PLAYER_START_Y: f64 = CANVAS_H - 40.0;
const PLAYER_FIRE_COOLDOWN: f64 = 0.25; // 250ms
const PLAYER_MAX_BULLETS: usize = 3;
const BULLET_SPEED: f64 = 450.0;
const BULLET_LIFETIME: f64 = 1.07; // ~480px at 450px/s
const INVULN_DURATION: f64 = 2.5;
const INVULN_FLASH_HZ: f64 = 8.0;

const ENEMY_GRUNT_HP: i32 = 15;
const GRUNT_SPEED: f64 = 40.0;
const FORMATION_COLS: u32 = 8;
#[allow(dead_code)]
const FORMATION_ROWS: u32 = 3;
const FORMATION_SPACING_X: f64 = 60.0;
const FORMATION_SPACING_Y: f64 = 40.0;
const FORMATION_START_Y: f64 = 30.0;

const ENEMY_BULLET_SPEED: f64 = 200.0;
const ENEMY_FIRE_INTERVAL: f64 = 0.5; // check every 500ms
const WAVE_TRANSITION_DURATION: f64 = 2.0;

const BOSS_AMPLITUDE: f64 = 100.0;
const BOSS_FREQ: f64 = 0.5; // Hz — matches spec
const BOSS_Y: f64 = 50.0; // fixed y in top third
const BOSS_FIRE_INTERVAL: f64 = 1.5;
const BOSS_BULLET_SPEED: f64 = 250.0;

const DIVER_SPEED: f64 = 80.0;
const DIVER_TRIGGER_RANGE: f64 = 80.0; // ±80px horizontal
const DIVER_RETURN_TIME: f64 = 2.0;
const DIVER_DIVE_COOLDOWN: f64 = 3.0;

const HIGH_SCORE_KEY: &str = "rw_defender_high_score";

// ── localStorage helpers ─────────────────────────────────────────────────────

fn load_high_score() -> u32 {
    let Ok(Some(storage)) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .map(|s| Ok::<_, ()>(Some(s)))
        .unwrap_or(Ok(None))
    else {
        return 0;
    };
    storage
        .get_item(HIGH_SCORE_KEY)
        .ok()
        .flatten()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0)
}

fn save_high_score(score: u32) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item(HIGH_SCORE_KEY, &score.to_string());
    }
}

// ── Game states ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    MainMenu,
    Playing,
    WaveTransition { timer: f64 },
    Paused,
    GameOver { timer: f64 },
}

// ── Sprite atlas ─────────────────────────────────────────────────────────────

pub struct SpriteAtlas {
    pub player: Sprite,
    pub player_thrust: Sprite,
    pub grunt: Sprite,
    pub weaver: Sprite,
    pub diver: Sprite,
    pub boss: Sprite,
    pub player_bullet: Sprite,
    pub enemy_bullet: Sprite,
    pub explosion_frames: Vec<Sprite>,
    pub powerup_colors: [u32; 6],
}

impl SpriteAtlas {
    pub fn new() -> Self {
        SpriteAtlas {
            player: SpriteGenerator::player_ship(),
            player_thrust: SpriteGenerator::player_ship_thrust(),
            grunt: SpriteGenerator::enemy_grunt(),
            weaver: SpriteGenerator::enemy_weaver(),
            diver: SpriteGenerator::enemy_diver(),
            boss: SpriteGenerator::enemy_boss(),
            player_bullet: SpriteGenerator::player_bullet(),
            enemy_bullet: SpriteGenerator::enemy_bullet(),
            explosion_frames: SpriteGenerator::explosion_frames(),
            powerup_colors: [
                RetroColors::ORANGE,   // TripleShot
                RetroColors::RED,      // ExplosiveShot
                RetroColors::YELLOW,   // RapidFire
                RetroColors::BLUE,     // LaserBeam
                RetroColors::GREEN,    // PiercingShot
                RetroColors::CYAN,     // Shield
            ],
        }
    }
}

// ── Player state ─────────────────────────────────────────────────────────────

pub struct Player {
    pub entity: Entity,
    pub lives: u32,
    pub score: u32,
    pub fire_cooldown: f64,
    pub active_powerup: Option<ActivePowerUp>,
    pub shields: u32,
    pub dead_timer: f64,    // 0 = alive, >0 = dying animation
    pub respawn_timer: f64, // countdown before respawn after death
    pub thrust_anim: f64,   // idle pulse timer
}

impl Player {
    pub fn new() -> Self {
        let entity = Entity::new(
            EntityType::Player,
            Vec2::new(PLAYER_START_X, PLAYER_START_Y),
            Rect::new(0.0, 0.0, 16.0, 16.0),
            30,
        );
        Player {
            entity,
            lives: 3,
            score: 0,
            fire_cooldown: 0.0,
            active_powerup: None,
            shields: 0,
            dead_timer: 0.0,
            respawn_timer: 0.0,
            thrust_anim: 0.0,
        }
    }

    pub fn respawn(&mut self) {
        self.entity.position = Vec2::new(PLAYER_START_X, PLAYER_START_Y);
        self.entity.active = true;
        self.entity.health = 30;
        self.entity.invuln_time = INVULN_DURATION;
        self.fire_cooldown = 0.0;
        self.dead_timer = 0.0;
        self.respawn_timer = 0.0;
    }

    pub fn is_alive(&self) -> bool {
        self.dead_timer <= 0.0 && self.respawn_timer <= 0.0
    }
}

// ── Main game struct ─────────────────────────────────────────────────────────

pub struct Game {
    pub state: GameState,
    pub player: Player,
    pub entities: Vec<Entity>,
    pub wave: u32,
    pub high_score: u32,
    /// Index into ALL_BACKGROUNDS for the current wave's background image.
    pub bg_index: usize,
    pub atlas: SpriteAtlas,
    pub enemy_fire_timer: f64,
    pub formation_dir: f64,
    pub formation_shift_timer: f64,
    pub spawn_queue: Vec<EntityType>,
    pub spawn_timer: f64,
    pub enemies_killed_this_wave: u32,
    pub wave_enemy_count: u32,
    rng: rand::rngs::SmallRng,
}

impl Game {
    pub fn new() -> Self {
        use rand::SeedableRng;
        Game {
            state: GameState::MainMenu,
            player: Player::new(),
            entities: Vec::with_capacity(200),
            wave: 0,
            high_score: load_high_score(),
            bg_index: 0,
            atlas: SpriteAtlas::new(),
            enemy_fire_timer: 0.0,
            formation_dir: 1.0,
            formation_shift_timer: 0.0,
            spawn_queue: Vec::new(),
            spawn_timer: 0.0,
            enemies_killed_this_wave: 0,
            wave_enemy_count: 0,
            rng: rand::rngs::SmallRng::seed_from_u64(42),
        }
    }

    // ── Public update entry ───────────────────────────────────────────────

    /// Filename of the current wave's background image (randomly chosen per wave).
    pub fn current_background(&self) -> &'static str {
        background_by_index(self.bg_index)
    }

    pub fn update(&mut self, input: &mut InputState, dt: f64) {
        let dt = dt.min(0.1); // cap delta time (prevent spiral of death)

        match self.state.clone() {
            GameState::MainMenu => {
                if input.consume_start() || input.fire_pressed {
                    input.fire_pressed = false;
                    self.start_game();
                }
            }
            GameState::Playing => {
                if input.consume_pause() {
                    self.state = GameState::Paused;
                    return;
                }
                self.update_playing(input, dt);
            }
            GameState::Paused => {
                if input.consume_pause() || input.consume_start() {
                    self.state = GameState::Playing;
                }
            }
            GameState::WaveTransition { timer } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    self.begin_wave();
                } else {
                    self.state = GameState::WaveTransition { timer: new_timer };
                }
            }
            GameState::GameOver { timer } => {
                let new_timer = timer - dt;
                if new_timer <= 0.0 {
                    if input.consume_start() || input.fire_pressed {
                        input.fire_pressed = false;
                        self.start_game();
                    } else {
                        self.state = GameState::GameOver { timer: 0.0 };
                    }
                } else {
                    self.state = GameState::GameOver { timer: new_timer };
                }
            }
        }
    }

    fn start_game(&mut self) {
        self.wave = 0;
        self.player = Player::new();
        self.entities.clear();
        self.enemy_fire_timer = 0.0;
        self.formation_dir = 1.0;
        self.formation_shift_timer = 0.0;
        self.enemies_killed_this_wave = 0;
        self.bg_index = self.rng.gen_range(0..ALL_BACKGROUNDS.len());
        self.state = GameState::WaveTransition { timer: WAVE_TRANSITION_DURATION };
    }

    fn begin_wave(&mut self) {
        self.wave += 1;
        self.bg_index = self.rng.gen_range(0..ALL_BACKGROUNDS.len());
        self.entities.retain(|e| matches!(e.entity_type, EntityType::Player));
        self.spawn_queue.clear();
        self.enemies_killed_this_wave = 0;

        if self.wave.is_multiple_of(5) {
            // Boss wave: single boss enemy
            self.spawn_queue.push(EntityType::Boss);
            self.wave_enemy_count = 1;
        } else {
            let count = self.wave_enemy_count();
            self.wave_enemy_count = count;
            for i in 0..count {
                let etype = self.enemy_type_for_wave(i);
                self.spawn_queue.push(etype);
            }
        }
        self.spawn_timer = 0.0;
        self.formation_shift_timer = 0.0;
        self.state = GameState::Playing;
    }

    fn wave_enemy_count(&self) -> u32 {
        (12 + (self.wave as f64 * 1.2) as u32).min(50)
    }

    fn enemy_type_for_wave(&mut self, _index: u32) -> EntityType {
        let roll: f64 = self.rng.gen();
        match self.wave {
            1 | 2 => EntityType::EnemyGrunt,
            3 | 4 => {
                if roll < 0.75 { EntityType::EnemyGrunt } else { EntityType::EnemyWeaver }
            }
            _ => {
                if roll < 0.50 {
                    EntityType::EnemyGrunt
                } else if roll < 0.85 {
                    EntityType::EnemyWeaver
                } else {
                    EntityType::EnemyDiver
                }
            }
        }
    }

    fn speed_multiplier(&self) -> f64 {
        (1.0_f64 + self.wave as f64 * 0.05).min(2.0)
    }

    // ── Playing update ────────────────────────────────────────────────────

    fn update_playing(&mut self, input: &InputState, dt: f64) {
        self.update_player(input, dt);
        self.update_spawning(dt);
        self.update_enemies(dt);
        self.update_bullets(dt);
        self.update_explosions(dt);
        self.update_powerups(dt);
        self.check_collisions();
        self.check_wave_complete();
    }

    fn update_player(&mut self, input: &InputState, dt: f64) {
        // Handle death/respawn timers
        if self.player.dead_timer > 0.0 {
            self.player.dead_timer -= dt;
            if self.player.dead_timer <= 0.0 {
                self.player.dead_timer = 0.0;
                self.player.respawn_timer = 1.0;
            }
            return;
        }
        if self.player.respawn_timer > 0.0 {
            self.player.respawn_timer -= dt;
            if self.player.respawn_timer <= 0.0 {
                if self.player.lives > 0 {
                    self.player.lives -= 1;
                    self.player.respawn();
                } else {
                    self.state = GameState::GameOver { timer: 3.0 };
                }
            }
            return;
        }

        let p = &mut self.player;

        // Movement
        let mut vel_x = 0.0;
        if input.left_pressed { vel_x -= PLAYER_SPEED; }
        if input.right_pressed { vel_x += PLAYER_SPEED; }

        p.entity.position.x += vel_x * dt;
        p.entity.position.x = p.entity.position.x.clamp(0.0, CANVAS_W - 16.0);

        // Invulnerability countdown
        if p.entity.invuln_time > 0.0 {
            p.entity.invuln_time -= dt;
        }

        // Power-up tick
        if let Some(ref mut pu) = p.active_powerup {
            pu.tick(dt);
            if pu.is_expired() {
                p.active_powerup = None;
            }
        }

        // Fire cooldown
        if p.fire_cooldown > 0.0 {
            p.fire_cooldown -= dt;
        }

        // Shooting
        let fire_cooldown = match &p.active_powerup {
            Some(pu) if pu.power_up_type == PowerUpType::RapidFire => 0.083,
            _ => PLAYER_FIRE_COOLDOWN,
        };
        let max_bullets = match &p.active_powerup {
            Some(pu) if pu.power_up_type == PowerUpType::RapidFire => 6,
            _ => PLAYER_MAX_BULLETS,
        };
        let player_bullet_count = self.entities.iter()
            .filter(|e| e.active && matches!(e.entity_type, EntityType::PlayerBullet))
            .count();

        if input.fire_pressed
            && p.fire_cooldown <= 0.0
            && player_bullet_count < max_bullets
        {
            let px = p.entity.position.x + 6.0;
            let py = p.entity.position.y;

            let is_triple = p.active_powerup.as_ref()
                .map(|pu| pu.power_up_type == PowerUpType::TripleShot)
                .unwrap_or(false);
            let is_piercing = p.active_powerup.as_ref()
                .map(|pu| pu.power_up_type == PowerUpType::PiercingShot)
                .unwrap_or(false);
            let is_explosive = p.active_powerup.as_ref()
                .map(|pu| pu.power_up_type == PowerUpType::ExplosiveShot)
                .unwrap_or(false);

            // We'll tag piercing/explosive in sprite_index for now (hack until weapons module)
            let tag = if is_piercing { 1 } else if is_explosive { 2 } else { 0 };

            self.spawn_player_bullet(px, py, Vec2::new(0.0, -BULLET_SPEED), tag);

            if is_triple {
                let angle = 15.0_f64.to_radians();
                let vx = BULLET_SPEED * angle.sin();
                let vy = -BULLET_SPEED * angle.cos();
                self.spawn_player_bullet(px, py, Vec2::new(-vx, vy), tag);
                self.spawn_player_bullet(px, py, Vec2::new(vx, vy), tag);
            }

            self.player.fire_cooldown = fire_cooldown;
        }

        // Thrust animation
        self.player.thrust_anim += dt;
    }

    fn spawn_player_bullet(&mut self, x: f64, y: f64, vel: Vec2, tag: usize) {
        let mut bullet = Entity::new(
            EntityType::PlayerBullet,
            Vec2::new(x, y),
            Rect::new(0.5, 0.5, 3.0, 7.0),
            1,
        );
        bullet.velocity = vel;
        bullet.lifetime = BULLET_LIFETIME;
        bullet.sprite_index = tag;
        self.entities.push(bullet);
    }

    fn update_spawning(&mut self, dt: f64) {
        if self.spawn_queue.is_empty() {
            return;
        }
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            if let Some(etype) = self.spawn_queue.pop() {
                let idx = self.wave_enemy_count as usize - self.spawn_queue.len() - 1;
                self.spawn_enemy(etype, idx);
            }
            self.spawn_timer = 0.2; // 200ms between spawns
        }
    }

    fn spawn_enemy(&mut self, etype: EntityType, idx: usize) {
        let (x, y, hp, hitbox) = match &etype {
            EntityType::Boss => {
                let boss_hp = 75 + (self.wave / 5) as i32 * 15;
                (CANVAS_W / 2.0 - 24.0, BOSS_Y, boss_hp, Rect::new(2.0, 2.0, 44.0, 28.0))
            }
            EntityType::EnemyGrunt | EntityType::EnemyWeaver | EntityType::EnemyDiver => {
                let col = (idx % FORMATION_COLS as usize) as f64;
                let row = (idx / FORMATION_COLS as usize) as f64;
                let x = 40.0 + col * FORMATION_SPACING_X;
                let y = FORMATION_START_Y + row * FORMATION_SPACING_Y;
                (x, y, ENEMY_GRUNT_HP, Rect::new(1.0, 1.0, 12.0, 10.0))
            }
            _ => {
                let col = (idx % FORMATION_COLS as usize) as f64;
                let row = (idx / FORMATION_COLS as usize) as f64;
                let x = 40.0 + col * FORMATION_SPACING_X;
                let y = FORMATION_START_Y + row * FORMATION_SPACING_Y;
                (x, y, 1, Rect::new(0.0, 0.0, 14.0, 12.0))
            }
        };

        let mut e = Entity::new(etype, Vec2::new(x, y), hitbox, hp);
        e.phase = (idx as f64) * 0.3; // stagger oscillation phases
        e.fire_cooldown = self.rng.gen_range(0.5..2.0); // random initial fire delay
        self.entities.push(e);
    }

    fn update_enemies(&mut self, dt: f64) {
        let speed = GRUNT_SPEED * self.speed_multiplier();
        let descend = speed * dt;

        // Formation lateral shift (non-boss enemies only)
        self.formation_shift_timer += dt;
        let shift_x = self.formation_dir * 30.0 * dt;

        // Check if any non-boss enemy would hit the edge
        let any_at_edge = self.entities.iter()
            .filter(|e| e.active && is_regular_enemy(&e.entity_type))
            .any(|e| {
                let nx = e.position.x + shift_x;
                !(32.0..=(CANVAS_W - 50.0)).contains(&nx)
            });
        if any_at_edge {
            self.formation_dir *= -1.0;
        }
        let shift_x = self.formation_dir * 30.0 * dt; // recalc after possible flip

        // Enemy fire check
        self.enemy_fire_timer += dt;
        let should_fire = self.enemy_fire_timer >= ENEMY_FIRE_INTERVAL;
        if should_fire {
            self.enemy_fire_timer -= ENEMY_FIRE_INTERVAL;
        }

        let player_x = self.player.entity.position.x + 8.0;
        let player_y = self.player.entity.position.y;
        let wave = self.wave;
        let proj_speed = ENEMY_BULLET_SPEED * (1.0_f64 + wave as f64 * 0.03).min(1.5);

        // Collect fire positions before mutating entities.
        // Tuple: (x, y, aimed, is_boss_spread)
        let mut fire_positions: Vec<(f64, f64, bool, bool)> = Vec::new();

        for e in self.entities.iter_mut() {
            if !e.active || !is_enemy(&e.entity_type) {
                continue;
            }

            // ── Boss movement ─────────────────────────────────────────────
            if matches!(e.entity_type, EntityType::Boss) {
                e.phase += dt;
                // Sine-wave lateral movement across top third
                let center_x = CANVAS_W / 2.0 - 24.0;
                e.position.x = center_x + BOSS_AMPLITUDE * (e.phase * BOSS_FREQ * std::f64::consts::TAU).sin();
                e.position.y = BOSS_Y; // fixed height

                // Boss fire: spread shot every BOSS_FIRE_INTERVAL
                e.fire_cooldown -= dt;
                if e.fire_cooldown <= 0.0 {
                    fire_positions.push((e.position.x + 24.0, e.position.y + 32.0, false, true));
                    e.fire_cooldown = BOSS_FIRE_INTERVAL;
                }
                continue;
            }

            // ── Diver dive behavior ───────────────────────────────────────
            if matches!(e.entity_type, EntityType::EnemyDiver) {
                e.phase += dt;

                if e.dive_timer > 0.0 {
                    // Actively diving: straight down at DIVER_SPEED
                    e.dive_timer -= dt;
                    e.position.y += DIVER_SPEED * dt;

                    if e.dive_timer <= 0.0 || e.position.y > CANVAS_H - 32.0 {
                        // End dive — mark inactive if off screen, else return to formation
                        if e.position.y > CANVAS_H - 32.0 {
                            e.active = false; // missed dive, went off screen
                        } else {
                            e.dive_timer = 0.0;
                            e.lifetime = DIVER_DIVE_COOLDOWN; // cooldown stored in lifetime
                        }
                    }
                    continue;
                }

                // Normal diver movement (same as grunt while waiting to dive)
                e.position.y += descend;
                e.position.x += shift_x;

                // Wave oscillation (wave 4+)
                if wave >= 4 {
                    let osc = (e.phase * std::f64::consts::PI / 1.5).sin();
                    e.position.x += osc * 60.0 * dt * 0.333;
                }
                e.position.x = e.position.x.clamp(0.0, CANVAS_W - 20.0);

                // Tick dive cooldown (stored in lifetime)
                if e.lifetime > 0.0 {
                    e.lifetime -= dt;
                }

                // Trigger dive if player within horizontal range and cooldown expired
                if e.lifetime <= 0.0 && (e.position.x - player_x).abs() < DIVER_TRIGGER_RANGE {
                    let pseudo_roll = (e.phase * 73.131).fract().abs();
                    if pseudo_roll < 0.30 {
                        e.dive_timer = DIVER_RETURN_TIME;
                        e.lifetime = 0.0;
                    }
                }

                // Normal fire (wave 3+)
                if wave >= 3 {
                    e.fire_cooldown -= dt;
                    if should_fire && e.fire_cooldown <= 0.0 {
                        let fire_chance = 0.05 + wave as f64 * 0.005;
                        let pseudo_roll = (e.phase * 137.508).fract().abs();
                        if pseudo_roll < fire_chance {
                            fire_positions.push((e.position.x + 5.0, e.position.y + 12.0, false, false));
                            e.fire_cooldown = 2.0 / (1.0 + wave as f64 * 0.1);
                        }
                    }
                }

                if e.position.y > CANVAS_H - 32.0 {
                    self.state = GameState::GameOver { timer: 3.0 };
                }
                continue;
            }

            // ── Normal enemy (Grunt / Weaver) ─────────────────────────────
            e.position.y += descend;
            e.position.x += shift_x;
            e.phase += dt;

            // Wave oscillation (wave 4+)
            if wave >= 4 {
                let osc = (e.phase * std::f64::consts::PI / 1.5).sin();
                e.position.x += osc * 60.0 * dt * 0.333;
            }

            // Weaver zigzag
            if matches!(e.entity_type, EntityType::EnemyWeaver) {
                let zz = (e.phase * std::f64::consts::PI * 2.0 * 1.5).sin();
                e.position.x += zz * 40.0 * dt;
            }

            // Clamp to screen horizontally
            e.position.x = e.position.x.clamp(0.0, CANVAS_W - 20.0);

            // Fire cooldown and shooting (wave 3+)
            if wave >= 3 {
                e.fire_cooldown -= dt;
                if should_fire && e.fire_cooldown <= 0.0 {
                    let fire_chance = 0.05 + wave as f64 * 0.005;
                    // Pseudo-random roll using entity phase as entropy (avoids borrow conflict with self.rng)
                    let pseudo_roll = (e.phase * 137.508).fract().abs();
                    if pseudo_roll < fire_chance {
                        let aimed = (e.phase * 31.0).fract() < 0.3; // ~30% aimed
                        fire_positions.push((e.position.x + 5.0, e.position.y + 12.0, aimed, false));
                        e.fire_cooldown = 2.0 / (1.0 + wave as f64 * 0.1);
                    }
                }
            }

            // Check if reached bottom
            if e.position.y > CANVAS_H - 32.0 {
                // Enemy reached bottom - game over
                self.state = GameState::GameOver { timer: 3.0 };
            }
        }

        // Spawn enemy bullets
        let existing = self.entities.iter()
            .filter(|e| e.active && matches!(e.entity_type, EntityType::EnemyBullet))
            .count();
        let max_enemy_bullets = (10 + wave as usize).min(30);

        for (bx, by, aimed, is_boss) in fire_positions {
            if existing >= max_enemy_bullets { break; }

            if is_boss {
                // 3-shot spread: straight, ±20°
                for angle_deg in [-20.0_f64, 0.0, 20.0] {
                    let angle = angle_deg.to_radians();
                    let vel = Vec2::new(angle.sin() * BOSS_BULLET_SPEED, angle.cos() * BOSS_BULLET_SPEED);
                    let mut bullet = Entity::new(
                        EntityType::EnemyBullet,
                        Vec2::new(bx, by),
                        Rect::new(0.5, 0.5, 5.0, 5.0),
                        1,
                    );
                    bullet.velocity = vel;
                    bullet.lifetime = 3.0;
                    self.entities.push(bullet);
                }
            } else {
                let vel = if aimed {
                    let dx = player_x - bx;
                    let dy = player_y - by;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);
                    Vec2::new(dx / len * proj_speed, dy / len * proj_speed)
                } else {
                    Vec2::new(0.0, proj_speed)
                };
                let mut bullet = Entity::new(
                    EntityType::EnemyBullet,
                    Vec2::new(bx, by),
                    Rect::new(0.5, 0.5, 3.0, 3.0),
                    1,
                );
                bullet.velocity = vel;
                bullet.lifetime = 3.0;
                self.entities.push(bullet);
            }
        }
    }

    fn update_bullets(&mut self, dt: f64) {
        for e in self.entities.iter_mut() {
            if !e.active {
                continue;
            }
            if !matches!(e.entity_type, EntityType::PlayerBullet | EntityType::EnemyBullet) {
                continue;
            }
            e.position += e.velocity * dt;
            e.lifetime -= dt;
            if e.lifetime <= 0.0
                || e.position.y < -10.0
                || e.position.y > CANVAS_H + 10.0
                || e.position.x < -10.0
                || e.position.x > CANVAS_W + 10.0
            {
                e.active = false;
            }
        }
    }

    fn update_explosions(&mut self, dt: f64) {
        for e in self.entities.iter_mut() {
            if !e.active || !matches!(e.entity_type, EntityType::Explosion) {
                continue;
            }
            e.anim_elapsed += dt;
            let frame_dur = 0.05; // 50ms per frame
            if e.anim_elapsed >= frame_dur {
                e.anim_elapsed -= frame_dur;
                e.anim_frame += 1;
                if e.anim_frame >= 6 {
                    e.active = false;
                }
            }
        }
    }

    fn update_powerups(&mut self, dt: f64) {
        for e in self.entities.iter_mut() {
            if !e.active || !matches!(e.entity_type, EntityType::PowerUp(_)) {
                continue;
            }
            e.position.y += 80.0 * dt; // float down at 80px/s
            e.lifetime -= dt;
            if e.lifetime <= 0.0 || e.position.y > CANVAS_H + 20.0 {
                e.active = false;
            }
        }
    }

    // ── Collision detection ───────────────────────────────────────────────

    fn check_collisions(&mut self) {
        if !self.player.is_alive() {
            return;
        }

        let player_hb = self.player.entity.world_hitbox();
        let player_invuln = self.player.entity.is_invulnerable();

        let mut to_kill: Vec<usize> = Vec::new();
        let mut score_gain: u32 = 0;
        let mut spawn_explosions: Vec<Vec2> = Vec::new();
        let mut spawn_powerups: Vec<Vec2> = Vec::new();
        let mut player_hit = false;
        let mut collected_powerup: Option<(PowerUpType, f64)> = None;

        for (i, e) in self.entities.iter().enumerate() {
            if !e.active { continue; }

            match &e.entity_type {
                EntityType::PlayerBullet => {
                    let bhb = e.world_hitbox();
                    // Check vs enemies
                    for (j, target) in self.entities.iter().enumerate() {
                        if !target.active || i == j { continue; }
                        if !is_enemy(&target.entity_type) { continue; }
                        if bhb.intersects(&target.world_hitbox()) {
                            to_kill.push(i); // bullet
                            to_kill.push(j); // enemy
                            score_gain += enemy_score_value(&target.entity_type, self.wave);
                            spawn_explosions.push(target.position);
                            // Chance to spawn powerup
                            spawn_powerups.push(target.position);
                        }
                    }
                }
                EntityType::EnemyBullet => {
                    if !player_invuln && e.world_hitbox().intersects(&player_hb) {
                        to_kill.push(i);
                        player_hit = true;
                    }
                }
                etype if is_enemy(etype) => {
                    if !player_invuln && e.world_hitbox().intersects(&player_hb) {
                        to_kill.push(i);
                        player_hit = true;
                        spawn_explosions.push(e.position);
                    }
                }
                EntityType::PowerUp(ptype) => {
                    let collect_radius = 16.0;
                    let dist = e.center().distance_to(self.player.entity.center());
                    if dist <= collect_radius {
                        to_kill.push(i);
                        score_gain += 25;
                        let duration = powerup_duration(ptype);
                        collected_powerup = Some((ptype.clone(), duration));
                    }
                }
                _ => {}
            }
        }

        // Apply kills
        to_kill.sort_unstable();
        to_kill.dedup();
        for idx in &to_kill {
            if let Some(e) = self.entities.get_mut(*idx) {
                if is_enemy(&e.entity_type) {
                    self.enemies_killed_this_wave += 1;
                }
                e.active = false;
            }
        }

        // Spawn explosions
        for pos in spawn_explosions {
            let mut exp = Entity::new(
                EntityType::Explosion,
                pos,
                Rect::new(0.0, 0.0, 16.0, 16.0),
                1,
            );
            exp.anim_frame = 0;
            exp.anim_elapsed = 0.0;
            self.entities.push(exp);
        }

        // Try to spawn power-ups (probabilistic)
        for pos in spawn_powerups {
            let roll: f64 = self.rng.gen();
            if roll < 0.08 {
                self.spawn_powerup_at(pos);
            }
        }

        // Apply score
        self.player.score += score_gain;
        if self.player.score > self.high_score {
            self.high_score = self.player.score;
            save_high_score(self.high_score);
        }

        // Apply collected power-up
        if let Some((ptype, duration)) = collected_powerup {
            if matches!(ptype, PowerUpType::Shield) {
                self.player.shields = (self.player.shields + 1).min(3);
            } else {
                self.player.active_powerup = Some(ActivePowerUp::new(ptype, duration));
            }
        }

        // Apply player hit
        if player_hit {
            if self.player.shields > 0 {
                self.player.shields -= 1;
                self.player.entity.invuln_time = 0.5; // brief invuln after shield absorb
            } else {
                self.player.dead_timer = 0.5;
                self.player.entity.active = false;
                // Spawn player explosion
                let pos = self.player.entity.position;
                let exp = Entity::new(EntityType::Explosion, pos, Rect::new(0.0, 0.0, 16.0, 16.0), 1);
                self.entities.push(exp);
            }
        }
    }

    fn spawn_powerup_at(&mut self, pos: Vec2) {
        let roll: f64 = self.rng.gen();
        let ptype = match (roll * 6.0) as u32 {
            0 => PowerUpType::TripleShot,
            1 => PowerUpType::ExplosiveShot,
            2 => PowerUpType::RapidFire,
            3 => PowerUpType::LaserBeam,
            4 => PowerUpType::PiercingShot,
            _ => PowerUpType::Shield,
        };
        let mut e = Entity::new(
            EntityType::PowerUp(ptype),
            pos,
            Rect::new(0.0, 0.0, 12.0, 12.0),
            1,
        );
        e.lifetime = 6.0;
        self.entities.push(e);
    }

    fn check_wave_complete(&mut self) {
        if !matches!(self.state, GameState::Playing) {
            return;
        }
        let enemies_remaining = self.entities.iter()
            .filter(|e| e.active && is_enemy(&e.entity_type))
            .count();
        let spawns_left = self.spawn_queue.len();

        if enemies_remaining == 0 && spawns_left == 0 && self.wave_enemy_count > 0 {
            // Wave clear bonus
            self.player.score += 100 * self.wave;
            self.state = GameState::WaveTransition { timer: WAVE_TRANSITION_DURATION };
        }
    }

    // ── Rendering ─────────────────────────────────────────────────────────

    pub fn render(&self, renderer: &Renderer) {
        renderer.clear();

        match &self.state {
            GameState::MainMenu => self.render_main_menu(renderer),
            GameState::Playing | GameState::Paused => {
                self.render_game(renderer);
                if matches!(self.state, GameState::Paused) {
                    self.render_pause_overlay(renderer);
                }
            }
            GameState::WaveTransition { timer } => {
                self.render_game(renderer);
                self.render_wave_transition(renderer, *timer);
            }
            GameState::GameOver { timer } => {
                self.render_game(renderer);
                self.render_game_over(renderer, *timer);
            }
        }
    }

    fn render_game(&self, renderer: &Renderer) {
        // Enemies
        for e in &self.entities {
            if !e.active { continue; }
            match &e.entity_type {
                EntityType::EnemyGrunt =>
                    self.atlas.grunt.draw(&renderer.ctx, e.position.x, e.position.y),
                EntityType::EnemyWeaver =>
                    self.atlas.weaver.draw(&renderer.ctx, e.position.x, e.position.y),
                EntityType::EnemyDiver =>
                    self.atlas.diver.draw(&renderer.ctx, e.position.x, e.position.y),
                EntityType::Boss =>
                    self.atlas.boss.draw(&renderer.ctx, e.position.x, e.position.y),
                EntityType::PlayerBullet =>
                    self.atlas.player_bullet.draw(&renderer.ctx, e.position.x, e.position.y),
                EntityType::EnemyBullet =>
                    self.atlas.enemy_bullet.draw(&renderer.ctx, e.position.x, e.position.y),
                EntityType::Explosion => {
                    if let Some(frame) = self.atlas.explosion_frames.get(e.anim_frame) {
                        let offset = frame.width as f64 / 2.0;
                        frame.draw(&renderer.ctx, e.position.x - offset, e.position.y - offset);
                    }
                }
                EntityType::PowerUp(ptype) => {
                    let color = self.atlas.powerup_colors[powerup_index(ptype)];
                    // Blink in last 2 seconds
                    let blink = (e.lifetime * 4.0) as u32;
                    let should_draw = e.lifetime > 2.0 || blink.is_multiple_of(2);
                    if should_draw {
                        let pu_sprite = SpriteGenerator::powerup_sprite(color);
                        pu_sprite.draw(&renderer.ctx, e.position.x, e.position.y);
                    }
                }
                _ => {}
            }
        }

        // Player
        if self.player.is_alive() || self.player.respawn_timer > 0.0 {
            let show = if self.player.entity.is_invulnerable() {
                let t = (self.player.entity.invuln_time * INVULN_FLASH_HZ) as u32;
                t.is_multiple_of(2)
            } else {
                true
            };
            if show && self.player.entity.active {
                let thrust_t = (self.player.thrust_anim * 6.0) as u32;
                let use_thrust = thrust_t.is_multiple_of(2);
                let sprite = if use_thrust { &self.atlas.player_thrust } else { &self.atlas.player };
                sprite.draw(&renderer.ctx, self.player.entity.position.x, self.player.entity.position.y);
            }
        }

        // HUD
        self.render_hud(renderer);
    }

    fn render_hud(&self, renderer: &Renderer) {
        // Score
        renderer.draw_text(
            &format!("SCORE {:06}", self.player.score),
            8.0, 16.0, RetroColors::WHITE, 12,
        );
        // High score
        renderer.draw_text(
            &format!("HI    {:06}", self.high_score),
            8.0, 30.0, RetroColors::YELLOW, 10,
        );
        // Wave
        renderer.draw_text_centered(
            &format!("WAVE {}", self.wave),
            16.0, RetroColors::CYAN, 12,
        );
        // Lives (draw small ship icons)
        for i in 0..self.player.lives {
            let x = CANVAS_W - 20.0 - i as f64 * 18.0;
            renderer.fill_rect(x + 3.0, 4.0, 6.0, 8.0, RetroColors::BLUE);
            renderer.fill_rect(x + 5.0, 2.0, 2.0, 3.0, RetroColors::CYAN);
        }
        // Shields
        for i in 0..self.player.shields {
            let x = CANVAS_W - 20.0 - i as f64 * 14.0;
            renderer.draw_text("S", x, 32.0, RetroColors::CYAN, 10);
        }
        // Active power-up
        if let Some(ref pu) = self.player.active_powerup {
            let color = self.atlas.powerup_colors[powerup_index(&pu.power_up_type)];
            let bar_color = if pu.fraction() > 0.66 {
                RetroColors::GREEN
            } else if pu.fraction() > 0.33 {
                RetroColors::YELLOW
            } else {
                RetroColors::RED
            };
            renderer.fill_rect(4.0, CANVAS_H - 24.0, 12.0, 12.0, color);
            renderer.draw_bar(20.0, CANVAS_H - 20.0, 60.0, 8.0, pu.fraction(), bar_color);
        }
    }

    fn render_main_menu(&self, renderer: &Renderer) {
        renderer.draw_text_centered("RW DEFENDER", 160.0, RetroColors::CYAN, 24);
        renderer.draw_text_centered("ARCADE SHOOTER", 190.0, RetroColors::YELLOW, 14);
        renderer.draw_text_centered("PRESS ENTER OR FIRE TO START", 280.0, RetroColors::WHITE, 12);
        renderer.draw_text_centered("ARROW KEYS / WASD TO MOVE", 310.0, RetroColors::GRAY, 10);
        renderer.draw_text_centered("SPACE / W TO FIRE", 326.0, RetroColors::GRAY, 10);
        renderer.draw_text_centered("ESC TO PAUSE", 342.0, RetroColors::GRAY, 10);
        renderer.draw_text_centered(
            &format!("HI SCORE {:06}", self.high_score),
            380.0, RetroColors::YELLOW, 12,
        );
    }

    fn render_wave_transition(&self, renderer: &Renderer, timer: f64) {
        let alpha = ((WAVE_TRANSITION_DURATION - timer) / 0.3).min(1.0);
        renderer.set_alpha(alpha);
        renderer.draw_text_centered(
            &format!("WAVE {}", self.wave + 1),
            CANVAS_H / 2.0 - 16.0, RetroColors::CYAN, 28,
        );
        renderer.reset_alpha();
    }

    fn render_pause_overlay(&self, renderer: &Renderer) {
        renderer.set_alpha(0.5);
        renderer.fill_rect(0.0, 0.0, CANVAS_W, CANVAS_H, RetroColors::BLACK);
        renderer.reset_alpha();
        renderer.draw_text_centered("PAUSED", CANVAS_H / 2.0 - 20.0, RetroColors::YELLOW, 24);
        renderer.draw_text_centered("PRESS ESC TO RESUME", CANVAS_H / 2.0 + 10.0, RetroColors::WHITE, 12);
    }

    fn render_game_over(&self, renderer: &Renderer, timer: f64) {
        let elapsed = 3.0 - timer;
        if elapsed < 1.0 { return; } // black screen for 1s
        let alpha = ((elapsed - 1.0) / 0.5).min(1.0);
        renderer.set_alpha(alpha * 0.6);
        renderer.fill_rect(0.0, 0.0, CANVAS_W, CANVAS_H, RetroColors::BLACK);
        renderer.reset_alpha();
        renderer.set_alpha(alpha);
        renderer.draw_text_centered("GAME OVER", CANVAS_H / 2.0 - 30.0, RetroColors::RED, 28);
        if elapsed > 1.5 {
            renderer.draw_text_centered(
                &format!("SCORE {:06}", self.player.score),
                CANVAS_H / 2.0 + 10.0, RetroColors::WHITE, 16,
            );
        }
        if elapsed > 2.0 {
            renderer.draw_text_centered(
                &format!("HI SCORE {:06}", self.high_score),
                CANVAS_H / 2.0 + 32.0, RetroColors::YELLOW, 14,
            );
        }
        if timer <= 0.0 {
            renderer.draw_text_centered("PRESS ENTER TO PLAY AGAIN", CANVAS_H / 2.0 + 60.0, RetroColors::GREEN, 10);
        }
        renderer.reset_alpha();
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

pub fn is_enemy(etype: &EntityType) -> bool {
    matches!(
        etype,
        EntityType::EnemyGrunt | EntityType::EnemyWeaver | EntityType::EnemyDiver | EntityType::Boss
    )
}

/// Returns true for regular (non-boss) enemies — used for formation logic.
fn is_regular_enemy(etype: &EntityType) -> bool {
    matches!(
        etype,
        EntityType::EnemyGrunt | EntityType::EnemyWeaver | EntityType::EnemyDiver
    )
}

fn enemy_score_value(etype: &EntityType, wave: u32) -> u32 {
    match etype {
        EntityType::EnemyGrunt => 10,
        EntityType::EnemyWeaver => 20,
        EntityType::EnemyDiver => 30,
        EntityType::Boss => 100 + wave * 20,
        _ => 0,
    }
}

fn powerup_duration(ptype: &PowerUpType) -> f64 {
    match ptype {
        PowerUpType::TripleShot | PowerUpType::RapidFire | PowerUpType::PiercingShot => 12.0,
        PowerUpType::ExplosiveShot => 15.0,
        PowerUpType::LaserBeam => 10.0,
        PowerUpType::Shield => 30.0,
    }
}

fn powerup_index(ptype: &PowerUpType) -> usize {
    match ptype {
        PowerUpType::TripleShot => 0,
        PowerUpType::ExplosiveShot => 1,
        PowerUpType::RapidFire => 2,
        PowerUpType::LaserBeam => 3,
        PowerUpType::PiercingShot => 4,
        PowerUpType::Shield => 5,
    }
}
