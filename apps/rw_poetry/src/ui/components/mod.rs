use leptos::prelude::*;
use leptos_router::components::A;

/// Slim top bar: app name on the left, Recordings link on the right.
#[component]
pub fn TopBar() -> impl IntoView {
    view! {
        <header class="top-bar">
            <A href="/" attr:class="top-bar__title">"RW Poetry"</A>
            <nav class="top-bar__nav">
                <A href="/readings" attr:class="top-bar__link">"Recordings"</A>
            </nav>
        </header>
    }
}
