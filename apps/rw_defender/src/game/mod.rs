mod constants;
mod player;
mod render;
mod update;

pub use constants::{CANVAS_H, CANVAS_W};
pub use player::{Player, SpriteAtlas};

use crate::entities::{Entity, EntityType, PowerUpType};
use crate::graphics::{background_by_index, ALL_BACKGROUNDS};
use crate::renderer::Renderer;
use crate::systems::InputState;
use constants::{load_high_score, WAVE_TRANSITION_DURATION};
use rand::Rng;

// ── Game state ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    MainMenu,
    Playing,
    WaveTransition { timer: f64 },
    Paused,
    GameOver { timer: f64 },
}

// ── Main game struct ──────────────────────────────────────────────────────────

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
    pub(crate) rng: rand::rngs::SmallRng,
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

    /// Filename of the current wave's background image (randomly chosen per wave).
    pub fn current_background(&self) -> &'static str {
        background_by_index(self.bg_index)
    }

    pub fn update(&mut self, input: &mut InputState, dt: f64) {
        let dt = dt.min(0.1); // cap delta time to prevent spiral of death

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
        // Wave 1 → 6 enemies, grows ~1.5 per wave, capped at 49
        (5 + (self.wave as f64 * 1.5) as u32).min(49)
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

    /// Speed scalar that increases 5% per wave, capped at 2×.
    pub(crate) fn speed_multiplier(&self) -> f64 {
        (1.0_f64 + self.wave as f64 * 0.05).min(2.0)
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

pub fn is_enemy(etype: &EntityType) -> bool {
    matches!(
        etype,
        EntityType::EnemyGrunt | EntityType::EnemyWeaver | EntityType::EnemyDiver | EntityType::Boss
    )
}

pub(crate) fn powerup_duration(ptype: &PowerUpType) -> f64 {
    match ptype {
        PowerUpType::TripleShot | PowerUpType::RapidFire | PowerUpType::PiercingShot => 12.0,
        PowerUpType::ExplosiveShot => 15.0,
        PowerUpType::LaserBeam => 10.0,
        PowerUpType::Shield => 30.0,
        PowerUpType::ExtraLife => 0.0, // instant — no timer
    }
}

pub(crate) fn powerup_index(ptype: &PowerUpType) -> usize {
    match ptype {
        PowerUpType::TripleShot => 0,
        PowerUpType::ExplosiveShot => 1,
        PowerUpType::RapidFire => 2,
        PowerUpType::LaserBeam => 3,
        PowerUpType::PiercingShot => 4,
        PowerUpType::Shield => 5,
        PowerUpType::ExtraLife => 6,
    }
}
