use leptos::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use gloo_events::EventListener;
use wasm_bindgen::prelude::*;

use crate::routing::Route;
use crate::components::{
    about::AboutView,
    deck::DeckView,
    header::Header,
    settings::SettingsView,
};
use crate::utils::viewport_scale::update_viewport_scale;

/// Root application component.
///
/// Reads the initial URL hash to set the starting route, then listens for
/// `hashchange` events so browser back/forward navigation keeps the route
/// signal in sync. Provides the `RwSignal<Route>` via Leptos context so any
/// descendant can read the current route.
///
/// `DeckView` is **always mounted** (hidden via CSS when not on the Main route)
/// so its audio graph, Effects, and keyboard shortcuts stay alive while
/// navigating to Settings / About.  Settings can then read and mutate deck
/// state via Leptos context, and the audio Effects fire immediately.
///
/// A debounced `resize` listener keeps `--app-scale` in sync as the user
/// resizes the window — see [`crate::utils::viewport_scale`].
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

    // Apply scale immediately so the first render is already at the right size.
    update_viewport_scale();

    // Debounced resize listener: recomputes --app-scale 100 ms after the user
    // stops resizing.  Same pattern as `hashchange` above — `std::mem::forget`
    // keeps the listener alive for the entire app lifetime.
    {
        // Stores the pending setTimeout handle so we can cancel it on rapid
        // resize events (debounce).
        let debounce_handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

        let resize_listener = EventListener::new(&win, "resize", move |_| {
            // Cancel any previously-scheduled update.
            if let Some(h) = debounce_handle.borrow_mut().take() {
                if let Some(w) = web_sys::window() {
                    w.clear_timeout_with_handle(h);
                }
            }

            // Schedule the update 100 ms from now.
            let cb = Closure::<dyn FnMut()>::new(|| {
                update_viewport_scale();
            });
            if let Some(w) = web_sys::window() {
                if let Ok(handle) = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    100,
                ) {
                    *debounce_handle.borrow_mut() = Some(handle);
                    // Only forget when the browser has taken ownership of the
                    // callback.  If set_timeout failed, `cb` drops normally here.
                    cb.forget();
                }
            }
        });
        std::mem::forget(resize_listener);
    }

    provide_context(current_route);

    view! {
        <div id="rw-mixit-root">
            <Header/>
            // DeckView stays mounted at all times so its audio graph, Effects,
            // and keyboard shortcuts survive route changes.  Hidden via CSS
            // when the user is on Settings or About.
            <div style=move || {
                if current_route.get() == Route::Main { "" } else { "display:none" }
            }>
                <DeckView/>
            </div>
            <Show when=move || current_route.get() == Route::Settings>
                <SettingsView/>
            </Show>
            <Show when=move || current_route.get() == Route::About>
                <AboutView/>
            </Show>
        </div>
    }
}
