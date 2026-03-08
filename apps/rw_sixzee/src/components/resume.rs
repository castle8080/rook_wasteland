use leptos::prelude::*;

use crate::state::game::{new_game, GameState};
use crate::state::scoring::grand_total;
use crate::state::storage;

/// Full-screen overlay shown when an in-progress game is detected in
/// localStorage on app load. Presents game metadata and two choices:
///
/// - **Resume Game** — restores the saved `GameState` and enters the game.
/// - **Discard and Start New** — clears the save, starts a fresh game,
///   and shows the Grandma opening-quote overlay.
///
/// The tab bar is hidden while this prompt is visible (via `show_resume`).
#[component]
pub fn ResumePrompt() -> impl IntoView {
    let show_resume =
        use_context::<RwSignal<bool>>().expect("show_resume context must be provided");
    let pending_resume = use_context::<RwSignal<Option<GameState>>>()
        .expect("pending_resume context must be provided");
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");
    let show_opening_quote =
        use_context::<RwSignal<bool>>().expect("show_opening_quote context must be provided");

    // Snapshot the saved state at component mount. `pending_resume` is always
    // `Some` when `show_resume` is `true`.
    let saved = pending_resume
        .get_untracked()
        .expect("pending_resume must be Some when ResumePrompt is shown");

    let started_at = saved.started_at.clone();
    let turn_display = saved.turn + 1;
    let current_score = grand_total(&saved.cells, saved.bonus_pool);

    // Clone for the Resume closure (must be Fn, not FnOnce).
    let saved_for_resume = saved.clone();

    let on_resume = move |_| {
        game_signal.set(saved_for_resume.clone());
        pending_resume.set(None);
        show_resume.set(false);
        // show_opening_quote is already false (set in App load sequence)
    };

    let on_start_new = move |_| {
        // Best-effort clear; storage failure is non-fatal here.
        let _ = storage::clear_in_progress();
        game_signal.set(new_game());
        pending_resume.set(None);
        show_resume.set(false);
        show_opening_quote.set(true);
    };

    view! {
        <div class="overlay">
            <div class="overlay__box resume-prompt">
                <h2>"Resume Game?"</h2>
                <div class="resume-prompt__info">
                    <div class="resume-prompt__meta">
                        <span class="resume-prompt__meta-item">
                            <span class="resume-prompt__meta-label">"Started"</span>
                            <span class="resume-prompt__meta-value">{started_at}</span>
                        </span>
                        <span class="resume-prompt__meta-item">
                            <span class="resume-prompt__meta-label">"Turn"</span>
                            <span class="resume-prompt__meta-value">{turn_display.to_string()}</span>
                        </span>
                        <span class="resume-prompt__meta-item">
                            <span class="resume-prompt__meta-label">"Score"</span>
                            <span class="resume-prompt__meta-value">{current_score.to_string()}</span>
                        </span>
                    </div>
                </div>
                <div class="resume-prompt__actions">
                    <button class="btn btn--primary" on:click=on_resume>
                        "Resume Game"
                    </button>
                    <button class="btn btn--secondary" on:click=on_start_new>
                        "Discard and Start New"
                    </button>
                </div>
            </div>
        </div>
    }
}
