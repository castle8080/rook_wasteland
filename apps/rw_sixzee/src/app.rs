use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::{
    error_banner::ErrorBanner,
    error_overlay::ErrorOverlay,
    game_view::GameView,
    grandma_quote::GrandmaQuoteOverlay,
    history::HistoryView,
    resume::ResumePrompt,
    settings::SettingsView,
    tab_bar::TabBar,
};
use crate::error::{report_error, AppError};
use crate::router::{parse_hash, Route};
use crate::state::game::{new_game, score_preview_all, GameState};
use crate::state::quotes::{load_quote_bank, QuoteBank};
use crate::state::scoring::grand_total as compute_grand_total;

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

    // ── M5: game state and derived signals ────────────────────────────────
    let game_signal: RwSignal<GameState> = RwSignal::new(new_game());

    let grand_total: Memo<u32> = Memo::new(move |_| {
        let s = game_signal.get();
        compute_grand_total(&s.cells, s.bonus_pool)
    });

    let score_preview: Memo<[[u8; 13]; 6]> = Memo::new(move |_| {
        let s = game_signal.get();
        score_preview_all(&s)
    });

    let quote_bank: RwSignal<Option<QuoteBank>> = RwSignal::new(None);

    // `true` while the opening-quote overlay should be shown.
    let show_opening_quote: RwSignal<bool> = RwSignal::new(true);

    // Set `true` by any overlay that must hide the tab bar (confirm_zero,
    // opening-quote). Checked by `TabBar` alongside `show_resume`.
    let hide_tab_bar: RwSignal<bool> = RwSignal::new(false);

    provide_context(route);
    provide_context(app_error);
    provide_context(show_resume);
    provide_context(game_signal);
    provide_context(grand_total);
    provide_context(score_preview);
    provide_context(quote_bank);
    provide_context(show_opening_quote);
    provide_context(hide_tab_bar);

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

    // Load Grandma's quote bank asynchronously; failure is Degraded.
    spawn_local(async move {
        match load_quote_bank().await {
            Ok(bank) => quote_bank.set(Some(bank)),
            Err(e) => report_error(e),
        }
    });

    view! {
        <div class="app">
            <ErrorBanner />
            {move || {
                // Opening-quote overlay: shown when a new game starts AND the
                // quote bank is loaded. If the bank hasn't loaded yet, the
                // overlay is skipped — the game starts immediately.
                let bank_ready = quote_bank.get().is_some();
                if show_opening_quote.get() && bank_ready {
                    let on_dismiss = Callback::new(move |_| {
                        show_opening_quote.set(false);
                        hide_tab_bar.set(false);
                    });
                    // Ensure the tab bar is hidden while the overlay is up.
                    hide_tab_bar.set(true);
                    return view! { <GrandmaQuoteOverlay on_dismiss=on_dismiss /> }.into_any();
                }
                if show_resume.get() {
                    return view! { <ResumePrompt /> }.into_any();
                }
                match route.get() {
                    Route::Game => view! { <GameView /> }.into_any(),
                    Route::History => view! { <HistoryView /> }.into_any(),
                    Route::HistoryDetail { id } => {
                        view! { <div class="placeholder">"History: " {id}</div> }.into_any()
                    }
                    Route::Settings => view! { <SettingsView /> }.into_any(),
                }
            }}
            <TabBar />
            <ErrorOverlay />
        </div>
    }
}

