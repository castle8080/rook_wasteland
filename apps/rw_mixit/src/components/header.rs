use leptos::prelude::*;

use crate::routing::Route;

/// Top navigation bar: logo (links to main view) and nav links for Settings
/// and About views.
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="rw-header">
            <a
                class="rw-logo"
                href="#/"
                on:click=move |e| {
                    e.prevent_default();
                    let _ = web_sys::window()
                        .expect("window unavailable")
                        .location()
                        .set_hash(Route::Main.to_hash());
                }
            >
                "rw_mixit"
            </a>
            <nav class="rw-nav">
                <a
                    href="#/settings"
                    on:click=move |e| {
                        e.prevent_default();
                        let _ = web_sys::window()
                            .expect("window unavailable")
                            .location()
                            .set_hash(Route::Settings.to_hash());
                    }
                >
                    "[Settings]"
                </a>
                <a
                    href="#/about"
                    on:click=move |e| {
                        e.prevent_default();
                        let _ = web_sys::window()
                            .expect("window unavailable")
                            .location()
                            .set_hash(Route::About.to_hash());
                    }
                >
                    "[About]"
                </a>
            </nav>
        </header>
    }
}

