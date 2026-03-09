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

/// After clicking Roll, dice show SVG faces (aria-label carries the value).
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

    // M8: dice show SVG faces — the die value is in the aria-label attribute
    // ("Die N value V [held]").  Verify every die has a value 1–6.
    for i in 0..5 {
        let btn = dice_buttons.item(i).expect("item");
        // Cast Node → Element to access get_attribute.
        let aria = btn
            .unchecked_ref::<web_sys::Element>()
            .get_attribute("aria-label")
            .unwrap_or_default();
        // After a roll, aria-label should NOT contain "?"
        assert!(
            !aria.contains('?'),
            "die {i} aria-label should not contain '?' after roll, got: {aria:?}"
        );
        // Extract the value token from "Die N value V [held]"
        let parts: Vec<&str> = aria.split_whitespace().collect();
        // parts: ["Die", "N", "value", "V", ...]
        let val_str = parts.get(3).copied().unwrap_or("");
        let val: u8 = val_str
            .parse()
            .unwrap_or_else(|_| panic!("die {i} value token {val_str:?} not numeric"));
        assert!(
            (1..=6).contains(&val),
            "die {i} value {val} out of range"
        );
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
    // Use a non-default theme so we can distinguish "loaded from storage" from
    // "fell back to the nordic_minimal default".
    storage::save_theme("devil_rock").expect("save ok");

    let _handle = mount_to(fresh_container(), App);
    tick().await;

    let theme_attr = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
        .and_then(|b| b.get_attribute("data-theme"));
    assert_eq!(
        theme_attr.as_deref(),
        Some("devil_rock"),
        "body data-theme must match the theme saved in localStorage"
    );
    ls_remove("rw_sixzee.theme");
}

// ─── M8 Theme integration tests ──────────────────────────────────────────────

/// Settings screen renders 6 theme cards.
#[wasm_bindgen_test]
async fn settings_renders_six_theme_cards() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    // Navigate to the settings route before mounting so App renders SettingsView.
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/settings");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let cards = container
        .query_selector_all(".settings__theme-card")
        .expect("query ok");
    assert_eq!(cards.length(), 6, "settings grid must have exactly 6 theme cards");

    // Restore hash so subsequent tests start at the default route.
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// Clicking a theme card updates data-theme on document.body.
#[wasm_bindgen_test]
async fn settings_card_click_changes_body_theme() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/settings");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Find the Borg card and click it.
    let borg_card = container
        .query_selector(".settings__theme-card[data-theme='borg']")
        .expect("query ok")
        .expect("borg card must exist");
    borg_card.unchecked_ref::<HtmlElement>().click();
    tick().await;

    let theme_attr = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
        .and_then(|b| b.get_attribute("data-theme"));
    assert_eq!(
        theme_attr.as_deref(),
        Some("borg"),
        "body data-theme must update to 'borg' after clicking the Borg card"
    );

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
    ls_remove("rw_sixzee.theme");
}

/// The active theme card has the --active modifier class.
#[wasm_bindgen_test]
async fn settings_active_card_has_active_class() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::storage;

    clear_game_storage();
    storage::save_theme("renaissance").expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/settings");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let active_card = container
        .query_selector(".settings__theme-card--active")
        .expect("query ok")
        .expect("active card must exist");
    let data_theme = active_card
        .get_attribute("data-theme")
        .unwrap_or_default();
    assert_eq!(
        data_theme, "renaissance",
        "the renaissance card must be marked active when that theme is loaded"
    );

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
    ls_remove("rw_sixzee.theme");
}

/// After switching themes, each card's aria-label reactively reflects active state.
///
/// Regression test for bug_002: aria-label was a static String (signal read outside
/// reactive context), so it never updated after the initial render.
#[wasm_bindgen_test]
async fn settings_card_aria_label_updates_on_theme_switch() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage(); // start fresh → default theme is nordic_minimal
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/settings");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Default active theme is Nordic Minimal — its aria-label must say "(active)".
    let nordic_card = container
        .query_selector(".settings__theme-card[data-theme='nordic_minimal']")
        .expect("query ok")
        .expect("nordic card must exist");
    let initial_label = nordic_card
        .get_attribute("aria-label")
        .unwrap_or_default();
    assert!(
        initial_label.contains("(active)"),
        "nordic_minimal aria-label must contain '(active)' initially; got: {initial_label:?}"
    );

    // Click the Borg card to switch the active theme.
    let borg_card = container
        .query_selector(".settings__theme-card[data-theme='borg']")
        .expect("query ok")
        .expect("borg card must exist");
    borg_card.unchecked_ref::<HtmlElement>().click();
    tick().await;

    // Nordic Minimal is no longer active — its aria-label must update reactively.
    let nordic_label_after = nordic_card
        .get_attribute("aria-label")
        .unwrap_or_default();
    assert!(
        !nordic_label_after.contains("(active)"),
        "nordic_minimal aria-label must NOT contain '(active)' after switching; got: {nordic_label_after:?}"
    );

    // Borg is now active — its aria-label must also update reactively.
    let borg_label_after = borg_card
        .get_attribute("aria-label")
        .unwrap_or_default();
    assert!(
        borg_label_after.contains("(active)"),
        "borg aria-label must contain '(active)' after becoming active; got: {borg_label_after:?}"
    );

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
    ls_remove("rw_sixzee.theme");
}

