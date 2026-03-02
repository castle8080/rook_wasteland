// Canvas dimensions are pub so lib.rs can re-export them.
pub const CANVAS_W: f64 = 640.0;
pub const CANVAS_H: f64 = 480.0;

pub const PLAYER_SPEED: f64 = 250.0;
pub const PLAYER_START_X: f64 = CANVAS_W / 2.0 - 16.0; // center of 32px-wide (2× scale) ship
pub const PLAYER_START_Y: f64 = CANVAS_H - 48.0;        // 16px gap from bottom for 32px-tall ship
pub const PLAYER_FIRE_COOLDOWN: f64 = 0.25; // 250ms
pub const PLAYER_MAX_BULLETS: usize = 3;

pub const BULLET_SPEED: f64 = 450.0;
pub const BULLET_LIFETIME: f64 = 1.07; // ~480px at 450px/s
pub const INVULN_DURATION: f64 = 2.5;
pub const INVULN_FLASH_HZ: f64 = 8.0;

pub const ENEMY_GRUNT_HP: i32 = 15;
pub const GRUNT_SPEED: f64 = 30.0;         // px/s descent
pub const FORMATION_COLS: u32 = 7;         // columns per row
#[allow(dead_code)] // documents the grid height; not read at runtime
pub const FORMATION_ROWS: u32 = 3;
pub const FORMATION_SPACING_X: f64 = 62.0;
pub const FORMATION_SPACING_Y: f64 = 50.0; // row gap for 32px-visual sprites
pub const FORMATION_START_Y: f64 = 30.0;

pub const ENEMY_BULLET_SPEED: f64 = 160.0; // px/s
pub const ENEMY_FIRE_INTERVAL: f64 = 0.5;  // check every 500ms
pub const WAVE_TRANSITION_DURATION: f64 = 2.0;

pub const BOSS_AMPLITUDE: f64 = 100.0;
pub const BOSS_FREQ: f64 = 0.5;   // Hz
pub const BOSS_Y: f64 = 50.0;     // fixed y in top third
pub const BOSS_FIRE_INTERVAL: f64 = 1.5;
pub const BOSS_BULLET_SPEED: f64 = 250.0;

pub const DIVER_SPEED: f64 = 80.0;
pub const DIVER_TRIGGER_RANGE: f64 = 80.0; // ±80px horizontal
pub const DIVER_RETURN_TIME: f64 = 2.0;
pub const DIVER_DIVE_COOLDOWN: f64 = 3.0;

/// Radius of the ground explosion when an enemy reaches the bottom (30% of canvas width).
pub const GROUND_EXPLOSION_RADIUS: f64 = 192.0;
/// How long the ground explosion blast window + visual lasts.
pub const GROUND_EXPLOSION_DURATION: f64 = 0.8;

const HIGH_SCORE_KEY: &str = "rw_defender_high_score";

pub(crate) fn load_high_score() -> u32 {
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

pub(crate) fn save_high_score(score: u32) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.set_item(HIGH_SCORE_KEY, &score.to_string());
    }
}
