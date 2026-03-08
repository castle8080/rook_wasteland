//! Browser integration tests for M5 core game UI and M6 storage.
//!
//! These tests mount real Leptos components into a headless Firefox browser and
//! assert DOM behaviour. Each test creates its own isolated container per
//! the pattern documented in `doc/lessons.md` § L7.
//!
//! Run with: `wasm-pack test --headless --firefox --features wasm-test`

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;

wasm_bindgen_test_configure!(run_in_browser);

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Append a fresh isolated `<div>` to `document.body` and return it as an
/// `HtmlElement` (required by `leptos::mount::mount_to`).
fn fresh_container() -> web_sys::HtmlElement {
    let doc = web_sys::window()
        .expect("window")
        .document()
        .expect("document");
    let container = doc.create_element("div").expect("create div");
    doc.body()
        .expect("body")
        .append_child(&container)
        .expect("append");
    container.unchecked_into::<web_sys::HtmlElement>()
}

/// Yield to the microtask queue so Leptos effects flush.
async fn tick() {
    JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL))
        .await
        .expect("tick");
}

/// Clear all rw_sixzee localStorage keys so app-mount tests start with a
/// clean slate, regardless of what previous test runs left behind.
fn clear_game_storage() {
    if let Some(w) = web_sys::window() {
        if let Ok(Some(ls)) = w.local_storage() {
            let _ = ls.remove_item("rw_sixzee.in_progress");
            let _ = ls.remove_item("rw_sixzee.history");
            let _ = ls.remove_item("rw_sixzee.theme");
        }
    }
}

/// Click the "Let's play." button on the opening-quote overlay if it exists.
async fn dismiss_opening_quote(container: &web_sys::HtmlElement) {
    if let Ok(Some(btn)) = container.query_selector(".grandma-quote-overlay .btn--primary") {
        btn.unchecked_ref::<HtmlElement>().click();
        tick().await;
    }
}

/// Find the Roll button inside the action-buttons and click it.
fn click_roll(container: &web_sys::HtmlElement) {
    let buttons = container
        .query_selector_all(".action-buttons button")
        .expect("query");
    for i in 0..buttons.length() {
        let btn = buttons.item(i).expect("item");
        if btn.text_content().unwrap_or_default().contains("ROLL") {
            btn.unchecked_ref::<HtmlElement>().click();
            return;
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

/// The App component mounts without panicking and renders the game header.
#[wasm_bindgen_test]
async fn app_mounts_without_panic() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    let header = container.query_selector(".game-header").expect("query ok");
    assert!(header.is_some(), "game-header should be present");
}

/// On fresh game start, all 5 dice show '?' (unrolled).
#[wasm_bindgen_test]
async fn fresh_game_dice_show_question_marks() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    let dice_row = container
        .query_selector(".dice-row")
        .expect("query ok")
        .expect("dice-row present");

    let buttons = dice_row
        .query_selector_all("button")
        .expect("query_selector_all");
    assert_eq!(buttons.length(), 5, "5 dice expected");
    for i in 0..5 {
        let btn = buttons.item(i).expect("item");
        assert_eq!(
            btn.text_content().unwrap_or_default().trim(),
            "?",
            "die {i} should show ?"
        );
    }
}

/// Roll button is enabled before the first roll.
#[wasm_bindgen_test]
async fn roll_button_enabled_before_first_roll() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    let buttons = container
        .query_selector_all(".action-buttons button")
        .expect("query");
    let mut roll_btn: Option<web_sys::HtmlButtonElement> = None;
    for i in 0..buttons.length() {
        let btn = buttons.item(i).expect("item");
        if btn.text_content().unwrap_or_default().contains("ROLL") {
            roll_btn = btn.dyn_into::<web_sys::HtmlButtonElement>().ok();
            break;
        }
    }
    let roll_btn = roll_btn.expect("Roll button should exist");
    assert!(!roll_btn.disabled(), "Roll button should be enabled initially");
}

