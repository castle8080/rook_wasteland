//! History list screen — all completed games sorted by final score.

use leptos::prelude::*;

use crate::router::{navigate, Route};
use crate::state::storage;

// ─── Date formatting ─────────────────────────────────────────────────────────

/// Parse an ISO 8601 timestamp (`"YYYY-MM-DDTHH:MM:SS…"`) into `"Mon D, YYYY"`.
///
/// Falls back to the raw string if parsing fails.
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

/// Returns `"🥇"`, `"🥈"`, `"🥉"` for ranks 1–3; the numeric rank otherwise.
fn rank_display(rank: usize) -> String {
    match rank {
        1 => "🥇".to_string(),
        2 => "🥈".to_string(),
        3 => "🥉".to_string(),
        n => n.to_string(),
    }
}

/// BEM modifier class for a history list row based on its rank.
fn row_class(rank: usize) -> &'static str {
    match rank {
        1 => "history-list__row history-list__row--gold",
        2 => "history-list__row history-list__row--silver",
        3 => "history-list__row history-list__row--bronze",
        _ => "history-list__row",
    }
}

// ─── Component ───────────────────────────────────────────────────────────────

/// History list view — all completed games ranked by final score.
///
/// Loads from `localStorage` on mount. If storage is unavailable the empty-state
/// message is shown rather than surfacing an error (read-only, non-critical path).
#[component]
pub fn HistoryView() -> impl IntoView {
    let route =
        use_context::<RwSignal<Route>>().expect("route context must be provided");

    // Sync localStorage read — history is small and pre-sorted by save_history().
    let history = storage::load_history().unwrap_or_default();

    view! {
        <div class="history-list">
            <h2 class="history-list__header">"📋  Game History"</h2>
            {if history.is_empty() {
                view! {
                    <p class="history-list__empty">
                        "No completed games yet. Finish your first game!"
                    </p>
                }
                .into_any()
            } else {
                view! {
                    <table class="history-list__table">
                        <thead>
                            <tr>
                                <th>"Rank"</th>
                                <th>"Date"</th>
                                <th>"Score"</th>
                                <th>"Bonus"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {history
                                .into_iter()
                                .enumerate()
                                .map(|(i, game)| {
                                    let rank = i + 1;
                                    let id = game.id.clone();
                                    view! {
                                        <tr class=row_class(rank)>
                                            <td class="history-list__rank">
                                                {rank_display(rank)}
                                            </td>
                                            <td class="history-list__date">
                                                {format_date(&game.completed_at)}
                                            </td>
                                            <td class="history-list__score">
                                                {game.final_score.to_string()}
                                            </td>
                                            <td class="history-list__bonus">
                                                {format!("+{}", game.bonus_pool)}
                                            </td>
                                            <td class="history-list__action">
                                                <button
                                                    class="btn btn--secondary history-list__view-btn"
                                                    on:click=move |_| {
                                                        let dest = Route::HistoryDetail {
                                                            id: id.clone(),
                                                        };
                                                        route.set(dest.clone());
                                                        navigate(&dest);
                                                    }
                                                >
                                                    "View →"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                }
                .into_any()
            }}
        </div>
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_date_parses_iso_to_human_readable() {
        assert_eq!(format_date("2026-03-07T12:34:56.789Z"), "Mar 7, 2026");
        assert_eq!(format_date("2025-01-01T00:00:00Z"), "Jan 1, 2025");
        assert_eq!(format_date("2024-12-31T23:59:59Z"), "Dec 31, 2024");
    }

    #[test]
    fn format_date_falls_back_on_malformed_input() {
        // Missing 'T' separator but has valid date portion.
        assert_eq!(format_date("2026-03-07"), "Mar 7, 2026");
        // Completely invalid — returns raw string.
        assert_eq!(format_date("not-a-date"), "not-a-date");
        // Empty string.
        assert_eq!(format_date(""), "");
    }

    #[test]
    fn rank_display_returns_medals_for_top_three() {
        assert_eq!(rank_display(1), "🥇");
        assert_eq!(rank_display(2), "🥈");
        assert_eq!(rank_display(3), "🥉");
    }

    #[test]
    fn rank_display_returns_numeric_for_rank_four_plus() {
        assert_eq!(rank_display(4), "4");
        assert_eq!(rank_display(10), "10");
        assert_eq!(rank_display(99), "99");
    }
}

