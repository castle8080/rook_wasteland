use leptos::prelude::*;
use gloo_events::EventListener;

use crate::components::camera_overlay::CameraOverlay;
use crate::components::canvas_view::CanvasView;
use crate::components::controls_panel::ControlsPanel;
use crate::components::header::Header;
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
    let app_state = AppState::new();
    provide_context(app_state);

    view! {
        <div id="rw-teleidoscope-root">
            <Header/>
            <div
                class="main-layout"
                class:is-panel-collapsed=move || !app_state.panel_open.get()
            >
                <ControlsPanel/>
                <CanvasView/>
            </div>
            // Mobile drawer handle — a persistent tab at the very bottom of the
            // screen.  Hidden on desktop via CSS (display: none above 768 px).
            <button
                class="drawer-handle"
                aria-label="Toggle controls drawer"
                on:click=move |_| app_state.drawer_open.update(|v| *v = !*v)
            >
                <span class="drawer-handle__pip"/>
            </button>
            // Dimmed backdrop behind the open drawer; tapping it closes the drawer.
            // Also hidden on desktop via CSS.
            <div
                class="drawer-backdrop"
                class:is-visible=move || app_state.drawer_open.get()
                on:click=move |_| app_state.drawer_open.set(false)
            />
            <CameraOverlay/>
        </div>
    }
}