// ─── Bug 003 — GrandmaQuoteInline rendering ───────────────────────────────────

/// `GrandmaQuoteInline` must render the actual quote text, not the literal
/// string `{quote}`.
///
/// Regression for bug_003: the `{quote}` Leptos dynamic-expression block was
/// accidentally enclosed inside a Rust string literal, so the macro never
/// evaluated the variable and the rendered DOM contained the literal characters
/// `{`, `q`, `u`, `o`, `t`, `e`, `}`.
#[wasm_bindgen_test]
async fn grandma_quote_inline_renders_quote_text() {
    use leptos::mount::mount_to;
    use leptos::prelude::*;
    use rw_sixzee::components::grandma_quote::GrandmaQuoteInline;

    const TEST_QUOTE: &str = "Patience is not waiting. It is knowing.";

    let container = fresh_container();
    let _handle = mount_to(container.clone(), || {
        view! { <GrandmaQuoteInline quote=TEST_QUOTE.to_string() /> }
    });
    tick().await;

    let text_span = container
        .query_selector(".grandma-quote__text")
        .expect("query ok")
        .expect(".grandma-quote__text must be present when quote is non-empty");

    let text = text_span.text_content().unwrap_or_default();

    assert!(
        text.contains(TEST_QUOTE),
        "quote text must appear verbatim in .grandma-quote__text; got: {text:?}"
    );
    assert!(
        !text.contains("{quote}"),
        "literal '{{quote}}' must NOT appear in rendered output; got: {text:?}"
    );
}

// ─── Feature 001: Quit Game ───────────────────────────────────────────────────

/// Helper: click the game-menu ⋮ button.
fn click_game_menu(container: &web_sys::HtmlElement) {
    if let Ok(Some(btn)) = container.query_selector(".game-menu__btn") {
        btn.unchecked_ref::<HtmlElement>().click();
    }
}

/// Helper: click "Quit Game" inside the open game-menu panel.
fn click_quit_game_in_panel(container: &web_sys::HtmlElement) {
    if let Ok(Some(btn)) = container.query_selector(".game-menu__item--danger") {
        btn.unchecked_ref::<HtmlElement>().click();
    }
}

/// The game header contains the ⋮ game menu button (aria-label="Game menu")
/// while an active game is in progress.
#[wasm_bindgen_test]
async fn game_menu_button_present_during_active_game() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    let btn = container
        .query_selector(".game-menu__btn[aria-label='Game menu']")
        .expect("query ok");
    assert!(btn.is_some(), "game-menu button should be in the header");
}

/// Clicking the ⋮ button opens the dropdown panel containing the "Quit Game"
/// option.
#[wasm_bindgen_test]
async fn game_menu_panel_shows_on_click() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_game_menu(&container);
    tick().await;

    let panel = container
        .query_selector(".game-menu__panel")
        .expect("query ok");
    assert!(panel.is_some(), "game-menu panel should appear after clicking ⋮");

    let quit_item = container
        .query_selector(".game-menu__item--danger")
        .expect("query ok");
    assert!(quit_item.is_some(), "Quit Game item should be in the panel");
}

/// Tapping "Quit Game" in the panel shows the quit confirmation overlay.
#[wasm_bindgen_test]
async fn confirm_quit_overlay_shown_on_quit_tap() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_game_menu(&container);
    tick().await;
    click_quit_game_in_panel(&container);
    tick().await;

    let overlay = container
        .query_selector(".overlay--quit")
        .expect("query ok");
    assert!(overlay.is_some(), "quit confirmation overlay should be visible");
}