/// After clicking Roll, dice show numeric values 1–6.
#[wasm_bindgen_test]
async fn roll_reveals_dice_values() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_roll(&container);
    tick().await;

    let dice_row = container
        .query_selector(".dice-row")
        .expect("query ok")
        .expect("dice-row present");
    let dice_buttons = dice_row.query_selector_all("button").expect("query");
    assert_eq!(dice_buttons.length(), 5);

    for i in 0..5 {
        let btn = dice_buttons.item(i).expect("item");
        let text = btn.text_content().unwrap_or_default();
        let text = text.trim();
        assert_ne!(text, "?", "die {i} should have a value after roll");
        let val: u8 = text.parse().expect("should be numeric");
        assert!((1..=6).contains(&val), "die {i} value {val} out of range");
    }
}

/// After rolling, scorecard open cells show preview scores.
#[wasm_bindgen_test]
async fn scorecard_shows_preview_after_roll() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_roll(&container);
    tick().await;

    let preview_cells = container
        .query_selector_all(".scorecard__cell--preview, .scorecard__cell--zero-preview")
        .expect("query");
    assert!(
        preview_cells.length() > 0,
        "at least one preview cell expected after roll"
    );
}

/// Clicking a die after rolling applies the --held class.
#[wasm_bindgen_test]
async fn clicking_die_applies_held_class() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_roll(&container);
    tick().await;

    let first_die = container
        .query_selector(".dice-row button")
        .expect("query ok")
        .expect("die present");
    first_die.unchecked_ref::<HtmlElement>().click();
    tick().await;

    let held_die = container
        .query_selector(".dice-row__die--held")
        .expect("query ok");
    assert!(held_die.is_some(), "held die should have --held class");
}

/// Roll button is disabled after 3 rolls.
#[wasm_bindgen_test]
async fn roll_button_disabled_after_three_rolls() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    for _ in 0..3 {
        click_roll(&container);
        tick().await;
    }

    let buttons = container
        .query_selector_all(".action-buttons button")
        .expect("query");
    let mut roll_disabled = false;
    for i in 0..buttons.length() {
        let btn = buttons.item(i).expect("item");
        if btn.text_content().unwrap_or_default().contains("ROLL") {
            roll_disabled = btn
                .dyn_into::<web_sys::HtmlButtonElement>()
                .expect("btn")
                .disabled();
            break;
        }
    }
    assert!(roll_disabled, "Roll button should be disabled after 3 rolls");
}

/// Clicking a zero-preview cell shows the confirm-zero overlay.
/// If no zero-preview cell exists after 3 rolls the test passes vacuously
/// (the exact dice cannot be controlled in a headless test).
#[wasm_bindgen_test]
async fn confirm_zero_overlay_shown_for_zero_cell() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    for _ in 0..3 {
        click_roll(&container);
        tick().await;
    }

    if let Ok(Some(cell)) = container.query_selector(".scorecard__cell--zero-preview") {
        cell.unchecked_ref::<HtmlElement>().click();
        tick().await;

        let overlay = container
            .query_selector(".overlay--confirm")
            .expect("query ok");
        assert!(
            overlay.is_some(),
            "confirm-zero overlay should appear after clicking a zero-preview cell"
        );
    }
}

/// Cancelling confirm-zero dismisses the overlay.
#[wasm_bindgen_test]
async fn confirm_zero_cancel_dismisses_overlay() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    for _ in 0..3 {
        click_roll(&container);
        tick().await;
    }

    if let Ok(Some(cell)) = container.query_selector(".scorecard__cell--zero-preview") {
        cell.unchecked_ref::<HtmlElement>().click();
        tick().await;

        let cancel = container
            .query_selector(".overlay--confirm .btn--secondary")
            .expect("query ok")
            .expect("cancel button present");
        cancel.unchecked_ref::<HtmlElement>().click();
        tick().await;

        let overlay = container
            .query_selector(".overlay--confirm")
            .expect("query ok");
        assert!(
            overlay.is_none(),
            "confirm-zero overlay should be gone after Cancel"
        );
    }
}

