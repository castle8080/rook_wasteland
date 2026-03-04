use leptos::prelude::*;
use gloo_events::EventListener;

use crate::routing::Route;
use crate::components::{deck::DeckView, header::Header};

/// Root application component.
///
/// Reads the initial URL hash to set the starting route, then listens for
/// `hashchange` events so browser back/forward navigation keeps the route
/// signal in sync. Provides the `RwSignal<Route>` via Leptos context so any
/// descendant can read the current route.
#[component]
pub fn App() -> impl IntoView {
    let win = web_sys::window().expect("window unavailable");

    let initial_hash = win.location().hash().unwrap_or_default();
    let current_route = RwSignal::new(Route::from_hash(&initial_hash));

    // Keep the listener alive for the entire app lifetime.
    // `std::mem::forget` prevents the drop that would call `removeEventListener`.
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

    view! {
        <div id="rw-mixit-root">
            <Header/>
            <Show when=move || current_route.get() == Route::Main>
                <DeckView/>
            </Show>
            <Show when=move || current_route.get() == Route::Settings>
                <div class="placeholder-view">"Settings — coming soon"</div>
            </Show>
            <Show when=move || current_route.get() == Route::About>
                <div class="placeholder-view">"About — coming soon"</div>
            </Show>
        </div>
    }
}
