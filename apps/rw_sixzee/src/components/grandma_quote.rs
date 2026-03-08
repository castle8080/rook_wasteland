//! Grandma quote UI components.
//!
//! - [`GrandmaQuoteOverlay`]: full-screen opening-quote overlay shown at game
//!   start. Reads from `RwSignal<Option<QuoteBank>>` in context.
//! - [`GrandmaQuoteInline`]: small styled quote block used for Sixzee inline
//!   quotes and as the scratch prompt inside `ConfirmZero`.

use leptos::prelude::*;

use crate::state::quotes::{pick_quote, QuoteBank};

/// Full-screen opening-quote overlay. Shown once per new game when the quote
/// bank is loaded. Dismissed by the "Let's play." button or tapping outside the
/// card.
///
/// The caller is responsible for hiding the tab bar while this overlay is
/// visible (via the `hide_tab_bar` context signal).
#[component]
pub fn GrandmaQuoteOverlay(
    /// Called when the player dismisses the overlay.
    on_dismiss: Callback<()>,
) -> impl IntoView {
    let quote_bank =
        use_context::<RwSignal<Option<QuoteBank>>>().expect("quote_bank context must be provided");

    // Pick the opening quote once when the component is first created.
    let opening_quote = {
        let bank = quote_bank.get_untracked();
        bank.as_ref()
            .and_then(|b| pick_quote(&b.opening))
            .map(str::to_owned)
    };

    let dismiss = move |_| on_dismiss.run(());

    view! {
        <div class="grandma-quote-overlay" on:click=dismiss>
            <div class="grandma-quote-overlay__title">"SIXZEE"</div>
            // Stop propagation so clicks inside the card don't also dismiss.
            <div
                class="grandma-quote-overlay__card"
                on:click=|ev| ev.stop_propagation()
            >
                {move || {
                    if let Some(ref q) = opening_quote {
                        view! {
                            <GrandmaQuoteInline quote=q.clone() />
                        }
                        .into_any()
                    } else {
                        view! { <span /> }.into_any()
                    }
                }}
            </div>
            <button class="btn btn--primary" on:click=dismiss>
                "Let's play."
            </button>
        </div>
    }
}

/// Small inline quote block. Renders a `👵 "…" — Grandma` display.
///
/// Used for Sixzee inline quotes (below the dice row) and scratch quotes
/// (inside `ConfirmZero`). Silently renders nothing when `quote` is empty.
#[component]
pub fn GrandmaQuoteInline(
    /// The quote text to display.
    quote: String,
) -> impl IntoView {
    if quote.is_empty() {
        return view! { <span /> }.into_any();
    }
    view! {
        <div class="grandma-quote-inline">
            <span class="grandma-quote__text">
                "👵 "" {quote} """
            </span>
            <span class="grandma-quote__attribution">" — Grandma"</span>
        </div>
    }
    .into_any()
}
