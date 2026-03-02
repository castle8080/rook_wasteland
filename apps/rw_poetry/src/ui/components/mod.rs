use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

const THEME_KEY: &str = "theme";

fn apply_theme(theme: &str) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = el.set_attribute("data-theme", theme);
    }
}

/// Slim top bar: app name on the left, Recordings link and dark mode toggle on the right.
#[component]
pub fn TopBar() -> impl IntoView {
    // Read persisted theme (or default to light)
    let initial: String = LocalStorage::get(THEME_KEY).unwrap_or_else(|_| "light".to_string());
    apply_theme(&initial);

    let is_dark = RwSignal::new(initial == "dark");

    let on_toggle = move |_| {
        let dark = !is_dark.get_untracked();
        is_dark.set(dark);
        let theme = if dark { "dark" } else { "light" };
        apply_theme(theme);
        let _ = LocalStorage::set(THEME_KEY, theme);
    };

    view! {
        <header class="top-bar">
            <a href="#/" class="top-bar__title">"RW Poetry"</a>
            <nav class="top-bar__nav">
                <a href="#/readings" class="top-bar__link">"Recordings"</a>
                <button
                    class="btn btn-icon top-bar__theme-toggle"
                    aria-label=move || if is_dark.get() { "Switch to light mode" } else { "Switch to dark mode" }
                    on:click=on_toggle
                >
                    {move || if is_dark.get() { "☀" } else { "🌙" }}
                </button>
            </nav>
        </header>
    }
}