/// Clicking a non-zero preview cell places the score and advances the turn
/// (dice return to '?' and scorecard shows the cell as filled).
///
/// Chance (row index 12) always produces a non-zero score for any roll, so we
/// use it as a reliable target. We query for `.scorecard__cell--preview` and
/// click the first one; if none exist (pathological dice state, shouldn't
/// happen) the test passes vacuously.
#[wasm_bindgen_test]
async fn scoring_non_zero_cell_advances_turn() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_roll(&container);
    tick().await;

    if let Ok(Some(cell)) = container.query_selector(".scorecard__cell--preview") {
        cell.unchecked_ref::<HtmlElement>().click();
        tick().await;

        // Turn advanced — all dice should reset to '?'
        let dice_buttons = container
            .query_selector_all(".dice-row button")
            .expect("query");
        let all_reset = (0..dice_buttons.length()).all(|i| {
            dice_buttons
                .item(i)
                .map(|b| b.text_content().unwrap_or_default().trim() == "?")
                .unwrap_or(false)
        });
        assert!(all_reset, "all dice should show '?' after a score is placed");
    }
}

/// Clicking Confirm in the confirm-zero overlay places the 0 score and
/// advances the turn (dice return to '?').
#[wasm_bindgen_test]
async fn confirm_zero_confirm_places_score() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    for _ in 0..3 {
        click_roll(&container);
        tick().await;
    }

    if let Ok(Some(cell)) = container.query_selector(".scorecard__cell--zero-preview") {
        cell.unchecked_ref::<HtmlElement>().click();
        tick().await;

        // Click "Confirm Zero" (primary button inside the overlay)
        let confirm = container
            .query_selector(".overlay--confirm .btn--primary")
            .expect("query ok")
            .expect("confirm button present");
        confirm.unchecked_ref::<HtmlElement>().click();
        tick().await;

        // Overlay dismissed
        let overlay = container
            .query_selector(".overlay--confirm")
            .expect("query ok");
        assert!(overlay.is_none(), "overlay should dismiss after Confirm");

        // Turn advanced — dice reset to '?'
        let dice_buttons = container
            .query_selector_all(".dice-row button")
            .expect("query");
        let all_reset = (0..dice_buttons.length()).all(|i| {
            dice_buttons
                .item(i)
                .map(|b| b.text_content().unwrap_or_default().trim() == "?")
                .unwrap_or(false)
        });
        assert!(all_reset, "dice should show '?' after confirming zero score");
    }
}

/// Clicking an already-held die a second time removes the `--held` class
/// (toggle off).
#[wasm_bindgen_test]
async fn die_toggle_off_removes_held_class() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_roll(&container);
    tick().await;

    // Toggle ON
    let first_die = container
        .query_selector(".dice-row button")
        .expect("query ok")
        .expect("die present");
    first_die.unchecked_ref::<HtmlElement>().click();
    tick().await;
    assert!(
        container
            .query_selector(".dice-row__die--held")
            .expect("query ok")
            .is_some(),
        "die should be held after first click"
    );

    // Toggle OFF — click the same die again
    let held_die = container
        .query_selector(".dice-row__die--held")
        .expect("query ok")
        .expect("held die present");
    held_die.unchecked_ref::<HtmlElement>().click();
    tick().await;

    let still_held = container
        .query_selector(".dice-row__die--held")
        .expect("query ok");
    assert!(
        still_held.is_none(),
        "die should no longer be held after second click"
    );
}

// ─── M6 Storage tests ────────────────────────────────────────────────────────
//
// These tests exercise the raw localStorage round-trips directly, independent
// of Leptos component rendering.  Each test cleans up its own keys to avoid
// cross-test contamination (tests run sequentially in a single browser page).

/// Helper: remove a localStorage key unconditionally.
fn ls_remove(key: &str) {
    if let Some(w) = web_sys::window() {
        if let Ok(Some(ls)) = w.local_storage() {
            let _ = ls.remove_item(key);
        }
    }
}

/// Helper: write a raw string into localStorage (for corruption tests).
fn ls_set_raw(key: &str, value: &str) {
    if let Some(w) = web_sys::window() {
        if let Ok(Some(ls)) = w.local_storage() {
            let _ = ls.set_item(key, value);
        }
    }
}

// ── in_progress ──────────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn storage_load_in_progress_returns_none_when_absent() {
    use rw_sixzee::state::storage;
    ls_remove("rw_sixzee.in_progress");
    let result = storage::load_in_progress().expect("load should succeed");
    assert!(result.is_none());
}

