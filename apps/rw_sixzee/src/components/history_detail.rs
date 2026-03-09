//! History Detail view — read-only scorecard snapshot for a completed game.

use leptos::prelude::*;

use crate::components::scorecard::ScorecardReadOnly;
use crate::router::{navigate, Route};
use crate::state::storage;

/// History Detail view for the game with the given `id`.
///
/// Looks up the `CompletedGame` in `localStorage`. Renders a read-only scorecard
/// with the final cell values, bonus pool, and grand total. Provides a
/// `[ ← History ]` back button that returns to the History list.
#[component]
pub fn HistoryDetail(
    /// The game UUID from the `#/history/:id` route.
    id: String,
) -> impl IntoView {
    let route =
        use_context::<RwSignal<Route>>().expect("route context must be provided");

    let history = storage::load_history().unwrap_or_default();
    let game = history.into_iter().find(|g| g.id == id);

    let on_back = move |_| {
        route.set(Route::History);
        navigate(&Route::History);
    };

    view! {
        <div class="history-detail">
            {match game {
                None => view! {
                    <div class="history-detail__not-found">
                        <p class="history-detail__not-found-msg">"Game not found."</p>
                        <button class="btn btn--secondary" on:click=on_back>
                            "← History"
                        </button>
                    </div>
                }
                .into_any(),
                Some(g) => {
                    let date_str = format_date(&g.completed_at);
                    let final_score = g.final_score;
                    let bonus_pool = g.bonus_pool;
                    let bonus_forfeited = g.bonus_forfeited;
                    let cells = g.cells;
                    view! {
                        <div class="history-detail__header">
                            <button class="btn btn--secondary history-detail__back-btn" on:click=on_back>
                                "← History"
                            </button>
                            <div class="history-detail__meta">
                                <span class="history-detail__date">{date_str}</span>
                                <span class="history-detail__score">
                                    {format!("Final Score: {final_score}")}
                                </span>
                            </div>
                        </div>
                        <ScorecardReadOnly
                            cells=cells
                            bonus_pool=bonus_pool
                            bonus_forfeited=bonus_forfeited
                        />
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

// ─── Date formatting helper ───────────────────────────────────────────────────

/// Parse `"YYYY-MM-DDTHH:MM:SS…"` → `"Mon D, YYYY"`. Falls back to raw string.
fn format_date(iso: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let date_part = iso.split('T').next().unwrap_or(iso);
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3 {
        if let (Ok(year), Ok(month), Ok(day)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<usize>(),
            parts[2].parse::<u32>(),
        ) {
            let month_name = MONTHS.get(month.saturating_sub(1)).copied().unwrap_or("?");
            return format!("{month_name} {day}, {year}");
        }
    }
    iso.to_string()
}
