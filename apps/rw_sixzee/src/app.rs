use leptos::prelude::*;
use wasm_bindgen::prelude::*;

use crate::components::{
    error_banner::ErrorBanner,
    error_overlay::ErrorOverlay,
    game_view::GameView,
    history::HistoryView,
    resume::ResumePrompt,
    settings::SettingsView,
    tab_bar::TabBar,
};
use crate::error::AppError;
use crate::router::{parse_hash, Route};

fn get_initial_route() -> Route {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .map(|h| parse_hash(&h))
        .unwrap_or(Route::Game)
}

fn set_body_theme(theme: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(body) = doc.body() {
                let _ = body.set_attribute("data-theme", theme);
            }
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let route: RwSignal<Route> = RwSignal::new(get_initial_route());
    let app_error: RwSignal<Option<AppError>> = RwSignal::new(None);
    let show_resume: RwSignal<bool> = RwSignal::new(false);

    provide_context(route);
    provide_context(app_error);
    provide_context(show_resume);

    set_body_theme("nordic_minimal");

    // Register the hashchange listener using a raw wasm-bindgen Closure so
    // we avoid the Send+Sync requirement that on_cleanup imposes. App is
    // mounted once and never unmounted in a SPA, so cb.forget() is correct.
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_event: web_sys::Event| {
        if let Some(win) = web_sys::window() {
            let hash = win.location().hash().unwrap_or_default();
            route.set(parse_hash(&hash));
        }
    });
    let window = web_sys::window().expect("App must run in a browser context");
    window
        .add_event_listener_with_callback("hashchange", cb.as_ref().unchecked_ref())
        .expect("failed to register hashchange listener");
    cb.forget();

    view! {
        <div class="app">
            <ErrorBanner />
            {move || {
                if show_resume.get() {
                    view! { <ResumePrompt /> }.into_any()
                } else {
                    match route.get() {
                        Route::Game => view! { <GameView /> }.into_any(),
                        Route::History => view! { <HistoryView /> }.into_any(),
                        Route::HistoryDetail { id } => {
                            view! { <div class="placeholder">"History: " {id}</div> }.into_any()
                        }
                        Route::Settings => view! { <SettingsView /> }.into_any(),
                    }
                }
            }}
            <TabBar />
            <ErrorOverlay />
        </div>
    }
}
