//! Ask Grandma advice panel overlay.
//!
//! Rendered unconditionally by `App`; visible only when `grandma_panel_state`
//! is not `GrandmaPanelState::Closed`.
//!
//! ## Interaction model
//!
//! - **Loading:** spinner/pulse shown while the worker computes.
//! - **Ready:** 5 action cards with Apply buttons.
//! - **Error:** inline error message with a Retry button.
//! - **✕ close:** resets state to `Closed` without changing game state.
//!
//! Apply actions route through the same handlers as normal gameplay so all
//! invariants (zero-confirm, persist) are respected.

use leptos::prelude::*;

use crate::error::report_error;
use crate::state::game::{completed_game_from_state, is_game_complete, place_score};
use crate::state::storage;
use crate::state::{HideTabBar};
use crate::worker::messages::{ActionKind, GrandmaRequest};
use crate::worker::{post_grandma_request, GrandmaPanelState};

/// Ask Grandma overlay panel.
///
/// Reads `grandma_panel_state` and `grandma_worker` from context.
/// Writes back to `game_signal` when the player applies a Score-now action.
#[component]
pub fn GrandmaPanel() -> impl IntoView {
    let panel_state =
        use_context::<RwSignal<GrandmaPanelState>>().expect("grandma_panel_state in context");
    let grandma_worker =
        use_context::<RwSignal<Option<web_sys::Worker>>>().expect("grandma_worker in context");
    let game_signal =
        use_context::<RwSignal<crate::state::game::GameState>>().expect("game_signal in context");
    let hide_tab_bar =
        use_context::<HideTabBar>().expect("hide_tab_bar in context").0;

    // ── Close handler ─────────────────────────────────────────────────────────
    let on_close = move |_| {
        panel_state.set(GrandmaPanelState::Closed);
    };

    // ── Retry: re-post the current game state to the worker ───────────────────
    let on_retry = move |_| {
        let state = game_signal.get_untracked();
        if let Some(worker) = grandma_worker.get_untracked() {
            let dice = match crate::state::game::current_dice(&state) {
                Some(d) => d,
                None => return,
            };
            let req = GrandmaRequest {
                cells: state.cells,
                dice,
                held: state.held,
                rolls_used: state.rolls_used,
                bonus_pool: state.bonus_pool,
                bonus_forfeited: state.bonus_forfeited,
            };
            panel_state.set(GrandmaPanelState::Loading);
            if let Err(e) = post_grandma_request(&worker, &req) {
                report_error(e);
                panel_state.set(GrandmaPanelState::Error(
                    "Could not reach Grandma — please try again".to_string(),
                ));
            }
        }
    };

    view! {
        {move || {
            let state = panel_state.get();
            if matches!(state, GrandmaPanelState::Closed) {
                return view! { <span /> }.into_any();
            }

            view! {
                <div class="overlay overlay--grandma" aria-modal="true" role="dialog">
                    <div class="overlay--grandma__panel">

                        // ── Header ─────────────────────────────────────────────
                        <div class="overlay--grandma__header">
                            <span class="overlay--grandma__title">
                                "👵 GRANDMA'S ADVICE — Top 5 Moves"
                            </span>
                            <button
                                class="overlay--grandma__close btn btn--ghost"
                                on:click=on_close
                                aria-label="Close Grandma's Advice"
                            >
                                "✕"
                            </button>
                        </div>

                        // ── Body ───────────────────────────────────────────────
                        {move || match panel_state.get() {
                            GrandmaPanelState::Loading => view! {
                                <div class="grandma-loading">
                                    <span class="grandma-spinner" />
                                    " Thinking…"
                                </div>
                            }.into_any(),

                            GrandmaPanelState::Error(msg) => view! {
                                <div class="grandma-error">
                                    <p class="grandma-error__msg">{msg}</p>
                                    <button class="btn btn--secondary" on:click=on_retry>
                                        "↻ Retry"
                                    </button>
                                </div>
                            }.into_any(),

                            GrandmaPanelState::Ready(actions) => {
                                let cards: Vec<_> = actions
                                    .into_iter()
                                    .enumerate()
                                    .map(|(idx, action)| {
                                        let rank = idx + 1;
                                        let description = action.description.clone();
                                        let detail = action.detail.clone();
                                        let est = action.est_final_score;
                                        let kind = action.kind.clone();
                                        let p_state = panel_state;
                                        let g_signal = game_signal;
                                        let htb = hide_tab_bar;

                                        let on_apply = move |_| {
                                            apply_action(&kind, p_state, g_signal, htb);
                                        };

                                        view! {
                                            <div class="grandma-card">
                                                <div class="grandma-card__rank">
                                                    {format!("#{rank}")}
                                                </div>
                                                <div class="grandma-card__description">
                                                    {description}
                                                </div>
                                                <div class="grandma-card__detail">
                                                    {detail}
                                                </div>
                                                <div class="grandma-card__score">
                                                    "Est. final: "
                                                    {format!("{:.0}", est)}
                                                </div>
                                                <button
                                                    class="grandma-card__apply btn btn--secondary"
                                                    on:click=on_apply
                                                >
                                                    "Apply"
                                                </button>
                                            </div>
                                        }
                                    })
                                    .collect();

                                view! {
                                    <div class="grandma-cards">
                                        {cards}
                                    </div>
                                    <p class="grandma-footer">
                                        "Based on DP value table + sampling"
                                    </p>
                                }.into_any()
                            }

                            GrandmaPanelState::Closed => view! { <span /> }.into_any(),
                        }}
                    </div>
                </div>
            }.into_any()
        }}
    }
}

