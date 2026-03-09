//! Full game screen — wires dice, scorecard, roll/score controls, and overlays.
//!
//! Reads all game-related signals from context and manages the local overlay
//! state: zero-score confirmation, quit confirmation, and the Sixzee inline quote.
//! When `GameActive` context is `false`, renders the `IdleScreen` instead.

use leptos::prelude::*;

use crate::components::confirm_quit::ConfirmQuit;
use crate::components::confirm_zero::ConfirmZero;
use crate::components::dice_row::DiceRow;
use crate::components::end_game::EndGame;
use crate::components::game_menu::GameMenu;
use crate::components::grandma_quote::GrandmaQuoteInline;
use crate::components::idle_screen::IdleScreen;
use crate::components::scorecard::Scorecard;
use crate::error::report_error;
use crate::state::game::{
    completed_game_from_state, is_game_complete, new_game, place_score, roll, GameState,
};
use crate::state::quotes::{pick_quote, QuoteBank};
use crate::state::scoring::score_sixzee;
use crate::state::storage;
use crate::state::{GameActive, HideTabBar, ShowOpeningQuote};
use crate::worker::messages::GrandmaRequest;
use crate::worker::{post_grandma_request, GrandmaPanelState};

/// Called after every `place_score` invocation to handle localStorage persistence.
///
/// - If the game is now complete: builds a `CompletedGame` record, appends it
///   to history (sorted), and clears the in-progress save.
/// - Otherwise: overwrites the in-progress save with the current state.
///
/// Storage errors are reported as `Degraded` (non-blocking banners); game state
/// in memory is always intact regardless of storage failures.
fn persist_after_score(state: &GameState) {
    if is_game_complete(state) {
        let cg = completed_game_from_state(state);
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
    } else if let Err(e) = storage::save_in_progress(state) {
        report_error(e);
    }
}

