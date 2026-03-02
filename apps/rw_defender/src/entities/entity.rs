use crate::utils::{Rect, Vec2};

/// All entity types in the game.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum EntityType {
    Player,
    EnemyGrunt,
    EnemyWeaver,
    EnemyDiver,
    Boss,
    PlayerBullet,
    EnemyBullet,
    Explosion,
    /// Large blast triggered when an enemy reaches the screen bottom.
    /// Damages the player and nearby enemies within GROUND_EXPLOSION_RADIUS.
    GroundExplosion,
    PowerUp(PowerUpType),
}

/// Available power-up types.
#[derive(Debug, Clone, PartialEq)]
pub enum PowerUpType {
    TripleShot,
    ExplosiveShot,
    RapidFire,
    LaserBeam,
    PiercingShot,
    Shield,
    /// Instantly grants +1 life (up to 9).
    ExtraLife,
}

/// A currently active player power-up with remaining duration.
#[derive(Debug, Clone)]
pub struct ActivePowerUp {
    pub power_up_type: PowerUpType,
    pub remaining_time: f64,
    pub total_time: f64,
}

impl ActivePowerUp {
    pub fn new(power_up_type: PowerUpType, duration: f64) -> Self {
        ActivePowerUp {
            power_up_type,
            remaining_time: duration,
            total_time: duration,
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.remaining_time = (self.remaining_time - dt).max(0.0);
    }

    pub fn is_expired(&self) -> bool {
        self.remaining_time <= 0.0
    }

    /// Fraction of time remaining (1.0 = full, 0.0 = empty).
    pub fn fraction(&self) -> f64 {
        if self.total_time <= 0.0 {
            return 0.0;
        }
        self.remaining_time / self.total_time
    }
}

/// Core game entity. Uses composition over inheritance.
pub struct Entity {
    pub position: Vec2,
    pub velocity: Vec2,
    /// Hitbox relative to position (offset from top-left of sprite).
    pub hitbox: Rect,
    pub entity_type: EntityType,
    pub health: i32,
    pub active: bool,
    /// Sprite index into the SpriteAtlas.
    pub sprite_index: usize,
    /// For animated entities (explosions, etc.) — frame elapsed time.
    pub anim_elapsed: f64,
    pub anim_frame: usize,
    /// General-purpose timer (bullets: lifetime; powerups: float lifetime).
    pub lifetime: f64,
    /// Invulnerability timer (player only).
    pub invuln_time: f64,
    /// For enemies: tracks individual fire cooldown.
    pub fire_cooldown: f64,
    /// For divers: tracks dive state.
    #[allow(dead_code)]
    pub dive_timer: f64,
    /// For bosses / weavers: tracks oscillation phase.
    pub phase: f64,
}

impl Entity {
    pub fn new(entity_type: EntityType, position: Vec2, hitbox: Rect, health: i32) -> Self {
        Entity {
            position,
            velocity: Vec2::ZERO,
            hitbox,
            entity_type,
            health,
            active: true,
            sprite_index: 0,
            anim_elapsed: 0.0,
            anim_frame: 0,
            lifetime: 0.0,
            invuln_time: 0.0,
            fire_cooldown: 0.0,
            dive_timer: 0.0,
            phase: 0.0,
        }
    }

    pub fn world_hitbox(&self) -> Rect {
        self.hitbox.at(self.position)
    }

    pub fn center(&self) -> Vec2 {
        let hb = self.world_hitbox();
        Vec2::new(hb.x + hb.width / 2.0, hb.y + hb.height / 2.0)
    }

    #[allow(dead_code)]
    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
        if self.health <= 0 {
            self.active = false;
        }
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invuln_time > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_take_damage() {
        let mut e = Entity::new(
            EntityType::EnemyGrunt,
            Vec2::new(100.0, 50.0),
            Rect::new(1.0, 1.0, 12.0, 10.0),
            15,
        );
        assert!(e.active);
        e.take_damage(10);
        assert!(e.active);
        e.take_damage(5);
        assert!(!e.active);
    }

    #[test]
    fn test_entity_world_hitbox() {
        let e = Entity::new(
            EntityType::Player,
            Vec2::new(100.0, 400.0),
            Rect::new(2.0, 2.0, 12.0, 12.0),
            30,
        );
        let hb = e.world_hitbox();
        assert!((hb.x - 102.0).abs() < 1e-9);
        assert!((hb.y - 402.0).abs() < 1e-9);
    }

    #[test]
    fn test_active_powerup_expires() {
        let mut pu = ActivePowerUp::new(PowerUpType::RapidFire, 1.0);
        assert!(!pu.is_expired());
        pu.tick(0.5);
        assert!(!pu.is_expired());
        pu.tick(0.6);
        assert!(pu.is_expired());
    }

    #[test]
    fn test_ground_explosion_entity_type_distinct() {
        let e = Entity::new(
            EntityType::GroundExplosion,
            Vec2::new(200.0, 448.0),
            Rect::new(0.0, 0.0, 10.0, 10.0),
            1,
        );
        assert!(matches!(e.entity_type, EntityType::GroundExplosion));
        assert!(e.active);
    }

    #[test]
    fn test_extra_life_powerup_type_distinct() {
        let pu = PowerUpType::ExtraLife;
        assert_eq!(pu, PowerUpType::ExtraLife);
        // Ensure it does not match other types
        assert_ne!(pu, PowerUpType::Shield);
    }
}
