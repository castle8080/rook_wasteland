use leptos::prelude::*;

/// Placeholder recordings list view — full implementation in T09.
#[component]
pub fn RecordingsListView() -> impl IntoView {
    view! {
        <main class="content-column">
            <h1>"Recordings"</h1>
            <p class="text-secondary">"No recordings yet."</p>
        </main>
    }
}