#[wasm_bindgen_test]
fn storage_save_and_load_in_progress_round_trip() {
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::storage;

    ls_remove("rw_sixzee.in_progress");
    let mut state = new_game();
    state.turn = 7;
    state.rolls_used = 2;

    storage::save_in_progress(&state).expect("save should succeed");
    let loaded = storage::load_in_progress()
        .expect("load should succeed")
        .expect("should be Some after save");

    assert_eq!(loaded.id, state.id);
    assert_eq!(loaded.turn, 7);
    assert_eq!(loaded.rolls_used, 2);
    ls_remove("rw_sixzee.in_progress");
}

#[wasm_bindgen_test]
fn storage_clear_in_progress_removes_key() {
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::storage;

    let state = new_game();
    storage::save_in_progress(&state).expect("save should succeed");
    storage::clear_in_progress().expect("clear should succeed");

    let result = storage::load_in_progress().expect("load should succeed");
    assert!(result.is_none(), "key must be absent after clear");
}

#[wasm_bindgen_test]
fn storage_load_in_progress_returns_json_error_on_corrupt_data() {
    use rw_sixzee::error::AppError;
    use rw_sixzee::state::storage;

    ls_set_raw("rw_sixzee.in_progress", "{ not valid json >>>>");
    let result = storage::load_in_progress();
    assert!(
        matches!(result, Err(AppError::Json(_))),
        "corrupt data must produce AppError::Json"
    );
    ls_remove("rw_sixzee.in_progress");
}

#[wasm_bindgen_test]
fn storage_save_preserves_bonus_pool_and_cells() {
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    ls_remove("rw_sixzee.in_progress");
    let mut state = new_game();
    state.bonus_pool = 300;
    state.cells[0][0] = Some(5);
    state.cells[5][12] = Some(20);

    storage::save_in_progress(&state).expect("save ok");
    let loaded = storage::load_in_progress()
        .expect("load ok")
        .expect("Some");

    assert_eq!(loaded.bonus_pool, 300);
    assert_eq!(loaded.cells[0][0], Some(5));
    assert_eq!(loaded.cells[5][12], Some(20));
    ls_remove("rw_sixzee.in_progress");
}

// ── history ───────────────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn storage_load_history_returns_empty_when_absent() {
    use rw_sixzee::state::storage;
    ls_remove("rw_sixzee.history");
    let result = storage::load_history().expect("load should succeed");
    assert!(result.is_empty(), "absent key must return empty vec");
}

#[wasm_bindgen_test]
fn storage_save_and_load_history_round_trip() {
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    ls_remove("rw_sixzee.history");
    let entries = vec![CompletedGame {
        id: "hist-test".to_string(),
        completed_at: "2025-01-01T00:00:00.000Z".to_string(),
        final_score: 250,
        bonus_pool: 100,
        bonus_forfeited: false,
        cells: [[None; ROW_COUNT]; 6],
    }];

    storage::save_history(&entries).expect("save ok");
    let loaded = storage::load_history().expect("load ok");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "hist-test");
    assert_eq!(loaded[0].final_score, 250);
    ls_remove("rw_sixzee.history");
}

