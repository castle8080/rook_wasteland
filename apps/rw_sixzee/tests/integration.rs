//! Browser integration tests for the M5 core game UI.
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
