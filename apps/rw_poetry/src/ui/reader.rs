use leptos::prelude::*;

/// Placeholder reader view — full implementation in T04.
#[component]
pub fn ReaderView() -> impl IntoView {
    view! {
        <main class="content-column">
            <p class="text-secondary">"Loading poem…"</p>
        </main>
    }
}