/// Tapping "Keep Playing" on the quit confirmation overlay dismisses it and
/// leaves the game header intact.
#[wasm_bindgen_test]
async fn cancel_quit_returns_to_active_game() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_game_menu(&container);
    tick().await;
    click_quit_game_in_panel(&container);
    tick().await;

    // Click "Keep Playing" (secondary button in overlay)
    if let Ok(Some(keep)) = container.query_selector(".overlay--quit .btn--secondary") {
        keep.unchecked_ref::<HtmlElement>().click();
        tick().await;
    }

    let overlay = container
        .query_selector(".overlay--quit")
        .expect("query ok");
    assert!(overlay.is_none(), "quit overlay should dismiss after Keep Playing");

    let header = container
        .query_selector(".game-header")
        .expect("query ok");
    assert!(header.is_some(), "game header should still be present after cancel");
}

/// Confirming quit hides the game content and shows the idle screen.
#[wasm_bindgen_test]
async fn confirm_quit_shows_idle_screen() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    click_game_menu(&container);
    tick().await;
    click_quit_game_in_panel(&container);
    tick().await;

    // Click the destructive "Quit" button
    if let Ok(Some(quit_btn)) = container.query_selector(".overlay--quit .btn--danger") {
        quit_btn.unchecked_ref::<HtmlElement>().click();
        tick().await;
    }

    let idle = container
        .query_selector(".idle-screen")
        .expect("query ok");
    assert!(idle.is_some(), "idle screen should appear after confirming quit");

    let header = container
        .query_selector(".game-header")
        .expect("query ok");
    assert!(header.is_none(), "game header should NOT be present on idle screen");

    let menu_btn = container
        .query_selector(".game-menu__btn")
        .expect("query ok");
    assert!(menu_btn.is_none(), "game menu button should NOT be present on idle screen");
}

/// The idle screen "Start New Game" button transitions back to an active game
/// (game header reappears).
#[wasm_bindgen_test]
async fn start_new_game_from_idle_returns_to_game() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    dismiss_opening_quote(&container).await;

    // Quit the game
    click_game_menu(&container);
    tick().await;
    click_quit_game_in_panel(&container);
    tick().await;
    if let Ok(Some(quit_btn)) = container.query_selector(".overlay--quit .btn--danger") {
        quit_btn.unchecked_ref::<HtmlElement>().click();
        tick().await;
    }

    // Tap "Start New Game"
    if let Ok(Some(start_btn)) = container.query_selector(".idle-screen__start-btn") {
        start_btn.unchecked_ref::<HtmlElement>().click();
        tick().await;
    }

    // Dismiss the opening quote if the bank was available
    dismiss_opening_quote(&container).await;

    let header = container
        .query_selector(".game-header")
        .expect("query ok");
    assert!(
        header.is_some(),
        "game header should be present after starting a new game from the idle screen"
    );

    let idle = container
        .query_selector(".idle-screen")
        .expect("query ok");
    assert!(idle.is_none(), "idle screen should be gone after starting a new game");
}

// ─── M9 History screen integration tests ─────────────────────────────────────

