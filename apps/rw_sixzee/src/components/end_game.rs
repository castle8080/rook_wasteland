//! End-of-game summary overlay.
//!
//! Appears when `is_game_complete()` returns true. Shows column totals, Sixzee
//! bonus pool, grand total, best column, and a closing Grandma quote. The tab
//! bar remains visible behind the overlay per spec §4.3.

use leptos::prelude::*;

use crate::components::grandma_quote::GrandmaQuoteInline;
use crate::router::{navigate, Route};
use crate::state::game::GameState;
use crate::state::quotes::{compute_tier, pick_quote, QuoteBank};
use crate::state::scoring::column_total;

/// End-of-game summary overlay.
///
/// `on_new_game` is called when the player taps "New Game". The caller should
/// reset `GameState` and set `show_opening_quote = true`.
#[component]
pub fn EndGame(
    /// Called when the player wants to start a new game.
    on_new_game: Callback<()>,
) -> impl IntoView {
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");
    let grand_total_memo =
        use_context::<Memo<u32>>().expect("grand_total context must be provided");
    let quote_bank =
        use_context::<RwSignal<Option<QuoteBank>>>().expect("quote_bank context must be provided");
    let route =
        use_context::<RwSignal<Route>>().expect("route context must be provided");

    // Capture current game state snapshot for the summary.
    let state = game_signal.get_untracked();
    let grand = grand_total_memo.get_untracked();

    // Compute column totals and find the best column.
    let col_totals: [u16; 6] =
        std::array::from_fn(|col| column_total(&state.cells[col]));
    let best_col = col_totals
        .iter()
        .enumerate()
        .max_by_key(|(_, &v)| v)
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Closing quote — resolved once on mount.
    let closing_quote: Option<String> = quote_bank
        .get_untracked()
        .as_ref()
        .and_then(|bank| {
            let tier = compute_tier(grand);
            pick_quote(tier.quotes(bank))
        })
        .map(str::to_owned);

    let game_id = state.id.clone();

    let on_view_scorecard = move |_| {
        let dest = Route::HistoryDetail { id: game_id.clone() };
        route.set(dest.clone());
        navigate(&dest);
    };

    let on_new = move |_| on_new_game.run(());

    let scorecard_total: u32 = col_totals.iter().map(|&v| v as u32).sum();
    let pool = state.bonus_pool;
    let forfeited = state.bonus_forfeited;

    view! {
        <div class="overlay overlay--end-game" role="dialog" aria-modal="true">
            <div class="overlay__box">
                <div class="end-game__title">"🎲  GAME COMPLETE  🎲"</div>

                <table class="end-game__scores">
                    <tbody>
                        // Column totals
                        {col_totals
                            .iter()
                            .enumerate()
                            .map(|(i, &v)| {
                                view! {
                                    <tr>
                                        <td>{format!("Column {}", i + 1)}</td>
                                        <td>{v.to_string()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                        <tr>
                            <td>"Scorecard Total"</td>
                            <td>{scorecard_total.to_string()}</td>
                        </tr>
                        <tr>
                            <td>
                                {if forfeited {
                                    "Sixzee Bonus Pool (forfeited)"
                                } else {
                                    "Sixzee Bonus Pool"
                                }}
                            </td>
                            <td>{format!("+{pool}")}</td>
                        </tr>
                    </tbody>
                </table>

                <div class="end-game__final-score">
                    {format!("FINAL SCORE: {grand}")}
                </div>

                <div class="end-game__best-col">
                    {format!(
                        "⭐  Best Column: Column {} — {} pts",
                        best_col + 1,
                        col_totals[best_col],
                    )}
                </div>

                {closing_quote.map(|q| view! { <GrandmaQuoteInline quote=q /> })}

                <div class="end-game__actions">
                    <button class="btn btn--primary" on:click=on_new>
                        "🎮  New Game"
                    </button>
                    <button class="btn btn--secondary" on:click=on_view_scorecard>
                        "📋  View Full Scorecard"
                    </button>
                </div>
            </div>
        </div>
    }
}