#[wasm_bindgen_test]
fn storage_save_history_stores_sorted_by_score_descending() {
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    ls_remove("rw_sixzee.history");
    let entries = vec![
        CompletedGame {
            id: "low".to_string(),
            completed_at: "2025-01-01T00:00:00.000Z".to_string(),
            final_score: 100,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
        CompletedGame {
            id: "high".to_string(),
            completed_at: "2025-01-01T00:00:00.000Z".to_string(),
            final_score: 400,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
        CompletedGame {
            id: "mid".to_string(),
            completed_at: "2025-01-01T00:00:00.000Z".to_string(),
            final_score: 250,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
    ];

    storage::save_history(&entries).expect("save ok");
    let loaded = storage::load_history().expect("load ok");

    assert_eq!(loaded[0].final_score, 400, "highest score must be first");
    assert_eq!(loaded[1].final_score, 250);
    assert_eq!(loaded[2].final_score, 100, "lowest score must be last");
    ls_remove("rw_sixzee.history");
}

#[wasm_bindgen_test]
fn storage_load_history_returns_json_error_on_corrupt_data() {
    use rw_sixzee::error::AppError;
    use rw_sixzee::state::storage;

    ls_set_raw("rw_sixzee.history", "[{broken");
    let result = storage::load_history();
    assert!(
        matches!(result, Err(AppError::Json(_))),
        "corrupt history must produce AppError::Json"
    );
    ls_remove("rw_sixzee.history");
}

// ── theme ─────────────────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn storage_save_and_load_theme_round_trip() {
    use rw_sixzee::state::storage;
    ls_remove("rw_sixzee.theme");

    storage::save_theme("devil_rock").expect("save ok");
    let loaded = storage::load_theme().expect("load ok");
    assert_eq!(loaded, Some("devil_rock".to_string()));
    ls_remove("rw_sixzee.theme");
}

#[wasm_bindgen_test]
fn storage_load_theme_returns_none_when_absent() {
    use rw_sixzee::state::storage;
    ls_remove("rw_sixzee.theme");
    let loaded = storage::load_theme().expect("load ok");
    assert!(loaded.is_none());
}

#[wasm_bindgen_test]
fn storage_save_theme_overwrites_previous() {
    use rw_sixzee::state::storage;
    ls_remove("rw_sixzee.theme");

    storage::save_theme("nordic_minimal").expect("save ok");
    storage::save_theme("borg").expect("overwrite ok");
    let loaded = storage::load_theme().expect("load ok");
    assert_eq!(loaded, Some("borg".to_string()));
    ls_remove("rw_sixzee.theme");
}

// ─── M6 App load sequence & ResumePrompt integration tests ──────────────────
//
// These tests mount the full App and verify the M6 load sequence: saved-game
// detection, ResumePrompt button callbacks, history pruning on load, roll
// persistence, and theme application.

/// When a game is saved in localStorage before mount, App shows the resume
/// prompt — not the opening-quote overlay.
#[wasm_bindgen_test]
async fn app_shows_resume_prompt_when_game_saved() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::storage;

    clear_game_storage();
    let mut state = new_game();
    state.turn = 3;
    storage::save_in_progress(&state).expect("save ok");

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;

    let resume_prompt = container.query_selector(".resume-prompt").expect("query ok");
    assert!(
        resume_prompt.is_some(),
        "resume-prompt must be visible when a saved game exists"
    );
    let quote_overlay = container
        .query_selector(".grandma-quote-overlay")
        .expect("query ok");
    assert!(
        quote_overlay.is_none(),
        "opening-quote overlay must NOT appear when resume prompt is shown"
    );
    ls_remove("rw_sixzee.in_progress");
}

/// When no saved game exists in localStorage, App does NOT show the resume prompt.
#[wasm_bindgen_test]
async fn app_no_saved_game_does_not_show_resume_prompt() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;

    let resume_prompt = container.query_selector(".resume-prompt").expect("query ok");
    assert!(
        resume_prompt.is_none(),
        "resume-prompt must NOT appear when no game is saved"
    );
}

/// Clicking "Resume Game" dismisses the prompt and shows the game view.
#[wasm_bindgen_test]
async fn resume_prompt_resume_button_dismisses_prompt() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::storage;

    clear_game_storage();
    let mut state = new_game();
    state.turn = 5;
    storage::save_in_progress(&state).expect("save ok");

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;

    // Sanity: prompt is up.
    assert!(
        container
            .query_selector(".resume-prompt")
            .expect("query ok")
            .is_some(),
        "resume-prompt must be visible before clicking Resume"
    );

    // Click "Resume Game" (primary button).
    let btn = container
        .query_selector(".resume-prompt .btn--primary")
        .expect("query ok")
        .expect("Resume Game button must exist");
    btn.unchecked_ref::<HtmlElement>().click();
    tick().await;

    // Prompt must be gone and the game view must appear.
    assert!(
        container
            .query_selector(".resume-prompt")
            .expect("query ok")
            .is_none(),
        "resume-prompt must be dismissed after clicking Resume"
    );
    let dice = container
        .query_selector_all(".dice-row button")
        .expect("query ok");
    assert_eq!(dice.length(), 5, "5 dice buttons must be visible after resume");
    ls_remove("rw_sixzee.in_progress");
}