/// History view shows empty-state message when no games are in storage.
#[wasm_bindgen_test]
async fn history_view_shows_empty_state_when_no_history() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let empty_msg = container
        .query_selector(".history-list__empty")
        .expect("query ok");
    assert!(
        empty_msg.is_some(),
        "empty-state message must be visible when no history exists"
    );

    // Restore hash.
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History view renders one row per completed game, sorted by score descending.
#[wasm_bindgen_test]
async fn history_view_renders_rows_for_completed_games() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let entries = vec![
        CompletedGame {
            id: "game-a".to_string(),
            completed_at: "2026-03-01T10:00:00Z".to_string(),
            final_score: 350,
            bonus_pool: 100,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
        CompletedGame {
            id: "game-b".to_string(),
            completed_at: "2026-03-02T10:00:00Z".to_string(),
            final_score: 200,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
    ];
    storage::save_history(&entries).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // No empty-state message.
    let empty_msg = container
        .query_selector(".history-list__empty")
        .expect("query ok");
    assert!(
        empty_msg.is_none(),
        "empty-state message must NOT appear when history exists"
    );

    // Exactly 2 rows.
    let rows = container
        .query_selector_all(".history-list__row")
        .expect("query ok");
    assert_eq!(rows.length(), 2, "must render one row per completed game");

    // First row is the gold medal row (highest score).
    let gold_row = container
        .query_selector(".history-list__row--gold")
        .expect("query ok");
    assert!(gold_row.is_some(), "top-ranked row must have --gold class");

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History detail shows "Game not found" when the id does not exist in storage.
#[wasm_bindgen_test]
async fn history_detail_shows_not_found_for_unknown_id() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history/no-such-game-id-xyz");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let not_found = container
        .query_selector(".history-detail__not-found")
        .expect("query ok");
    assert!(
        not_found.is_some(),
        "not-found section must appear for an unknown game id"
    );

    // Scorecard must NOT be present.
    let scorecard = container
        .query_selector(".scorecard")
        .expect("query ok");
    assert!(
        scorecard.is_none(),
        "scorecard must NOT render when game is not found"
    );

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History detail renders the scorecard for a known game id.
#[wasm_bindgen_test]
async fn history_detail_renders_scorecard_for_known_id() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    // Build a game with all cells in column 0 filled.
    let mut cells = [[None; ROW_COUNT]; 6];
    cells[0].iter_mut().enumerate().for_each(|(row, cell)| {
        *cell = Some((row as u8) + 1);
    });
    let game = CompletedGame {
        id: "detail-test-game".to_string(),
        completed_at: "2026-03-07T12:00:00Z".to_string(),
        final_score: 300,
        bonus_pool: 100,
        bonus_forfeited: false,
        cells,
    };
    storage::save_history(&[game]).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history/detail-test-game");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Scorecard wrapper must be present.
    let scorecard = container
        .query_selector(".scorecard")
        .expect("query ok");
    assert!(
        scorecard.is_some(),
        "scorecard must render for a known game id"
    );

    // Not-found section must NOT appear.
    let not_found = container
        .query_selector(".history-detail__not-found")
        .expect("query ok");
    assert!(
        not_found.is_none(),
        "not-found section must NOT appear when game is found"
    );

    // At least one filled cell must be present.
    let filled_cells = container
        .query_selector_all(".scorecard__cell--filled")
        .expect("query ok");
    assert!(
        filled_cells.length() > 0,
        "filled scorecard cells must be visible"
    );

    // Header meta (date, score) must be present.
    let meta = container
        .query_selector(".history-detail__meta")
        .expect("query ok");
    assert!(
        meta.is_some(),
        "history-detail__meta must be rendered for a known game"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// The back button on History Detail navigates to the History list.
#[wasm_bindgen_test]
async fn history_detail_back_button_navigates_to_history_list() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let game = CompletedGame {
        id: "back-btn-test".to_string(),
        completed_at: "2026-03-07T12:00:00Z".to_string(),
        final_score: 250,
        bonus_pool: 0,
        bonus_forfeited: false,
        cells: [[None; ROW_COUNT]; 6],
    };
    storage::save_history(&[game]).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history/back-btn-test");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Click the back button.
    let back_btn = container
        .query_selector(".history-detail__back-btn")
        .expect("query ok")
        .expect("back button must exist on detail view");
    back_btn.unchecked_ref::<HtmlElement>().click();
    tick().await;
    tick().await;

    // Must now show the history list.
    let history_list = container
        .query_selector(".history-list")
        .expect("query ok");
    assert!(
        history_list.is_some(),
        "history list must be visible after clicking the back button"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

// ─── M9 extended coverage ─────────────────────────────────────────────────────

/// "View →" button on a History list row navigates to the detail view.
#[wasm_bindgen_test]
async fn history_view_view_button_navigates_to_detail() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let game = CompletedGame {
        id: "view-btn-nav-test".to_string(),
        completed_at: "2026-03-01T10:00:00Z".to_string(),
        final_score: 300,
        bonus_pool: 0,
        bonus_forfeited: false,
        cells: [[None; ROW_COUNT]; 6],
    };
    storage::save_history(&[game]).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Click the "View →" button.
    let view_btn = container
        .query_selector(".history-list__view-btn")
        .expect("query ok")
        .expect("View → button must exist in history list");
    view_btn.unchecked_ref::<HtmlElement>().click();
    tick().await;
    tick().await;

    // History list must be gone.
    let history_list = container
        .query_selector(".history-list")
        .expect("query ok");
    assert!(
        history_list.is_none(),
        "history list must be replaced by detail view after clicking View →"
    );

    // Detail view must be present.
    let detail = container
        .query_selector(".history-detail")
        .expect("query ok");
    assert!(
        detail.is_some(),
        "history detail view must appear after clicking View →"
    );

    // Scorecard must be present.
    let scorecard = container
        .query_selector(".scorecard")
        .expect("query ok");
    assert!(
        scorecard.is_some(),
        "scorecard must be visible on the detail view"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History detail view is read-only: no dice row, no roll button, no preview cells.
#[wasm_bindgen_test]
async fn history_detail_is_read_only() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let game = CompletedGame {
        id: "readonly-test".to_string(),
        completed_at: "2026-03-01T10:00:00Z".to_string(),
        final_score: 250,
        bonus_pool: 0,
        bonus_forfeited: false,
        cells: [[None; ROW_COUNT]; 6],
    };
    storage::save_history(&[game]).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history/readonly-test");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // No dice row.
    let dice_row = container.query_selector(".dice-row").expect("query ok");
    assert!(
        dice_row.is_none(),
        "history detail must NOT contain a dice row"
    );

    // No roll button.
    let buttons = container
        .query_selector_all(".action-buttons button")
        .expect("query ok");
    let has_roll = (0..buttons.length()).any(|i| {
        buttons
            .item(i)
            .map(|b| b.text_content().unwrap_or_default().contains("ROLL"))
            .unwrap_or(false)
    });
    assert!(!has_roll, "history detail must NOT contain a roll button");

    // No preview cells (only appear in active game scorecard).
    let preview_cells = container
        .query_selector_all(".scorecard__cell--preview, .scorecard__cell--zero-preview")
        .expect("query ok");
    assert_eq!(
        preview_cells.length(),
        0,
        "history detail scorecard must have zero preview cells"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History list applies --silver and --bronze medal classes to 2nd and 3rd entries.
#[wasm_bindgen_test]
async fn history_view_all_three_medal_classes_present() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let entries = vec![
        CompletedGame {
            id: "medal-a".to_string(),
            completed_at: "2026-03-01T00:00:00Z".to_string(),
            final_score: 400,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
        CompletedGame {
            id: "medal-b".to_string(),
            completed_at: "2026-03-02T00:00:00Z".to_string(),
            final_score: 300,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
        CompletedGame {
            id: "medal-c".to_string(),
            completed_at: "2026-03-03T00:00:00Z".to_string(),
            final_score: 200,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
    ];
    storage::save_history(&entries).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let gold = container
        .query_selector(".history-list__row--gold")
        .expect("query ok");
    assert!(gold.is_some(), "gold medal row must exist");

    let silver = container
        .query_selector(".history-list__row--silver")
        .expect("query ok");
    assert!(silver.is_some(), "silver medal row must exist");

    let bronze = container
        .query_selector(".history-list__row--bronze")
        .expect("query ok");
    assert!(bronze.is_some(), "bronze medal row must exist");

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// 4th and beyond entries have no medal class (plain `.history-list__row` only).
#[wasm_bindgen_test]
async fn history_view_fourth_entry_has_no_medal_class() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let entries: Vec<CompletedGame> = (0..4_u32)
        .map(|i| CompletedGame {
            id: format!("rank-{i}"),
            completed_at: "2026-03-01T00:00:00Z".to_string(),
            final_score: 400 - i * 50,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        })
        .collect();
    storage::save_history(&entries).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let rows = container
        .query_selector_all(".history-list__row")
        .expect("query ok");
    assert_eq!(rows.length(), 4, "four rows expected");

    // 4th row (index 3) must not have any medal modifier.
    let fourth_row = rows.item(3).expect("4th row must exist");
    let class_attr = fourth_row
        .unchecked_ref::<web_sys::Element>()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(
        !class_attr.contains("--gold")
            && !class_attr.contains("--silver")
            && !class_attr.contains("--bronze"),
        "4th entry must have no medal class, got: {class_attr}"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History list score cells are rendered in descending score order.
#[wasm_bindgen_test]
async fn history_view_score_cells_are_in_descending_order() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    // Insert in non-sorted order; save_history sorts descending.
    let entries = vec![
        CompletedGame {
            id: "sort-low".to_string(),
            completed_at: "2026-03-01T00:00:00Z".to_string(),
            final_score: 150,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
        CompletedGame {
            id: "sort-high".to_string(),
            completed_at: "2026-03-02T00:00:00Z".to_string(),
            final_score: 450,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
        CompletedGame {
            id: "sort-mid".to_string(),
            completed_at: "2026-03-03T00:00:00Z".to_string(),
            final_score: 300,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        },
    ];
    storage::save_history(&entries).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let score_cells = container
        .query_selector_all(".history-list__score")
        .expect("query ok");
    assert_eq!(score_cells.length(), 3, "three score cells expected");

    let scores: Vec<u32> = (0..score_cells.length())
        .filter_map(|i| {
            score_cells
                .item(i)
                .and_then(|n| n.text_content())
                .and_then(|t| t.trim().parse().ok())
        })
        .collect();

    assert_eq!(scores.len(), 3, "all score cells must have parseable text");
    assert!(
        scores[0] > scores[1] && scores[1] > scores[2],
        "score cells must be in descending order, got: {scores:?}"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History list rows display the correct final score and `+N` bonus pool text.
#[wasm_bindgen_test]
async fn history_view_row_shows_score_and_bonus_text() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let game = CompletedGame {
        id: "content-test".to_string(),
        completed_at: "2026-03-01T00:00:00Z".to_string(),
        final_score: 375,
        bonus_pool: 200,
        bonus_forfeited: false,
        cells: [[None; ROW_COUNT]; 6],
    };
    storage::save_history(&[game]).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    let score_cell = container
        .query_selector(".history-list__score")
        .expect("query ok")
        .expect("score cell must exist");
    let score_text = score_cell.text_content().unwrap_or_default();
    assert_eq!(
        score_text.trim(),
        "375",
        "score cell must show final_score, got: {score_text:?}"
    );

    let bonus_cell = container
        .query_selector(".history-list__bonus")
        .expect("query ok")
        .expect("bonus cell must exist");
    let bonus_text = bonus_cell.text_content().unwrap_or_default();
    assert_eq!(
        bonus_text.trim(),
        "+200",
        "bonus cell must show '+N' format, got: {bonus_text:?}"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

/// History detail header shows formatted date and final score.
#[wasm_bindgen_test]
async fn history_detail_header_shows_date_and_score() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::CompletedGame;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();

    let game = CompletedGame {
        id: "header-test".to_string(),
        completed_at: "2026-03-07T12:00:00Z".to_string(),
        final_score: 412,
        bonus_pool: 0,
        bonus_forfeited: false,
        cells: [[None; ROW_COUNT]; 6],
    };
    storage::save_history(&[game]).expect("save ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/history/header-test");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Date element must contain the month abbreviation and year.
    let date_el = container
        .query_selector(".history-detail__date")
        .expect("query ok")
        .expect(".history-detail__date must be present");
    let date_text = date_el.text_content().unwrap_or_default();
    assert!(
        date_text.contains("Mar"),
        ".history-detail__date must show formatted month, got: {date_text:?}"
    );
    assert!(
        date_text.contains("2026"),
        ".history-detail__date must show year, got: {date_text:?}"
    );

    // Score element must contain the final score.
    let score_el = container
        .query_selector(".history-detail__score")
        .expect("query ok")
        .expect(".history-detail__score must be present");
    let score_text = score_el.text_content().unwrap_or_default();
    assert!(
        score_text.contains("412"),
        ".history-detail__score must show final score, got: {score_text:?}"
    );

    ls_remove("rw_sixzee.history");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

// ─── M9 End-game → History navigation test ───────────────────────────────────

/// Clicking "View Full Scorecard" in the end-game overlay navigates to the
/// correct HistoryDetail view.
///
/// Covers the coverage gap identified in the M9 audit: the `on_view_scorecard`
/// click handler in `end_game.rs` sets `Route::HistoryDetail` and calls
/// `navigate()`. Tested by pre-seeding a fully-complete `GameState` in
/// `in_progress` (so the resume prompt fires) and the matching `CompletedGame`
/// in `history` (so `HistoryDetail` can look it up).
#[wasm_bindgen_test]
async fn end_game_view_full_scorecard_navigates_to_history_detail() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::{completed_game_from_state, new_game};
    use rw_sixzee::state::storage;

    clear_game_storage();

    // Build a fully-complete game state (all 6 × 13 cells filled with any value).
    let mut state = new_game();
    for col in state.cells.iter_mut() {
        for cell in col.iter_mut() {
            *cell = Some(5);
        }
    }
    state.turn = 78;

    let game_id = state.id.clone();
    let completed = completed_game_from_state(&state);

    // Pre-seed both storage entries.
    storage::save_in_progress(&state).expect("save in_progress ok");
    storage::save_history(&[completed]).expect("save history ok");

    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Resume prompt must appear (saved in-progress game was found).
    let resume_btn = container
        .query_selector(".resume-prompt .btn--primary")
        .expect("query ok")
        .expect("Resume button must be present");
    resume_btn.unchecked_ref::<HtmlElement>().click();
    tick().await;
    tick().await;
    tick().await;

    // EndGame overlay must be visible because all cells are filled.
    let end_game = container
        .query_selector(".overlay--end-game")
        .expect("query ok");
    assert!(
        end_game.is_some(),
        "end-game overlay must appear when all cells are filled"
    );

    // Find and click "View Full Scorecard".
    let buttons = container
        .query_selector_all(".overlay--end-game button")
        .expect("query ok");
    let mut view_btn: Option<web_sys::Element> = None;
    for i in 0..buttons.length() {
        let btn = buttons.item(i).expect("item");
        if btn
            .text_content()
            .unwrap_or_default()
            .contains("View Full Scorecard")
        {
            view_btn = Some(btn.unchecked_ref::<web_sys::Element>().clone());
            break;
        }
    }
    let view_btn = view_btn.expect("View Full Scorecard button must exist in end-game overlay");
    view_btn.unchecked_ref::<HtmlElement>().click();
    tick().await;
    tick().await;

    // HistoryDetail must now be rendered.
    let detail = container
        .query_selector(".history-detail")
        .expect("query ok");
    assert!(
        detail.is_some(),
        "history-detail must render after clicking View Full Scorecard"
    );

    // URL hash must reference the correct game id.
    let hash = web_sys::window()
        .expect("window")
        .location()
        .hash()
        .unwrap_or_default();
    assert!(
        hash.contains(&game_id),
        "URL hash must contain the game id; hash={hash:?}, id={game_id:?}"
    );

    // Cleanup.
    ls_remove("rw_sixzee.history");
    ls_remove("rw_sixzee.in_progress");
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }
}

// ─── M10 Accessibility & Polish tests ────────────────────────────────────────

/// Active tab button has aria-current="page"; inactive ones have aria-current="false".
#[wasm_bindgen_test]
async fn tab_bar_active_tab_has_aria_current_page() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();

    // Start on Game tab (default).
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Dismiss opening quote if present.
    dismiss_opening_quote(&container).await;

    let buttons = container
        .query_selector_all(".tab-bar__item")
        .expect("query ok");
    assert_eq!(buttons.length(), 3, "expected 3 tab buttons");

    let mut page_count = 0u32;
    let mut false_count = 0u32;
    for i in 0..buttons.length() {
        let btn = buttons.item(i).expect("item");
        let val = btn
            .unchecked_ref::<web_sys::Element>()
            .get_attribute("aria-current")
            .unwrap_or_default();
        if val == "page" {
            page_count += 1;
        }
        if val == "false" {
            false_count += 1;
        }
    }
    assert_eq!(page_count, 1, "exactly one tab must have aria-current=page");
    assert_eq!(false_count, 2, "the other two tabs must have aria-current=false");
}

/// Tab bar is hidden while ConfirmZero overlay is visible (PendingZero context drives HideTabBar).
#[wasm_bindgen_test]
async fn tab_bar_hidden_while_confirm_zero_visible() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }

    // Pre-fill all cells except (col=0, row=0) so a single roll + click
    // on that cell after scoring would be Ones. We need the cell to be open
    // and the preview value to be 0 so the ConfirmZero overlay appears.
    // Use dice [2,2,2,2,2] → score_ones = 0, triggering ConfirmZero.
    // We'll set up the game in localStorage with all cells pre-filled except (0,0),
    // then mount, roll, and click the zero-preview cell.
    let mut g = new_game();
    // Fill all cells except (col=0, row=0 = Ones).
    for col in 0..6_usize {
        for row in 0..ROW_COUNT {
            if !(col == 0 && row == 0) {
                g.cells[col][row] = Some(10);
            }
        }
    }
    // Set dice to all-twos (Ones preview = 0).
    g.dice = [Some(2); 5];
    g.rolls_used = 1;
    storage::save_in_progress(&g).expect("save ok");

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Resume the saved game.
    let resume_btn = container.query_selector(".resume-prompt .btn--primary").expect("query");
    if let Some(btn) = resume_btn {
        btn.unchecked_ref::<HtmlElement>().click();
        tick().await;
        tick().await;
    }

    // Verify tab bar is currently visible before clicking zero cell.
    let tab_bar = container
        .query_selector(".tab-bar")
        .expect("query")
        .expect("tab-bar must exist");
    let style_before = tab_bar
        .unchecked_ref::<web_sys::Element>()
        .get_attribute("style")
        .unwrap_or_default();
    assert!(
        !style_before.contains("display: none"),
        "tab bar must be visible before confirm-zero; style={style_before:?}"
    );

    // Click the zero-preview cell (col=0, row=0 = Ones with [2,2,2,2,2] dice).
    let zero_cells = container
        .query_selector_all(".scorecard__cell--zero-preview")
        .expect("query");
    assert!(zero_cells.length() > 0, "at least one zero-preview cell must exist");
    zero_cells
        .item(0)
        .expect("item")
        .unchecked_ref::<HtmlElement>()
        .click();
    tick().await;
    tick().await;

    // Confirm-zero overlay must be visible.
    let overlay = container
        .query_selector(".overlay--confirm")
        .expect("query")
        .expect("confirm-zero overlay must appear");
    assert_eq!(
        overlay.unchecked_ref::<web_sys::Element>()
            .get_attribute("role")
            .unwrap_or_default(),
        "dialog",
        "confirm-zero overlay must have role=dialog"
    );

    // Tab bar must now be hidden.
    let style_after = tab_bar
        .unchecked_ref::<web_sys::Element>()
        .get_attribute("style")
        .unwrap_or_default();
    assert!(
        style_after.contains("display: none"),
        "tab bar must be hidden while confirm-zero is open; style={style_after:?}"
    );

    ls_remove("rw_sixzee.in_progress");
}

/// End-game overlay has role="dialog" and aria-modal="true".
#[wasm_bindgen_test]
async fn overlays_have_role_dialog_and_aria_modal() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;
    use rw_sixzee::state::game::new_game;
    use rw_sixzee::state::scoring::ROW_COUNT;
    use rw_sixzee::state::storage;

    clear_game_storage();
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }

    // Build a near-complete game (all cells filled) — one more roll + score triggers EndGame.
    let mut g = new_game();
    for col in 0..6_usize {
        for row in 0..ROW_COUNT {
            if !(col == 5 && row == 12) {
                // leave (col=5, row=12 = Chance) open
                g.cells[col][row] = Some(5);
            }
        }
    }
    // Set dice to [1,2,3,4,5] (Chance = 15 > 0, so no ConfirmZero).
    g.dice = [Some(1), Some(2), Some(3), Some(4), Some(5)];
    g.rolls_used = 1;
    storage::save_in_progress(&g).expect("save ok");

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Resume.
    let resume_btn = container.query_selector(".resume-prompt .btn--primary").expect("query");
    if let Some(btn) = resume_btn {
        btn.unchecked_ref::<HtmlElement>().click();
        tick().await;
        tick().await;
    }

    // Click the Chance preview cell on column 6 (col=5, row=12).
    let preview_cells = container
        .query_selector_all(".scorecard__cell--preview")
        .expect("query");
    // There should be exactly one preview cell — the Chance cell.
    assert_eq!(preview_cells.length(), 1, "exactly one preview cell (Chance, col 6)");
    preview_cells
        .item(0)
        .expect("item")
        .unchecked_ref::<HtmlElement>()
        .click();
    tick().await;
    tick().await;

    // End-game overlay must appear.
    let end_overlay = container
        .query_selector(".overlay--end-game")
        .expect("query")
        .expect("end-game overlay must appear");
    assert_eq!(
        end_overlay.unchecked_ref::<web_sys::Element>()
            .get_attribute("role")
            .unwrap_or_default(),
        "dialog",
        "end-game overlay must have role=dialog"
    );
    assert_eq!(
        end_overlay.unchecked_ref::<web_sys::Element>()
            .get_attribute("aria-modal")
            .unwrap_or_default(),
        "true",
        "end-game overlay must have aria-modal=true"
    );

    ls_remove("rw_sixzee.in_progress");
}

/// Scorecard cells have an aria-label attribute describing their state.
#[wasm_bindgen_test]
async fn scorecard_cells_have_aria_label() {
    use leptos::mount::mount_to;
    use rw_sixzee::app::App;

    clear_game_storage();
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_hash("/");
    }

    let container = fresh_container();
    let _handle = mount_to(container.clone(), App);
    tick().await;
    tick().await;

    // Dismiss opening quote.
    dismiss_opening_quote(&container).await;

    // Before rolling, cells are in 'open' state and should have aria-labels.
    let open_cells = container
        .query_selector_all(".scorecard__cell--open")
        .expect("query");
    assert!(open_cells.length() > 0, "open cells must exist before rolling");

    let first_open = open_cells.item(0).expect("item");
    let label = first_open
        .unchecked_ref::<web_sys::Element>()
        .get_attribute("aria-label")
        .unwrap_or_default();
    assert!(
        !label.is_empty(),
        "open scorecard cell must have a non-empty aria-label"
    );
    assert!(
        label.contains("empty"),
        "open cell aria-label must contain 'empty'; got: {label:?}"
    );
}
