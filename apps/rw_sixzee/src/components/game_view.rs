//! Full game screen — wires dice, scorecard, roll/score controls, and overlays.
//!
//! Reads all game-related signals from context and manages the local overlay
//! state: zero-score confirmation and the Sixzee inline quote.

use leptos::prelude::*;

use crate::components::confirm_zero::ConfirmZero;
use crate::components::dice_row::DiceRow;
use crate::components::end_game::EndGame;
use crate::components::grandma_quote::GrandmaQuoteInline;
use crate::components::scorecard::Scorecard;
use crate::state::game::{is_game_complete, new_game, place_score, roll, GameState};
use crate::state::quotes::{pick_quote, QuoteBank};
use crate::state::scoring::score_sixzee;

/// The full game screen component.
///
/// Renders the game header, dice row, Roll/Ask Grandma buttons, optional Sixzee
/// inline quote, scorecard, and any active overlays (confirm_zero, end_game).
#[component]
pub fn GameView() -> impl IntoView {
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");
    let quote_bank =
        use_context::<RwSignal<Option<QuoteBank>>>().expect("quote_bank context must be provided");
    let hide_tab_bar =
        use_context::<RwSignal<bool>>().expect("hide_tab_bar context must be provided");
    let show_opening_quote =
        use_context::<RwSignal<bool>>().expect("show_opening_quote context must be provided");

    // ── Local overlay state ──────────────────────────────────────────────────

    // `Some((col, row))` when the zero-score confirmation prompt is open.
    let pending_zero: RwSignal<Option<(usize, usize)>> = RwSignal::new(None);

    // Set to a picked Sixzee quote when the current dice all match.
    // Cleared on the next roll or score placement.
    let sixzee_inline_quote: RwSignal<Option<String>> = RwSignal::new(None);

    // ── Roll handler ─────────────────────────────────────────────────────────

    let on_roll = move |_| {
        game_signal.update(|state| {
            // Guard: must be < 3 rolls and game must not be complete.
            if state.rolls_used >= 3 || is_game_complete(state) {
                return;
            }
            let _ = roll(state);
        });

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

    view! {
        // ── Active overlays ──────────────────────────────────────────────────
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

        // ── Game screen ──────────────────────────────────────────────────────
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
                disabled=move || {
                    let s = game_signal.get();
                    s.rolls_used == 0 || is_game_complete(&s)
                }
                title="Ask Grandma for advice — coming in a future update"
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
}
