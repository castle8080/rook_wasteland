//! Scorecard component — 6-column × 13-row scoring grid.
//!
//! Reads `game_signal`, `score_preview`, and `grand_total` from context.
//! Open cells show a preview after rolling; filled cells display their stored
//! value. The `on_cell_click` callback is invoked with `(col, row)` when the
//! player clicks an open cell.

use leptos::prelude::*;

use crate::state::game::GameState;
use crate::state::scoring::{
    bonus_pool_label, column_total, lower_subtotal, upper_bonus, upper_subtotal, ROW_LABELS,
    ROW_SIXZEE,
};

// ─── Component ───────────────────────────────────────────────────────────────

/// 6-column × 13-row Sixzee scorecard with live score previews.
///
/// `on_cell_click` receives `(col, row)` for every click on an open cell.
/// Filled cells are inert.
#[component]
pub fn Scorecard(
    /// Called with `(col, row)` when the player taps an open cell.
    on_cell_click: Callback<(usize, usize)>,
) -> impl IntoView {
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");
    let score_preview =
        use_context::<Memo<[[u8; 13]; 6]>>().expect("score_preview context must be provided");
    let grand_total_memo =
        use_context::<Memo<u32>>().expect("grand_total context must be provided");

    view! {
        <div class="scorecard-wrapper">
            <table class="scorecard">
                <thead>
                    <tr>
                        <th></th>
                        <th>"C1"</th>
                        <th>"C2"</th>
                        <th>"C3"</th>
                        <th>"C4"</th>
                        <th>"C5"</th>
                        <th>"C6"</th>
                    </tr>
                </thead>
                <tbody>
                    // ── Upper section rows 0–5 ────────────────────────────
                    {(0..6_usize)
                        .map(|row| {
                            view! {
                                <tr>
                                    <td>{ROW_LABELS[row]}</td>
                                    {(0..6_usize)
                                        .map(|col| {
                                            cell_view(
                                                game_signal,
                                                score_preview,
                                                col,
                                                row,
                                                on_cell_click,
                                            )
                                        })
                                        .collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                    // ── Upper sub-totals ──────────────────────────────────
                    <tr class="scorecard__row--separator">
                        <td>"Upper Sub"</td>
                        {(0..6_usize)
                            .map(|col| {
                                view! {
                                    <td class="scorecard__cell">
                                        {move || {
                                            let s = game_signal.get();
                                            upper_subtotal(&s.cells[col]).to_string()
                                        }}
                                    </td>
                                }
                            })
                            .collect_view()}
                    </tr>
                    // ── Upper bonus row ───────────────────────────────────
                    <tr class="scorecard__row--separator">
                        <td>"Bonus (+35≥63)"</td>
                        {(0..6_usize)
                            .map(|col| {
                                view! {
                                    <td class="scorecard__cell">
                                        {move || {
                                            let s = game_signal.get();
                                            let b = upper_bonus(&s.cells[col]);
                                            if b > 0 { format!("+{b}") } else { String::new() }
                                        }}
                                    </td>
                                }
                            })
                            .collect_view()}
                    </tr>
                    // ── Lower section rows 6–12 ───────────────────────────
                    {(6..13_usize)
                        .map(|row| {
                            view! {
                                <tr>
                                    <td>{ROW_LABELS[row]}</td>
                                    {(0..6_usize)
                                        .map(|col| {
                                            cell_view(
                                                game_signal,
                                                score_preview,
                                                col,
                                                row,
                                                on_cell_click,
                                            )
                                        })
                                        .collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                    // ── Lower sub-totals ──────────────────────────────────
                    <tr class="scorecard__row--separator">
                        <td>"Lower Sub"</td>
                        {(0..6_usize)
                            .map(|col| {
                                view! {
                                    <td class="scorecard__cell">
                                        {move || {
                                            let s = game_signal.get();
                                            lower_subtotal(&s.cells[col]).to_string()
                                        }}
                                    </td>
                                }
                            })
                            .collect_view()}
                    </tr>
                    // ── Column totals ─────────────────────────────────────
                    <tr class="scorecard__row--separator">
                        <td>"Col Total"</td>
                        {(0..6_usize)
                            .map(|col| {
                                view! {
                                    <td class="scorecard__cell">
                                        {move || {
                                            let s = game_signal.get();
                                            column_total(&s.cells[col]).to_string()
                                        }}
                                    </td>
                                }
                            })
                            .collect_view()}
                    </tr>
                </tbody>
            </table>
        </div>
        // ── Footer: Bonus pool + Grand total ──────────────────────────────
        <div class="scorecard-footer">
            <div class="bonus-pool">
                <div class="bonus-pool__label">"SIXZEE BONUS POOL"</div>
                <div class="bonus-pool__value">
                    {move || {
                        let s = game_signal.get();
                        format!("+{}", s.bonus_pool)
                    }}
                </div>
                {move || {
                    let s = game_signal.get();
                    if s.bonus_forfeited {
                        view! { <div class="bonus-pool__forfeited">"FORFEITED"</div> }.into_any()
                    } else {
                        let filled = s.cells.iter().filter(|col| col[ROW_SIXZEE].is_some()).count();
                        view! {
                            <div class="bonus-pool__forfeited">
                                {bonus_pool_label(filled)}
                            </div>
                        }
                        .into_any()
                    }
                }}
            </div>
            <div class="grand-total">
                <div class="grand-total__label">"GRAND TOTAL"</div>
                <div>{move || grand_total_memo.get().to_string()}</div>
            </div>
        </div>
    }
}

// ─── Cell rendering helper ───────────────────────────────────────────────────

/// Render one `<td>` cell for `(col, row)`.
fn cell_view(
    game_signal: RwSignal<GameState>,
    score_preview: Memo<[[u8; 13]; 6]>,
    col: usize,
    row: usize,
    on_cell_click: Callback<(usize, usize)>,
) -> impl IntoView {
    view! {
        {move || {
            let state = game_signal.get();
            let cell = state.cells[col][row];
            let rolls_used = state.rolls_used;

            if let Some(v) = cell {
                // Filled cell — display value, not clickable.
                return view! {
                    <td class="scorecard__cell scorecard__cell--filled">{v.to_string()}</td>
                }
                .into_any();
            }

            if rolls_used == 0 {
                // Not yet rolled — open cell, no preview, not clickable.
                return view! {
                    <td class="scorecard__cell scorecard__cell--open"></td>
                }
                .into_any();
            }

            // Rolled — show preview.
            let preview = score_preview.get()[col][row];
            if preview > 0 {
                view! {
                    <td
                        class="scorecard__cell scorecard__cell--preview"
                        on:click=move |_| on_cell_click.run((col, row))
                    >
                        {format!("[{preview}]")}
                    </td>
                }
                .into_any()
            } else {
                view! {
                    <td
                        class="scorecard__cell scorecard__cell--zero-preview"
                        on:click=move |_| on_cell_click.run((col, row))
                    >
                        "[0]"
                    </td>
                }
                .into_any()
            }
        }}
    }
}
