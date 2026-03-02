use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::routing::{Route, parse_hash};
use crate::ui::{
    components::TopBar, reader::ReaderView, recording_detail::RecordingDetailView,
    recordings_list::RecordingsListView,
};

/// Top-level App component. Uses hand-rolled hash routing so the app works
/// on any static file server without server-side catch-all configuration.
/// Navigation is driven by `window.location.hash`; a `hashchange` listener
/// keeps the route signal in sync, giving back/forward support for free.
#[component]
pub fn App() -> impl IntoView {
    // Read initial route from whatever hash is in the URL on load.
    let initial_hash = web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default();

    let route: RwSignal<Route> = RwSignal::new(parse_hash(&initial_hash));

    // Provide the route signal as context so any component can navigate.
    provide_context(route);

    // Listen for hashchange events (covers back/forward and <a href="#/..."> clicks).
    let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
        if let Some(hash) = web_sys::window().and_then(|w| w.location().hash().ok()) {
            route.set(parse_hash(&hash));
        }
    });
    if let Some(window) = web_sys::window() {
        let _ = window
            .add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref());
    }
    // App is never unmounted, so this listener lives for the page lifetime.
    closure.forget();

    view! {
        <TopBar />
        {move || match route.get() {
            Route::Reader { poem_id } => view! {
                <ReaderView poem_id=poem_id />
            }.into_any(),
            Route::RecordingsList => view! {
                <RecordingsListView />
            }.into_any(),
            Route::RecordingDetail(id) => view! {
                <RecordingDetailView recording_id=id />
            }.into_any(),
        }}
    }
}