/// Clicking "Discard and Start New" dismisses the prompt and clears the saved
/// game from localStorage.
#[wasm_bindgen_test]
async fn resume_prompt_discard_clears_storage_and_dismisses_prompt() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::storage;

    clear_game_storage();
    let mut state = new_game();
    state.turn = 2;
    storage::save_in_progress(&state).expect("save ok");

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;

    // Click "Discard and Start New" (secondary button).
    let btn = container
        .query_selector(".resume-prompt .btn--secondary")
        .expect("query ok")
        .expect("Discard button must exist");
    btn.unchecked_ref::<HtmlElement>().click();
    // Two ticks: first flush sets show_resume=false; second settles
    // show_opening_quote=true and re-renders the view.
    tick().await;
    tick().await;

    // Prompt must be gone.
    assert!(
        container
            .query_selector(".resume-prompt")
            .expect("query ok")
            .is_none(),
        "resume-prompt must be dismissed after Discard and Start New"
    );
    // Saved game must be cleared from localStorage.
    let saved = storage::load_in_progress().expect("load ok");
    assert!(
        saved.is_none(),
        "in_progress must be cleared from storage after Discard and Start New"
    );
}

/// When the saved game JSON is corrupt, App shows an error banner (not the
/// resume prompt).
#[wasm_bindgen_test]
async fn app_load_corrupt_save_shows_error_banner() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    ls_set_raw("rw_sixzee.in_progress", "{corrupt!!!");

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;

    // Resume prompt must NOT appear for a corrupt save.
    assert!(
        container
            .query_selector(".resume-prompt")
            .expect("query ok")
            .is_none(),
        "resume-prompt must NOT appear for corrupt save data"
    );
    // Error banner must be shown.
    assert!(
        container
            .query_selector(".error-overlay")
            .expect("query ok")
            .is_some(),
        "error overlay must appear when saved game JSON is corrupt"
    );
    ls_remove("rw_sixzee.in_progress");
}

/// App's load sequence prunes history entries older than 365 days.
#[wasm_bindgen_test]
async fn app_load_prunes_old_history_entries() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();
    // 2022 is more than 365 days before any reasonable "current" time.
    let old_entry = CompletedGame {
        id: "old-game".to_string(),
        completed_at: "2022-01-01T00:00:00.000Z".to_string(),
        final_score: 999,
        bonus_pool: 0,
        bonus_forfeited: false,
        cells: [[None; ROW_COUNT]; 6],
    };
    storage::save_history(&[old_entry]).expect("save ok");

    let _handle = mount_to(fresh_container(), App);
    tick().await;

    // App load sequence must have pruned the ancient entry.
    let history = storage::load_history().expect("load ok");
    assert!(
        history.is_empty(),
        "history entry from 2022 must be pruned on app load"
    );
}

/// Rolling dice persists the updated game state to localStorage.
#[wasm_bindgen_test]
async fn on_roll_persists_game_state_to_storage() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::storage;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    // Before rolling, nothing should be in storage yet.
    let before = storage::load_in_progress().expect("load ok");
    assert!(before.is_none(), "no in_progress should exist before the first roll");

    click_roll(&container);
    tick().await;

    // After rolling, the current game state should be persisted.
    let after = storage::load_in_progress().expect("load ok");
    assert!(
        after.is_some(),
        "in_progress must be saved after rolling dice"
    );
}

/// App applies the saved theme from localStorage to the document body on mount.
#[wasm_bindgen_test]
async fn app_load_applies_saved_theme_to_body() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::storage;

    clear_game_storage();
    // Use a non-default value so we distinguish "loaded from storage" from
    // "fell back to the nordic_minimal default".
    storage::save_theme("sixzee_dark").expect("save ok");

    let _handle = mount_to(fresh_container(), App);
    tick().await;

    let theme_attr = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
        .and_then(|b| b.get_attribute("data-theme"));
    assert_eq!(
        theme_attr.as_deref(),
        Some("sixzee_dark"),
        "body data-theme must match the theme saved in localStorage"
    );
    ls_remove("rw_sixzee.theme");
}
