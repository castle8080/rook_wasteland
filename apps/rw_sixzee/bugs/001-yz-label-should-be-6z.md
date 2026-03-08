# Bug 001 — "Yz" label should be "6z"

## Status
Fixed

## Summary
The Sixzee bonus pool indicator displayed `(0 of 6 Yz filled)` in the UI. The
abbreviation `Yz` is a leftover from the Yahtzee fork. The correct game-specific
term is `6z`.

## Steps to Reproduce
1. Launch the app.
2. Look at the bonus pool section of the scorecard.

## Expected Behaviour
Label reads: `(0 of 6 6z filled)`.

## Actual Behaviour
Label reads: `(0 of 6 Yz filled)`.

## Affected Locations
| File | Line | Snippet |
|---|---|---|
| `src/components/scorecard.rs` | 176 | `format!("({filled} of 6 Yz filled)")` |
| `doc/wireframes.md` | 90 | `(3 of 6 Yz filled ✓)` |

---

## Root Cause

The format string `"({filled} of 6 Yz filled)"` was written during the initial fork
from a Yahtzee prototype and was never updated to the game's own terminology. The
string was inline inside the Leptos component view macro in
`src/components/scorecard.rs:176`, making it invisible to any existing tests (all
tests live in pure-Rust non-wasm modules; the `components` module is gated behind
`#[cfg(target_arch = "wasm32")]`). The same artifact appeared in the wireframe
example in `doc/wireframes.md:90`. It wasn't caught earlier because there was no
test asserting the content of the label and no text-search pass at rename time.

## Fix

**`src/state/scoring.rs`** — Added a public `bonus_pool_label(filled: usize) ->
String` helper in a new `// ─── Display helpers ───` section, returning
`"({filled} of 6 6z filled)"`.

**`src/components/scorecard.rs`** — Replaced the inline `format!` with a call to
`bonus_pool_label(filled)` and updated the import line.

**`doc/wireframes.md`** — Updated the wireframe example from `Yz` to `6z`.

Before:
```rust
{format!("({filled} of 6 Yz filled)")}
```
After:
```rust
{bonus_pool_label(filled)}
// in scoring.rs:
pub fn bonus_pool_label(filled: usize) -> String {
    format!("({filled} of 6 6z filled)")
}
```

## Regression Test

**Tests added** (in `src/state/scoring.rs`, `mod tests`):

| Test name | Scenario |
|---|---|
| `bonus_pool_label_uses_6z_abbreviation` | Asserts the label contains `"6z"` and does **not** contain `"Yz"` |
| `bonus_pool_label_formats_count_correctly` | Asserts exact strings for `filled=0` and `filled=6` |

Moving the format logic to `scoring.rs` (a non-wasm module) makes it reachable by
`cargo test` on native targets — the previous inline `format!` inside the wasm-gated
component was untestable natively.

## Post-Mortem / Lessons Learned

### Test coverage gaps in wasm-gated components
Display strings, labels, and format strings embedded inside Leptos component view
macros are invisible to `cargo test` because the entire `components` module is gated
behind `#[cfg(target_arch = "wasm32")]`. This means any label wording can silently
hold stale copy indefinitely. The fix: extract any non-trivial string formatting
into small helper functions in a non-gated module (e.g. `state/scoring.rs`) so they
can be covered by ordinary unit tests.

### No new broadly-applicable lessons
The `display helpers in non-wasm modules` pattern is project-specific enough to not
warrant a new `doc/lessons.md` entry (the module-gating behaviour is already
documented in `doc/lessons.md` L4). No new addition needed.