/// The full game screen component.
///
/// When `GameActive` context signal is `false`, renders `IdleScreen`. When
/// `true`, renders the game header, dice row, Roll/Ask Grandma buttons,
/// optional Sixzee inline quote, scorecard, and any active overlays
/// (confirm_quit, confirm_zero, end_game).
#[component]
pub fn GameView() -> impl IntoView {
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");
    let quote_bank =
        use_context::<RwSignal<Option<QuoteBank>>>().expect("quote_bank context must be provided");
    let hide_tab_bar =
        use_context::<HideTabBar>().expect("hide_tab_bar context must be provided").0;
    let show_opening_quote =
        use_context::<ShowOpeningQuote>().expect("show_opening_quote context must be provided").0;
    let game_active =
        use_context::<GameActive>().expect("game_active context must be provided").0;

    // ── M7: Ask Grandma ──────────────────────────────────────────────────────
    let grandma_worker =
        use_context::<RwSignal<Option<web_sys::Worker>>>().expect("grandma_worker in context");
    let grandma_panel_state =
        use_context::<RwSignal<GrandmaPanelState>>().expect("grandma_panel_state in context");

    // ── Local overlay state ──────────────────────────────────────────────────

    // `Some((col, row))` when the zero-score confirmation prompt is open.
    let pending_zero: RwSignal<Option<(usize, usize)>> = RwSignal::new(None);

    // Set to a picked Sixzee quote when the current dice all match.
    // Cleared on the next roll or score placement.
    let sixzee_inline_quote: RwSignal<Option<String>> = RwSignal::new(None);

    // `true` while the quit-confirmation overlay is open.
    let show_confirm_quit: RwSignal<bool> = RwSignal::new(false);

    // ── Roll handler ─────────────────────────────────────────────────────────

    let on_roll = move |_| {
        game_signal.update(|state| {
            // Guard: must be < 3 rolls and game must not be complete.
            if state.rolls_used >= 3 || is_game_complete(state) {
                return;
            }
            let _ = roll(state);
        });

        // Persist after roll (Degraded on failure — roll already happened).
        let state = game_signal.get_untracked();
        if state.rolls_used > 0 {
            // rolls_used == 0 means a bonus Sixzee fired and start_turn was called;
            // the dice are reset but we still save so the bonus_pool is persisted.
            if let Err(e) = storage::save_in_progress(&state) {
                report_error(e);
            }
        } else {
            // Bonus Sixzee fired — dice cleared, save the post-bonus state.
            if let Err(e) = storage::save_in_progress(&state) {
                report_error(e);
            }
        }

        // Clear any previous Sixzee inline quote, then check for a new one.
        sixzee_inline_quote.set(None);

        let state = game_signal.get_untracked();
        // If detect_bonus_sixzee fired inside roll(), dice are all None and
        // rolls_used == 0 — skip the inline quote check.
        if state.rolls_used > 0 {
            let all_rolled = state.dice.iter().all(|d| d.is_some());
            if all_rolled {
                let dice = [
                    state.dice[0].expect("dice guaranteed Some when rolls_used > 0"),
                    state.dice[1].expect("dice guaranteed Some when rolls_used > 0"),
                    state.dice[2].expect("dice guaranteed Some when rolls_used > 0"),
                    state.dice[3].expect("dice guaranteed Some when rolls_used > 0"),
                    state.dice[4].expect("dice guaranteed Some when rolls_used > 0"),
                ];
                if score_sixzee(dice) == 50 {
                    let q = quote_bank
                        .get_untracked()
                        .as_ref()
                        .and_then(|b| pick_quote(&b.sixzee))
                        .map(str::to_owned);
                    sixzee_inline_quote.set(q);
                }
            }
        }
    };

    // ── Cell click handler ────────────────────────────────────────────────────

    let on_cell_click = Callback::new(move |(col, row): (usize, usize)| {
        let state = game_signal.get_untracked();

        // Guard: must have rolled at least once and cell must be open.
        if state.rolls_used == 0 || state.cells[col][row].is_some() {
            return;
        }

        let all_rolled = state.dice.iter().all(|d| d.is_some());
        if !all_rolled {
            return;
        }
        let dice = [
            state.dice[0].expect("dice guaranteed Some when rolls_used > 0"),
            state.dice[1].expect("dice guaranteed Some when rolls_used > 0"),
            state.dice[2].expect("dice guaranteed Some when rolls_used > 0"),
            state.dice[3].expect("dice guaranteed Some when rolls_used > 0"),
            state.dice[4].expect("dice guaranteed Some when rolls_used > 0"),
        ];
        let preview_score = crate::state::scoring::score_for_row(row, dice);

        if preview_score > 0 {
            sixzee_inline_quote.set(None);
            game_signal.update(|s| {
                let _ = place_score(s, col, row);
            });
            persist_after_score(&game_signal.get_untracked());
        } else {
            hide_tab_bar.set(true);
            pending_zero.set(Some((col, row)));
        }
    });

    // ── Confirm-zero callbacks ────────────────────────────────────────────────

    let on_cancel_zero = Callback::new(move |_| {
        hide_tab_bar.set(false);
        pending_zero.set(None);
    });

    let on_confirm_zero = Callback::new(move |_| {
        hide_tab_bar.set(false);
        if let Some((col, row)) = pending_zero.get_untracked() {
            sixzee_inline_quote.set(None);
            game_signal.update(|s| {
                let _ = place_score(s, col, row);
            });
            persist_after_score(&game_signal.get_untracked());
        }
        pending_zero.set(None);
    });

    // ── New Game handler ──────────────────────────────────────────────────────

    let on_new_game = Callback::new(move |_| {
        game_signal.set(new_game());
        sixzee_inline_quote.set(None);
        pending_zero.set(None);
        show_opening_quote.set(true);
    });

    // ── Quit game handlers ────────────────────────────────────────────────────

    let on_quit_requested = Callback::new(move |_| {
        show_confirm_quit.set(true);
    });

    let on_cancel_quit = Callback::new(move |_| {
        show_confirm_quit.set(false);
    });

    let on_confirm_quit = Callback::new(move |_| {
        show_confirm_quit.set(false);
        pending_zero.set(None);
        sixzee_inline_quote.set(None);
        if let Err(e) = storage::clear_in_progress() {
            report_error(e);
        }
        game_active.set(false);
    });

    // ── Idle-screen "Start New Game" handler ──────────────────────────────────

    let on_start_game = Callback::new(move |_| {
        game_signal.set(new_game());
        pending_zero.set(None);
        sixzee_inline_quote.set(None);
        game_active.set(true);
        show_opening_quote.set(true);
    });

    // ── Ask Grandma handler ───────────────────────────────────────────────────

    let on_ask_grandma = move |_| {
        let state = game_signal.get_untracked();

        // Require at least one roll before asking.
        if state.rolls_used == 0 {
            return;
        }
        let all_rolled = state.dice.iter().all(|d| d.is_some());
        if !all_rolled {
            return;
        }
        let dice = [
            state.dice[0].expect("dice Some after roll"),
            state.dice[1].expect("dice Some after roll"),
            state.dice[2].expect("dice Some after roll"),
            state.dice[3].expect("dice Some after roll"),
            state.dice[4].expect("dice Some after roll"),
        ];

        let req = GrandmaRequest {
            cells: state.cells,
            dice,
            held: state.held,
            rolls_used: state.rolls_used,
            bonus_pool: state.bonus_pool,
            bonus_forfeited: state.bonus_forfeited,
        };

        if let Some(worker) = grandma_worker.get_untracked() {
            grandma_panel_state.set(GrandmaPanelState::Loading);
            if let Err(e) = post_grandma_request(&worker, &req) {
                report_error(e);
                grandma_panel_state.set(GrandmaPanelState::Error(
                    "Could not reach Grandma — please try again".to_string(),
                ));
            }
        }
    };

    view! {
        // ── Idle screen (shown when no active game) ──────────────────────────
        {move || {
            if !game_active.get() {
                return view! { <IdleScreen on_start_game=on_start_game /> }.into_any();
            }

            // ── Active overlays ──────────────────────────────────────────────
            view! {
                {move || {
                    let state = game_signal.get();
                    if is_game_complete(&state) {
                        return view! { <EndGame on_new_game=on_new_game /> }.into_any();
                    }
                    view! { <span /> }.into_any()
                }}

                {move || {
                    match pending_zero.get() {
                        Some((col, row)) => view! {
                            <ConfirmZero
                                col=col
                                row=row
                                on_cancel=on_cancel_zero
                                on_confirm=on_confirm_zero
                            />
                        }
                        .into_any(),
                        None => view! { <span /> }.into_any(),
                    }
                }}

                {move || {
                    if show_confirm_quit.get() {
                        return view! {
                            <ConfirmQuit
                                on_confirm=on_confirm_quit
                                on_cancel=on_cancel_quit
                            />
                        }
                        .into_any();
                    }
                    view! { <span /> }.into_any()
                }}

                // ── Game screen ──────────────────────────────────────────────
                <header class="game-header">
                    <span class="game-header__title">"SIXZEE"</span>
                    <div class="game-header__turn-info">
                        {move || {
                            let s = game_signal.get();
                            let turn_num = s.turn + 1;
                            let remaining = 3u8.saturating_sub(s.rolls_used);
                            let pips: String =
                                "●".repeat(remaining as usize) + &"○".repeat(s.rolls_used as usize);
                            format!("Turn {turn_num}  {pips} {remaining} rolls")
                        }}
                    </div>
                    <GameMenu on_quit_requested=on_quit_requested />
                </header>

                <DiceRow />

                <div class="action-buttons">
                    <button
                        class="btn btn--primary"
                        on:click=on_roll
                        disabled=move || {
                            let s = game_signal.get();
                            s.rolls_used >= 3 || is_game_complete(&s)
                        }
                    >
                        "🎲  ROLL"
                    </button>
                    <button
                        class="btn btn--secondary"
                        on:click=on_ask_grandma
                        disabled=move || {
                            let s = game_signal.get();
                            s.rolls_used == 0
                                || is_game_complete(&s)
                                || grandma_worker.get().is_none()
                        }
                        title="Ask Grandma for advice on the best move"
                    >
                        "👵 ASK GRANDMA"
                    </button>
                </div>

                {move || {
                    match sixzee_inline_quote.get() {
                        Some(q) => view! { <GrandmaQuoteInline quote=q /> }.into_any(),
                        None => view! { <span /> }.into_any(),
                    }
                }}

                <Scorecard on_cell_click=on_cell_click />
            }
            .into_any()
        }}
    }
}
