use super::constants::*;
use super::{powerup_index, Game};
use crate::entities::EntityType;
use crate::graphics::{RetroColors, SpriteGenerator};
use crate::renderer::Renderer;

impl Game {
    pub(super) fn render_game(&self, renderer: &Renderer) {
        // Enemies and world entities
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
                EntityType::GroundExplosion => {
                    // Fade out as lifetime decreases. lifetime goes from GROUND_EXPLOSION_DURATION → 0.
                    let alpha = (e.lifetime / GROUND_EXPLOSION_DURATION).clamp(0.0, 1.0);
                    // center.y == CANVAS_H (screen bottom); center.x == horizontal blast center.
                    let cx = e.center().x;
                    let r = GROUND_EXPLOSION_RADIUS;
                    renderer.set_alpha(alpha * 0.85);
                    // Outer arc rising from the ground — orange
                    renderer.fill_rect(cx - r, CANVAS_H - r * 0.5, r * 2.0, r * 0.5, RetroColors::ORANGE);
                    // Inner core — bright yellow
                    renderer.fill_rect(cx - r * 0.6, CANVAS_H - r * 0.35, r * 1.2, r * 0.35, RetroColors::YELLOW);
                    // Hot white core
                    renderer.fill_rect(cx - r * 0.25, CANVAS_H - r * 0.2, r * 0.5, r * 0.2, RetroColors::WHITE);
                    renderer.reset_alpha();
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
        // Lives (draw small ship icons; cap at 9 so they fit the HUD)
        for i in 0..self.player.lives.min(9) {
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

    pub(super) fn render_main_menu(&self, renderer: &Renderer) {
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

    pub(super) fn render_wave_transition(&self, renderer: &Renderer, timer: f64) {
        let alpha = ((WAVE_TRANSITION_DURATION - timer) / 0.3).min(1.0);
        renderer.set_alpha(alpha);
        renderer.draw_text_centered(
            &format!("WAVE {}", self.wave + 1),
            CANVAS_H / 2.0 - 16.0, RetroColors::CYAN, 28,
        );
        renderer.reset_alpha();
    }

    pub(super) fn render_pause_overlay(&self, renderer: &Renderer) {
        renderer.set_alpha(0.5);
        renderer.fill_rect(0.0, 0.0, CANVAS_W, CANVAS_H, RetroColors::BLACK);
        renderer.reset_alpha();
        renderer.draw_text_centered("PAUSED", CANVAS_H / 2.0 - 20.0, RetroColors::YELLOW, 24);
        renderer.draw_text_centered("PRESS ESC TO RESUME", CANVAS_H / 2.0 + 10.0, RetroColors::WHITE, 12);
    }

    pub(super) fn render_game_over(&self, renderer: &Renderer, timer: f64) {
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
            renderer.draw_text_centered(
                "PRESS ENTER TO PLAY AGAIN",
                CANVAS_H / 2.0 + 60.0, RetroColors::GREEN, 10,
            );
        }
        renderer.reset_alpha();
    }
}
