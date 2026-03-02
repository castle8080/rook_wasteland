use super::constants::{INVULN_DURATION, PLAYER_START_X, PLAYER_START_Y};
use crate::entities::{ActivePowerUp, Entity, EntityType};
use crate::graphics::{RetroColors, Sprite, SpriteGenerator};
use crate::utils::{Rect, Vec2};

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
    pub powerup_colors: [u32; 7],
}

impl SpriteAtlas {
    pub fn new() -> Self {
        SpriteAtlas {
            // Ship sprite is 16×16 px; scale=2 makes it 32×32 visual — comparable to enemies.
            player: SpriteGenerator::player_ship().with_scale(2),
            player_thrust: SpriteGenerator::player_ship_thrust().with_scale(2),
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
                RetroColors::MAGENTA,  // ExtraLife
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
            // Hitbox inset slightly from the 32×32 visual (2× scale of 16×16 sprite)
            Rect::new(2.0, 2.0, 28.0, 28.0),
            30,
        );
        Player {
            entity,
            lives: 5,
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
