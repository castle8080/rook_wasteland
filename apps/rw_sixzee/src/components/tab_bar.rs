use leptos::prelude::*;
use crate::router::{navigate, Route};

/// Persistent bottom tab bar. Hidden (via inline style) while the resume
/// prompt overlay or any game overlay (confirm_zero, opening quote) is visible.
/// Active tab is highlighted.
#[component]
pub fn TabBar() -> impl IntoView {
    let route =
        use_context::<RwSignal<Route>>().expect("route context must be provided");
    let show_resume =
        use_context::<RwSignal<bool>>().expect("show_resume context must be provided");
    let hide_tab_bar =
        use_context::<RwSignal<bool>>().expect("hide_tab_bar context must be provided");

    let tab_style = move || {
        if show_resume.get() || hide_tab_bar.get() {
            "display: none;"
        } else {
            ""
        }
    };

    let on_game = move |_| {
        route.set(Route::Game);
        navigate(&Route::Game);
    };
    let on_history = move |_| {
        route.set(Route::History);
        navigate(&Route::History);
    };
    let on_settings = move |_| {
        route.set(Route::Settings);
        navigate(&Route::Settings);
    };

    view! {
        <nav class="tab-bar" style=tab_style>
            <button
                class=move || {
                    if matches!(route.get(), Route::Game) {
                        "tab-bar__item tab-bar__item--active"
                    } else {
                        "tab-bar__item"
                    }
                }
                on:click=on_game
            >
                "🎲 Game"
            </button>
            <button
                class=move || {
                    if matches!(route.get(), Route::History | Route::HistoryDetail { .. }) {
                        "tab-bar__item tab-bar__item--active"
                    } else {
                        "tab-bar__item"
                    }
                }
                on:click=on_history
            >
                "📋 History"
            </button>
            <button
                class=move || {
                    if matches!(route.get(), Route::Settings) {
                        "tab-bar__item tab-bar__item--active"
                    } else {
                        "tab-bar__item"
                    }
                }
                on:click=on_settings
            >
                "⚙️ Settings"
            </button>
        </nav>
    }
}