// ── Apply action helper ────────────────────────────────────────────────────────

/// Apply a recommended action from the grandma panel.
///
/// Reroll: updates `held` mask, closes the panel.
/// Score-now with nonzero: places the score and persists. Closes panel.
/// Score-now with zero: closes panel without placing — the user must confirm
/// by clicking the cell normally (which shows the zero-confirm overlay).
fn apply_action(
    kind: &ActionKind,
    panel_state: RwSignal<GrandmaPanelState>,
    game_signal: RwSignal<crate::state::game::GameState>,
    hide_tab_bar: RwSignal<bool>,
) {
    match kind {
        ActionKind::Reroll { hold_mask } => {
            let mask = *hold_mask;
            game_signal.update(|s| {
                s.held = mask;
            });
            panel_state.set(GrandmaPanelState::Closed);
        }
        ActionKind::Score { col, row, .. } => {
            let (c, r) = (*col, *row);
            let state = game_signal.get_untracked();

            // Require all dice to be present before scoring.
            let all_rolled = state.dice.iter().all(|d| d.is_some());
            if !all_rolled {
                panel_state.set(GrandmaPanelState::Closed);
                return;
            }

            let dice = [
                state.dice[0].expect("dice Some"),
                state.dice[1].expect("dice Some"),
                state.dice[2].expect("dice Some"),
                state.dice[3].expect("dice Some"),
                state.dice[4].expect("dice Some"),
            ];
            let preview_score = crate::state::scoring::score_for_row(r, dice);

            panel_state.set(GrandmaPanelState::Closed);

            if preview_score == 0 {
                // Score == 0: close and let the user click the cell to trigger
                // the normal zero-confirm flow.  Do not auto-place.
                hide_tab_bar.set(false);
            } else {
                game_signal.update(|s| {
                    let _ = place_score(s, c, r);
                });
                persist_after_score(game_signal);
            }
        }
    }
}

/// Persist game state (in-progress or completed) after a score placement.
fn persist_after_score(game_signal: RwSignal<crate::state::game::GameState>) {
    let new_state = game_signal.get_untracked();
    if is_game_complete(&new_state) {
        let cg = completed_game_from_state(&new_state);
        let mut history = match storage::load_history() {
            Ok(h) => h,
            Err(e) => {
                report_error(e);
                vec![]
            }
        };
        history.push(cg);
        if let Err(e) = storage::save_history(&history) {
            report_error(e);
        }
        if let Err(e) = storage::clear_in_progress() {
            report_error(e);
        }
    } else if let Err(e) = storage::save_in_progress(&new_state) {
        report_error(e);
    }
}
