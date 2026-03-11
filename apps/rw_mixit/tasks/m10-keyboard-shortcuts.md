# Task M10: Keyboard Shortcuts

**Milestone:** M10 — Keyboard Shortcuts
**Status:** 🔄 In Progress

## Restatement

This task wires a global `keydown`/`keyup` listener pair on `window` so the
app is fully playable without a mouse. Shortcuts must not fire when the user is
typing in a text input. The listener pair is registered inside `DeckView` where
both deck states and audio deck holders are already available, and is kept alive
for the app's entire lifetime via `std::mem::forget`. All actions mirror the
existing mouse-driven logic in `Controls`, `LoopControls`, and `HotCues`
components. Keyboard hot cue behavior: press a set cue → jump; press an unset
cue → set it at the current playhead position. Touch/mobile gesture support is
explicitly out of scope.

## Design

### Data flow

`window keydown` → `is_input_focused()` guard → `code` match → `do_*` helper
→ mutates `DeckState` signals and/or calls `AudioDeck` methods.

`window keyup` → `is_input_focused()` guard → `code` match for nudge keys →
`do_nudge_end` → `AudioDeck::nudge_end()`.

### Function / type signatures

```rust
/// Returns `true` when an INPUT or TEXTAREA element currently has focus.
pub fn is_input_focused() -> bool;

/// Container that keeps the global keydown + keyup listeners alive.
/// Drop = remove listeners; forget = keep forever.
pub struct KeyboardListeners { _keydown: EventListener, _keyup: EventListener }

/// Register shortcuts for both decks on `window` and return the listener handle.
pub fn register_keyboard_shortcuts(
    state_a: DeckState,
    audio_a: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    state_b: DeckState,
    audio_b: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
) -> KeyboardListeners;
```

Private helpers (not exported):
- `do_play_pause(state, holder)` — mirrors Controls logic
- `do_cue(state, holder)` — mirrors Controls logic
- `do_loop_in(state)` — mirrors LoopControls `on_loop_in`
- `do_loop_out(state)` — mirrors LoopControls `on_loop_out`
- `do_nudge_start(holder, direction)` — calls `AudioDeck::nudge_start`
- `do_nudge_end(holder)` — calls `AudioDeck::nudge_end`
- `do_hot_cue(state, holder, idx)` — jump if set, else set cue

### Edge cases

- `is_input_focused()` fails gracefully (returns `false`) when window / document
  / activeElement is unavailable (SSR or test harness).
- No audio deck loaded yet (holder is `None`) → all audio actions are no-ops.
- Nudge keydown auto-repeat: `ev.repeat() == true` → skip the `nudge_start` call
  so `pre_nudge_rate` is saved only once per physical key press.
- `Space` / `Enter` / `ArrowLeft` / `ArrowRight` / `BracketLeft` / `BracketRight`
  are `prevent_default`-ed when handled to avoid scroll / form submit.

### Integration points

- **New file**: `src/utils/keyboard.rs`
- **Modified**: `src/utils/mod.rs` — add `pub mod keyboard;`
- **Modified**: `src/components/deck.rs` (`DeckView`) — call
  `register_keyboard_shortcuts` and `std::mem::forget` the result

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Nudge auto-repeat would call `nudge_start` twice, overwriting `pre_nudge_rate` with the already-nudged rate | Guard with `ev.repeat()` check — skip `nudge_start` on repeats |
| Correctness | `Space` default scroll would fight with play/pause | Call `ev.prevent_default()` for handled keys |
| Simplicity | Hold-to-set hot cue (300 ms timer) via keyboard is complex and requires recursive closure patterns | Simpler rule: press unset cue → set; press set cue → jump. Fully functional without a timer |
| Coupling | `do_loop_out` duplicates clamping logic from `loop_controls.rs` | Extracted as a private helper `clamped_loop_out(loop_in, current)` mirroring the existing public `clamp_loop_out` in the same way; no cross-module coupling |
| Performance | Global keydown fires on every keystroke | Guard exits in O(1) via `is_input_focused()` and `ev.repeat()` — negligible cost |
| Testability | Most helpers touch `web-sys` types and can't run natively | Extract a pure `key_to_action(code)` function that maps strings to an enum; test natively |

## Implementation Notes

- Use `ev.code()` (physical key identity) not `ev.key()` (character) so shortcuts
  work regardless of keyboard layout.
- The keyup listener only cares about nudge keys; no need to share hot-cue timer
  state between keydown/keyup.
- Call `std::mem::forget` on the returned `KeyboardListeners` in `DeckView` — same
  pattern as the `hashchange` listener in `app.rs`.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `is_input_focused` returns false when no element focused | 1 | ✅ | via pure logic helper |
| `is_input_focused` returns true for INPUT tag | 1 | ✅ | pure helper |
| `is_input_focused` returns true for TEXTAREA tag | 1 | ✅ | pure helper |
| `key_to_action` maps Space → DeckA PlayPause | 1 | ✅ | |
| `key_to_action` maps Enter → DeckB PlayPause | 1 | ✅ | |
| `key_to_action` maps Q → DeckA Cue | 1 | ✅ | |
| `key_to_action` maps P → DeckB Cue | 1 | ✅ | |
| `key_to_action` maps Z/X → DeckA LoopIn/Out | 1 | ✅ | |
| `key_to_action` maps N/M → DeckB LoopIn/Out | 1 | ✅ | |
| `key_to_action` maps Arrow keys → DeckA Nudge | 1 | ✅ | |
| `key_to_action` maps Bracket keys → DeckB Nudge | 1 | ✅ | |
| `key_to_action` maps Digit1-4 → DeckA HotCue(0-3) | 1 | ✅ | |
| `key_to_action` maps Digit7-0 → DeckB HotCue(0-3) | 1 | ✅ | |
| `key_to_action` unknown key → None | 1 | ✅ | |
| Nudge end on keyup | 1 | ✅ | nudge action enum tested |
| Hot cue jump when set (AudioDeck.seek) | 3 | ❌ waived | requires live AudioContext; covered by controls manual testing |
| Hot cue set when unset | 3 | ❌ waived | requires live AudioContext; covered by controls manual testing |
| is_input_focused suppresses shortcuts | 3 | ❌ waived | requires DOM focus manipulation in headless browser |

## Test Results

(filled in Phase 10)

## Review Notes

(filled in Phase 7)

## Callouts / Gotchas

(filled in Phase 10)
