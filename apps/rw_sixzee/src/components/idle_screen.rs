//! Pre-game idle screen.
//!
//! Rendered on the Game tab when no active game is in progress (i.e., after a
//! mid-game quit). Shows the SIXZEE wordmark, a random Grandma opening quote,
//! and a prominent "Start New Game" button.

use leptos::prelude::*;

use crate::components::grandma_quote::GrandmaQuoteInline;
use crate::state::quotes::{pick_quote, QuoteBank};

/// Pre-game idle screen.
///
/// `on_start_game` is called when the player taps "Start New Game". The caller
/// is responsible for resetting `GameState` and triggering the opening-quote
/// overlay.
#[component]
pub fn IdleScreen(
    /// Called when the player taps "Start New Game".
    on_start_game: Callback<()>,
) -> impl IntoView {
    let quote_bank =
        use_context::<RwSignal<Option<QuoteBank>>>().expect("quote_bank context must be provided");

    // Pick an opening quote once on mount.
    let idle_quote: Option<String> = quote_bank
        .get_untracked()
        .as_ref()
        .and_then(|b| pick_quote(&b.opening))
        .map(str::to_owned);

    let start = move |_| on_start_game.run(());

    view! {
        <div class="idle-screen">
            <div class="idle-screen__title">"SIXZEE"</div>
            {idle_quote.map(|q| view! { <GrandmaQuoteInline quote=q /> })}
            <button
                class="btn btn--primary idle-screen__start-btn"
                on:click=start
            >
                "Start New Game"
            </button>
        </div>
    }
}
