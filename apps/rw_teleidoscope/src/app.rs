use leptos::prelude::*;
use gloo_events::EventListener;

use crate::routing::Route;
use crate::state::{AppState, KaleidoscopeParams};

/// Root application component.
///
/// Creates `KaleidoscopeParams` and `AppState`, provides both via Leptos context
/// so any descendant can access them, and renders the main layout. Hash-based
/// routing is wired up here so browser back/forward navigation keeps the route
/// signal in sync.
#[component]
pub fn App() -> impl IntoView {
    let win = web_sys::window().expect("window unavailable");

    let initial_hash = win.location().hash().unwrap_or_default();
    let current_route = RwSignal::new(Route::from_hash(&initial_hash));

    // Keep the listener alive for the entire app lifetime.
    let listener = EventListener::new(&win, "hashchange", move |_| {
        let hash = web_sys::window()
            .expect("window unavailable")
            .location()
            .hash()
            .unwrap_or_default();
        current_route.set(Route::from_hash(&hash));
    });
    std::mem::forget(listener);

    provide_context(current_route);
    provide_context(KaleidoscopeParams::new());
    provide_context(AppState::new());

    view! {
        <div id="rw-teleidoscope-root">
            <h1>"Teleidoscope"</h1>
            <p>"Canvas placeholder — rendering begins in M2."</p>
        </div>
    }
}
