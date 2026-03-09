//! Quit-game confirmation overlay.
//!
//! Shown when the player taps "Quit Game" in the game menu. Displays a random
//! Grandma quit quote and two actions: "Quit" (destructive) and "Keep Playing".
//! Tapping the backdrop also cancels, consistent with `GrandmaQuoteOverlay`.

use leptos::prelude::*;

use crate::components::grandma_quote::GrandmaQuoteInline;
use crate::state::quotes::{pick_quote, QuoteBank};

/// Quit-game confirmation overlay.
///
/// Picks a random quote from `QuoteBank.quit` once on mount. Tapping the
/// backdrop or "Keep Playing" calls `on_cancel`. Tapping "Quit" calls
/// `on_confirm`.
#[component]
pub fn ConfirmQuit(
    /// Called when the player confirms they want to quit.
    on_confirm: Callback<()>,
    /// Called when the player cancels and wants to keep playing.
    on_cancel: Callback<()>,
) -> impl IntoView {
    let quote_bank =
        use_context::<RwSignal<Option<QuoteBank>>>().expect("quote_bank context must be provided");

    // Resolve a quit quote once on mount.
    let quit_quote: Option<String> = quote_bank
        .get_untracked()
        .as_ref()
        .and_then(|b| pick_quote(&b.quit))
        .map(str::to_owned);

    let cancel = move |_| on_cancel.run(());
    let confirm = move |_| on_confirm.run(());

    view! {
        // Backdrop: clicking it cancels.
        <div class="overlay overlay--quit" on:click=cancel>
            // Inner card: stop propagation so clicks inside don't also cancel.
            <div
                class="overlay__box"
                on:click=|ev| ev.stop_propagation()
            >
                <h3>"Quit this game?"</h3>
                <p>
                    "Your progress "
                    <strong>"will not be saved."</strong>
                </p>
                {quit_quote.map(|q| view! { <GrandmaQuoteInline quote=q /> })}
                <div class="overlay__actions">
                    <button
                        class="btn btn--secondary"
                        on:click=cancel
                    >
                        "Keep Playing"
                    </button>
                    <button
                        class="btn btn--danger"
                        on:click=confirm
                    >
                        "Quit"
                    </button>
                </div>
            </div>
        </div>
    }
}
