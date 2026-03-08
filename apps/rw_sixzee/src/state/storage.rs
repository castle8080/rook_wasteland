//! localStorage persistence for rw_sixzee.
//!
//! All public functions return `AppResult<_>`. A `SecurityError` from the
//! browser (e.g. private browsing mode where storage is blocked) is converted
//! to `AppError::Storage` — callers should report this as `Degraded` and
//! continue playing without persistence.
//!
//! Keys used:
//! - `rw_sixzee.in_progress` — serialised `GameState` for the current game
//! - `rw_sixzee.history`     — serialised `Vec<CompletedGame>`, sorted desc
//! - `rw_sixzee.theme`       — theme id string

use wasm_bindgen::JsValue;

use crate::error::{AppError, AppResult};
use crate::state::game::{sort_history_by_score, CompletedGame, GameState};

const KEY_IN_PROGRESS: &str = "rw_sixzee.in_progress";
const KEY_HISTORY: &str = "rw_sixzee.history";
const KEY_THEME: &str = "rw_sixzee.theme";

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn js_to_storage_err(v: JsValue) -> AppError {
    AppError::Storage(
        v.as_string()
            .unwrap_or_else(|| "JS storage error".to_string()),
    )
}

/// Obtain the `localStorage` object or return `AppError::Storage`.
///
/// Handles both the `SecurityError` thrown in private browsing mode (where
/// `window.localStorage` is blocked) and the `None` case where the property
/// returns `null`.
fn local_storage() -> AppResult<web_sys::Storage> {
    let window =
        web_sys::window().ok_or_else(|| AppError::Storage("no window object".to_string()))?;
    window
        .local_storage()
        .map_err(js_to_storage_err)?
        .ok_or_else(|| AppError::Storage("localStorage is unavailable".to_string()))
}

// ─── In-progress game ─────────────────────────────────────────────────────────

/// Load the in-progress game from localStorage.
///
/// Returns `Ok(None)` when the key is absent (fresh session).
/// Returns `Err(AppError::Json(_))` if the stored value is not valid JSON
/// or does not deserialise to `GameState` — treat this as a fatal corrupt save.
pub fn load_in_progress() -> AppResult<Option<GameState>> {
    let storage = local_storage()?;
    match storage
        .get_item(KEY_IN_PROGRESS)
        .map_err(js_to_storage_err)?
    {
        None => Ok(None),
        Some(json) => {
            let state: GameState = serde_json::from_str(&json)?;
            Ok(Some(state))
        }
    }
}

/// Serialise `state` and write it to localStorage under `rw_sixzee.in_progress`.
pub fn save_in_progress(state: &GameState) -> AppResult<()> {
    let storage = local_storage()?;
    let json = serde_json::to_string(state)?;
    storage
        .set_item(KEY_IN_PROGRESS, &json)
        .map_err(js_to_storage_err)?;
    Ok(())
}

/// Remove the `rw_sixzee.in_progress` key from localStorage.
pub fn clear_in_progress() -> AppResult<()> {
    let storage = local_storage()?;
    storage
        .remove_item(KEY_IN_PROGRESS)
        .map_err(js_to_storage_err)?;
    Ok(())
}

// ─── History ──────────────────────────────────────────────────────────────────

/// Load the completed-game history list from localStorage.
///
/// Returns `Ok(vec![])` when the key is absent.
pub fn load_history() -> AppResult<Vec<CompletedGame>> {
    let storage = local_storage()?;
    match storage.get_item(KEY_HISTORY).map_err(js_to_storage_err)? {
        None => Ok(vec![]),
        Some(json) => {
            let history: Vec<CompletedGame> = serde_json::from_str(&json)?;
            Ok(history)
        }
    }
}

/// Sort `history` by `final_score` descending and write it to localStorage.
///
/// The stored list is always kept sorted so readers need no re-sort on load.
pub fn save_history(history: &[CompletedGame]) -> AppResult<()> {
    let storage = local_storage()?;
    let mut sorted = history.to_vec();
    sort_history_by_score(&mut sorted);
    let json = serde_json::to_string(&sorted)?;
    storage
        .set_item(KEY_HISTORY, &json)
        .map_err(js_to_storage_err)?;
    Ok(())
}

// ─── Theme ────────────────────────────────────────────────────────────────────

/// Load the stored theme id string, or `None` if absent.
pub fn load_theme() -> AppResult<Option<String>> {
    let storage = local_storage()?;
    storage.get_item(KEY_THEME).map_err(js_to_storage_err)
}

/// Persist a theme id string to localStorage.
pub fn save_theme(theme: &str) -> AppResult<()> {
    let storage = local_storage()?;
    storage
        .set_item(KEY_THEME, theme)
        .map_err(js_to_storage_err)?;
    Ok(())
}
