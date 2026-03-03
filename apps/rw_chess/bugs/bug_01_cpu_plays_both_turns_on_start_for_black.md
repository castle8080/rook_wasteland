# CPU Takes black and white turn on start.

**Status: FIXED**

When the player selects black on the initial screen and then starts the game; the CPU or computer makes both the first move for black and white. After that the player can play as black. The CPU should not be making a move for black, if the player used the form to choose black as their side. It appears the turn selection logic or something like that is correct.

The expected behavior is that white always moves first, so if the computer is white they move, and if the player is white the move first. Then the turn switches.

# Bug Fix Instructions

Quickly assess the bug with a very short analysis and update the bug report. Perform a deeper code analysis and determine the cause. Add unit tests to cover the issue. Fix the code. Then document in the bug report the cause of the bug, what the fix is, and how it was re-tested. Write a section on lessons learned if applicable for the bug. Also update the bug report with the status when completed. You can also use files under docs for reference to other expected behavior. I don't think there is anything that needs to be updated in the spec, but if it seems like the spec was wrong you can also update that. I think this is just a logic bug though.

---

# Analysis

## Quick Assessment

The engine was being triggered twice at game start when the player chose Black — once by a reactive `Effect` and once by an explicit function call in the same callback.

## Root Cause

In `src/ui/app.rs`, the `on_start` callback (and `on_rematch`) did two things at the end of its body:

1. Set `in_setup` to `false`, which caused the reactive `Effect::new(...)` (a few lines below) to re-evaluate. The effect checks `active_color != player_color` and `!is_setup`, which at that point both pass, so it calls `trigger_engine_move`.

2. **Explicitly** called `trigger_engine_move` again via:
   ```rust
   if config.player_color == Color::Black {
       trigger_engine_move(game.clone());
   }
   ```

Since `trigger_engine_move` uses `spawn_local` with a 150 ms delay, both spawned tasks queued simultaneously. The first task woke up, read `active_color = White`, played White's move, and set `active_color = Black`. The second task then woke up, read `active_color = Black` (the engine's freshly updated value), and played Black's move — stealing the player's turn.

The same double-trigger existed in `on_rematch`.

## Fix

Two changes were made in `src/ui/app.rs`:

1. **Removed duplicate `trigger_engine_move` calls** from both `on_start` and `on_rematch`. The reactive `Effect` already handles triggering the engine whenever it becomes the engine's turn (including the initial turn after `in_setup` becomes `false`).

2. **Added a defensive guard inside `trigger_engine_move`** (after the 150 ms async yield) that re-reads `active_color`, `player_color`, and `phase` at execution time and bails early if it is no longer the engine's turn. Because `trigger_engine_move` is async, game state can change in the gap between the call site and when the task actually runs. If the guard trips, `leptos::logging::error!` emits a console error to make the unexpected condition visible during development:

   ```rust
   if !engine_should_move(active, player, phase) {
       leptos::logging::error!(
           "[trigger_engine_move] Unexpected: called when it is not the engine's turn \
            (active={active:?}, player={player:?}, phase={phase:?}). Bailing."
       );
       return;
   }
   ```

Additionally, extracted a pure helper function `engine_should_move(active, player, phase) -> bool` in `src/state/game.rs` to make the turn-guard logic testable in isolation and reusable by both the Effect and the guard inside `trigger_engine_move`.

## Tests Added (`src/state/game.rs`)

Seven new unit tests were added under `state::game::tests`:

| Test | Purpose |
|------|---------|
| `engine_moves_first_when_player_is_black` | Engine (White) must move when player chose Black at game start |
| `player_moves_first_when_player_is_white` | Engine must NOT move when player chose White |
| `engine_should_not_move_when_game_is_over` | Checkmate / Stalemate / DrawFiftyMove block engine trigger |
| `engine_moves_on_check_turn` | Engine still moves when active color is in check |
| `initial_active_color_is_white` | `GameState::new()` starts with White to move |
| `reset_restores_white_active_color` | `GameState::reset()` resets `active_color` to White |
| `after_reset_engine_should_move_when_player_is_black` | Regression — after reset the conditions that trigger the effect are correct for Black |
| `after_reset_engine_should_not_move_when_player_is_white` | Regression — after reset the conditions are correct for White |

All 42 tests pass (`cargo test`).

## Re-testing

- `cargo test` — all 42 tests pass, no warnings.
- Manual observation (code trace): `on_start` with `player_color = Black` now results in exactly one `spawn_local` call (from the Effect). White makes exactly one opening move; the board then waits for the human Black player.

---

# Lessons Learned

**Avoid mixing reactive effects and imperative triggers for the same action.** When a reactive framework (Leptos) already has a declarative effect watching state, explicitly calling the same side effect in response to the same state change creates a duplicate. The rule of thumb: if a reactive Effect owns a side-effect, let it be the only trigger. Imperative calls in event callbacks should be reserved for actions the Effect cannot observe (e.g., one-time fire-and-forget events on conditions the Effect doesn't track).

