use super::constants::*;
use super::constants::save_high_score;
use super::{is_enemy, powerup_duration, Game, GameState};
use crate::entities::{ActivePowerUp, Entity, EntityType, PowerUpType};
use crate::systems::InputState;
use crate::utils::{Rect, Vec2};
use rand::Rng;

impl Game {
    pub(super) fn update_playing(&mut self, input: &InputState, dt: f64) {
        self.update_player(input, dt);
        self.update_spawning(dt);
        self.update_enemies(dt);
        self.update_bullets(dt);
        self.update_explosions(dt);
        self.update_ground_explosions(dt);
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
        p.entity.position.x = p.entity.position.x.clamp(0.0, CANVAS_W - 32.0);

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
            let px = p.entity.position.x + 14.0; // center of 32px-wide (2× scale) sprite
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

            // Tag piercing/explosive in sprite_index (placeholder until weapons module)
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
                // Hitbox for 20×16 sprite rendered at scale=2 (40×32 visual)
                (x, y, ENEMY_GRUNT_HP, Rect::new(4.0, 2.0, 32.0, 26.0))
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
        let shift_x = self.formation_dir * 20.0 * dt;

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
        let shift_x = self.formation_dir * 20.0 * dt; // recalc after possible flip

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
        // Ground explosions triggered this frame (enemy reached bottom).
        let mut ground_explosion_positions: Vec<Vec2> = Vec::new();

        for e in self.entities.iter_mut() {
            if !e.active || !is_enemy(&e.entity_type) {
                continue;
            }

            // ── Boss movement ─────────────────────────────────────────────
            if matches!(e.entity_type, EntityType::Boss) {
                e.phase += dt;
                let center_x = CANVAS_W / 2.0 - 24.0;
                e.position.x = center_x + BOSS_AMPLITUDE * (e.phase * BOSS_FREQ * std::f64::consts::TAU).sin();
                e.position.y = BOSS_Y;

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
                    ground_explosion_positions.push(e.position);
                    e.active = false;
                }
                continue;
            }

            // ── Normal enemy (Grunt / Weaver) ─────────────────────────────
            e.position.y += descend;
            e.position.x += shift_x;
            e.phase += dt;

            if wave >= 4 {
                let osc = (e.phase * std::f64::consts::PI / 1.5).sin();
                e.position.x += osc * 60.0 * dt * 0.333;
            }

            // Weaver zigzag
            if matches!(e.entity_type, EntityType::EnemyWeaver) {
                let zz = (e.phase * std::f64::consts::PI * 2.0 * 1.5).sin();
                e.position.x += zz * 40.0 * dt;
            }

            e.position.x = e.position.x.clamp(0.0, CANVAS_W - 20.0);

            // Fire cooldown and shooting (wave 3+)
            if wave >= 3 {
                e.fire_cooldown -= dt;
                if should_fire && e.fire_cooldown <= 0.0 {
                    let fire_chance = 0.03 + wave as f64 * 0.004;
                    let pseudo_roll = (e.phase * 137.508).fract().abs();
                    if pseudo_roll < fire_chance {
                        let aimed_threshold = (0.15 + (wave.saturating_sub(3)) as f64 * 0.025).min(0.5);
                        let aimed = (e.phase * 31.0).fract() < aimed_threshold;
                        fire_positions.push((e.position.x + 20.0, e.position.y + 30.0, aimed, false));
                        e.fire_cooldown = 2.0 / (1.0 + wave as f64 * 0.1);
                    }
                }
            }

            if e.position.y > CANVAS_H - 32.0 {
                ground_explosion_positions.push(e.position);
                e.active = false;
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

        // Spawn ground explosions for enemies that reached the bottom this frame.
        for pos in ground_explosion_positions {
            self.spawn_ground_explosion_at(pos);
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

    fn update_ground_explosions(&mut self, dt: f64) {
        for e in self.entities.iter_mut() {
            if !e.active || !matches!(e.entity_type, EntityType::GroundExplosion) {
                continue;
            }
            e.lifetime -= dt;
            if e.lifetime <= 0.0 {
                e.active = false;
            }
        }
    }

    fn spawn_ground_explosion_at(&mut self, pos: Vec2) {
        // Center horizontally on the enemy, pin vertically to the screen bottom.
        let center_x = pos.x + 20.0; // approximate enemy center (sprite is 40px wide at 2×)
        // Position the bounding-box top-left so that entity.center() lands at
        // (center_x, CANVAS_H): center.x = pos.x + radius, center.y = pos.y + radius.
        let blast_pos = Vec2::new(
            center_x - GROUND_EXPLOSION_RADIUS,
            CANVAS_H - GROUND_EXPLOSION_RADIUS, // → center.y == CANVAS_H (screen bottom)
        );
        let mut e = Entity::new(
            EntityType::GroundExplosion,
            blast_pos,
            Rect::new(0.0, 0.0, GROUND_EXPLOSION_RADIUS * 2.0, GROUND_EXPLOSION_RADIUS * 2.0),
            1,
        );
        e.lifetime = GROUND_EXPLOSION_DURATION;
        self.entities.push(e);
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

    // ── Collision detection ───────────────────────────────────────────────────

    fn check_collisions(&mut self) {
        if !self.player.is_alive() {
            return;
        }

        let player_hb = self.player.entity.world_hitbox();
        let player_invuln = self.player.entity.is_invulnerable();
        let player_center = self.player.entity.center();

        let mut to_kill: Vec<usize> = Vec::new();
        let mut score_gain: u32 = 0;
        let mut spawn_explosions: Vec<Vec2> = Vec::new();
        let mut spawn_powerups: Vec<Vec2> = Vec::new();
        let mut player_hit = false;
        let mut collected_powerup: Option<(PowerUpType, f64)> = None;
        // Centers of active ground explosions — collected first, applied after the main loop.
        let mut ground_explosion_centers: Vec<Vec2> = Vec::new();

        for (i, e) in self.entities.iter().enumerate() {
            if !e.active { continue; }

            match &e.entity_type {
                EntityType::PlayerBullet => {
                    let bhb = e.world_hitbox();
                    for (j, target) in self.entities.iter().enumerate() {
                        if !target.active || i == j { continue; }
                        if !is_enemy(&target.entity_type) { continue; }
                        if bhb.intersects(&target.world_hitbox()) {
                            to_kill.push(i); // bullet
                            to_kill.push(j); // enemy
                            score_gain += enemy_score_value(&target.entity_type, self.wave);
                            spawn_explosions.push(target.position);
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
                    let dist = e.center().distance_to(player_center);
                    if dist <= collect_radius {
                        to_kill.push(i);
                        score_gain += 25;
                        let duration = powerup_duration(ptype);
                        collected_powerup = Some((ptype.clone(), duration));
                    }
                }
                EntityType::GroundExplosion => {
                    ground_explosion_centers.push(e.center());
                }
                _ => {}
            }
        }

        // Ground explosion — check player and enemies in radius.
        // No score awarded for collateral enemy kills (the explosion is a penalty, not a reward).
        for center in &ground_explosion_centers {
            if !player_invuln
                && player_center.distance_to(*center) <= GROUND_EXPLOSION_RADIUS
            {
                player_hit = true;
            }
            for (j, target) in self.entities.iter().enumerate() {
                if !target.active { continue; }
                if !is_enemy(&target.entity_type) { continue; }
                if target.center().distance_to(*center) <= GROUND_EXPLOSION_RADIUS {
                    to_kill.push(j);
                }
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
            match ptype {
                PowerUpType::Shield => {
                    self.player.shields = (self.player.shields + 1).min(3);
                }
                PowerUpType::ExtraLife => {
                    self.player.lives = (self.player.lives + 1).min(9);
                }
                _ => {
                    self.player.active_powerup = Some(ActivePowerUp::new(ptype, duration));
                }
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
        // Weights: 6 weapon/shield slots at ~13.7% each; ExtraLife at ~4% (narrower top band).
        let ptype = if roll < 0.04 {
            PowerUpType::ExtraLife
        } else {
            match ((roll - 0.04) / 0.96 * 6.0) as u32 {
                0 => PowerUpType::TripleShot,
                1 => PowerUpType::ExplosiveShot,
                2 => PowerUpType::RapidFire,
                3 => PowerUpType::LaserBeam,
                4 => PowerUpType::PiercingShot,
                _ => PowerUpType::Shield,
            }
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
}

// ── Private helpers ───────────────────────────────────────────────────────────

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
