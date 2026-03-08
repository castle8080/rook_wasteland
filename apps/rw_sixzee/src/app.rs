use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::{
    error_banner::ErrorBanner,
    error_overlay::ErrorOverlay,
    game_view::GameView,
    grandma::GrandmaPanel,
    grandma_quote::GrandmaQuoteOverlay,
    history::HistoryView,
    resume::ResumePrompt,
    settings::SettingsView,
    tab_bar::TabBar,
};
use crate::error::{report_error, AppError};
use crate::router::{parse_hash, Route};
use crate::state::game::{new_game, prune_old_entries, score_preview_all, GameState};
use crate::state::quotes::{load_quote_bank, QuoteBank};
use crate::state::scoring::grand_total as compute_grand_total;
use crate::state::storage;
use crate::state::{HideTabBar, ShowOpeningQuote, ShowResume};
use crate::worker::{spawn_grandma_worker, GrandmaPanelState};

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

    // ── M6: saved game state waiting for resume decision ──────────────────
    let pending_resume: RwSignal<Option<GameState>> = RwSignal::new(None);

    // ── M7: Ask Grandma worker + panel state ──────────────────────────────
    let grandma_worker: RwSignal<Option<web_sys::Worker>> = RwSignal::new(None);
    let grandma_panel_state: RwSignal<GrandmaPanelState> =
        RwSignal::new(GrandmaPanelState::Closed);

    provide_context(route);
    provide_context(app_error);
    provide_context(ShowResume(show_resume));
    provide_context(game_signal);
    provide_context(grand_total);
    provide_context(score_preview);
    provide_context(quote_bank);
    provide_context(ShowOpeningQuote(show_opening_quote));
    provide_context(HideTabBar(hide_tab_bar));
    provide_context(pending_resume);
    provide_context(grandma_worker);
    provide_context(grandma_panel_state);

    // ── M6: App load sequence ─────────────────────────────────────────────
    // 1. Load and apply theme (best-effort; storage failure → Degraded).
    match storage::load_theme() {
        Ok(Some(theme)) => set_body_theme(&theme),
        Ok(None) => set_body_theme("nordic_minimal"),
        Err(e) => {
            set_body_theme("nordic_minimal");
            report_error(e);
        }
    }

    // 2. Prune history on load (best-effort, fire-and-forget).
    if let Ok(history) = storage::load_history() {
        let now_ms = js_sys::Date::now();
        let pruned = prune_old_entries(history, now_ms);
        let _ = storage::save_history(&pruned);
    }

    // 3. Load in-progress game.
    match storage::load_in_progress() {
        Ok(Some(saved)) => {
            // Saved game found — show the resume prompt instead of the opening quote.
            show_opening_quote.set(false);
            pending_resume.set(Some(saved));
            show_resume.set(true);
        }
        Ok(None) => {
            // No saved game — fresh start, opening quote will show normally.
        }
        Err(AppError::Json(_)) => {
            // Corrupt save — treat as Fatal so the user can start fresh.
            report_error(AppError::Json(
                "Saved game data is corrupt — please start a new game.".to_string(),
            ));
        }
        Err(e) => {
            // Storage unavailable — Degraded banner, game is still playable.
            report_error(e);
        }
    }

    // Keep hide_tab_bar in sync with overlay visibility.  Using an Effect (not
    // a signal-set inside the view closure) avoids a secondary reactive flush
    // that can cause the opening-quote overlay to persist after dismissal.
    Effect::new(move |_| {
        let quote_visible = show_opening_quote.get() && quote_bank.get().is_some();
        let grandma_open = !matches!(grandma_panel_state.get(), GrandmaPanelState::Closed);
        hide_tab_bar.set(quote_visible || show_resume.get() || grandma_open);
    });

    // ── M7: Spawn the grandma worker eagerly (best-effort; failure → Degraded) ─
    if let Err(e) = spawn_grandma_worker(grandma_worker, grandma_panel_state) {
        report_error(e);
    }

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
            <GrandmaPanel />
            {move || {
                // Opening-quote overlay: shown when a new game starts AND the
                // quote bank is loaded. If the bank hasn't loaded yet, the
                // overlay is skipped — the game starts immediately.
                let bank_ready = quote_bank.get().is_some();
                if show_opening_quote.get() && bank_ready {
                    let on_dismiss = Callback::new(move |_| {
                        show_opening_quote.set(false);
                        // hide_tab_bar is managed by the Effect above.
                    });
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

