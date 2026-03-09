//! Zero-score confirmation overlay.
//!
//! Shown when the player clicks a cell that would score 0. For Sixzee cells
//! (row 11), a forfeiture warning is added. A random scratch quote from the
//! `QuoteBank` is shown if available.

use leptos::prelude::*;

use crate::components::grandma_quote::GrandmaQuoteInline;
use crate::state::game::GameState;
use crate::state::quotes::{pick_quote, QuoteBank};
use crate::state::scoring::{ROW_LABELS, ROW_SIXZEE};

/// Zero-score confirmation prompt.
///
/// Shown when the player clicks a cell that would score 0. The `PendingZero`
/// context signal is managed by the caller (`GameView`); `ConfirmZero` itself
/// does not write to `HideTabBar` — the `app.rs` Effect reacts to `PendingZero`
/// going `Some`/`None` automatically.
#[component]
pub fn ConfirmZero(
    /// Column index (0-based) of the cell being scratched.
    col: usize,
    /// Row index (0-based) of the cell being scratched.
    row: usize,
    /// Called when the player cancels.
    on_cancel: Callback<()>,
    /// Called when the player confirms placing 0.
    on_confirm: Callback<()>,
) -> impl IntoView {
    let quote_bank =
        use_context::<RwSignal<Option<QuoteBank>>>().expect("quote_bank context must be provided");

    // Resolve a scratch quote once on mount.
    let scratch_quote: Option<String> = quote_bank
        .get_untracked()
        .as_ref()
        .and_then(|b| pick_quote(&b.scratch))
        .map(str::to_owned);

    let row_name = ROW_LABELS.get(row).copied().unwrap_or("?");
    let is_sixzee = row == ROW_SIXZEE;
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");
    let already_forfeited = game_signal.get_untracked().bonus_forfeited;

    let cancel = move |_| on_cancel.run(());
    let confirm = move |_| on_confirm.run(());

    view! {
        <div class="overlay overlay--confirm" role="dialog" aria-modal="true">
            <div class="overlay__box">
                <h3>
                    {format!("Place 0 in {} — Col {}?", row_name, col + 1)}
                </h3>
                <p>
                    "This cell would score "
                    <strong>"0 points"</strong>
                    " with your current dice."
                </p>
                {scratch_quote.map(|q| view! { <GrandmaQuoteInline quote=q /> })}
                {if is_sixzee && !already_forfeited {
                    view! {
                        <div class="forfeit-warning">
                            <strong>"⚠️  WARNING"</strong>
                            "Scratching a Sixzee cell will permanently forfeit your entire Sixzee Bonus Pool."
                        </div>
                    }
                    .into_any()
                } else {
                    view! { <span /> }.into_any()
                }}
                <div class="overlay__actions">
                    <button class="btn btn--secondary" on:click=cancel>
                        "Cancel"
                    </button>
                    <button class="btn btn--primary" on:click=confirm>
                        "Confirm Zero"
                    </button>
                </div>
            </div>
        </div>
    }
}
