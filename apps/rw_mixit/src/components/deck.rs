use leptos::prelude::*;

use crate::components::mixer::Mixer;

/// Placeholder single-deck column, displayed before M1 wires up real audio.
#[component]
pub fn DeckPlaceholder(
    /// Deck identifier shown in the label — `"A"` or `"B"`.
    side: &'static str,
) -> impl IntoView {
    let css_class = format!("deck deck-{}", side.to_lowercase());
    let label = format!("DECK {side}");
    view! {
        <div class=css_class>
            <p class="deck-label">{label}</p>
        </div>
    }
}

/// Three-column layout: `[Deck A] [Mixer] [Deck B]`.
/// Deck columns are placeholders until M1; the Mixer column is a placeholder until M5.
#[component]
pub fn DeckView() -> impl IntoView {
    view! {
        <div class="deck-row">
            <DeckPlaceholder side="A"/>
            <Mixer/>
            <DeckPlaceholder side="B"/>
        </div>
    }
}
