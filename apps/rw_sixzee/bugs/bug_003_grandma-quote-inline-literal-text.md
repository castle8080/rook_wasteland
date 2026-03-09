# Bug 003 — GrandmaQuoteInline Renders Literal `{quote}` Instead of Quote Text

## Status
Fixed

## Summary
The `GrandmaQuoteInline` component displays the literal text `{quote}` instead of the actual
quote string. This is caused by a Rust string-literal tokenization error in the `view!` macro:
the `{quote}` variable reference is accidentally enclosed inside a string literal rather than
being a standalone dynamic expression block. Because `GrandmaQuoteInline` is the shared renderer
for all four in-game quote moments, **all four quote scenarios are broken** — opening overlay,
closing end-game quote, Sixzee inline quote, and the scratch prompt in zero-score confirmation.

## Steps to Reproduce
1. Run `python make.py build` (debug mode, trunk build / localhost)
2. Open the app in a browser; a new-game state triggers the opening quote overlay
3. Observe the grandma quote card inside the overlay

**Observed result:** The card shows `👵  {quote}` — a double-space after the emoji and the
literal string `{quote}` where the actual quote text should appear.

## Expected Behavior
The card should display an actual quote from `assets/grandma_quotes.json` (e.g.
`👵 "You only need one die to learn something." — Grandma`).

## Actual Behavior
The card shows `👵  {quote} — Grandma` with the literal characters `{`, `q`, `u`, `o`, `t`,
`e`, `}` rendered as DOM text.

## Environment / Context
- **App:** rw_sixzee
- **Build mode:** Debug (trunk build / localhost)
- **Browser:** Not specified
- **Recent changes:** Milestone 7 (Ask Grandma / Worker) completed — quote rendering code predates
  M7 but was not exercised by M7 tests
- **Error output:** None — the DOM renders without console errors; the wrong text just appears

## Initial Triage

The bug is a Rust tokenization error on line 77 of `src/components/grandma_quote.rs`.
The offending line inside the Leptos `view!` macro is:

```rust
"👵 "" {quote} """
```

All six `"` characters are ASCII U+0022. The Rust tokenizer therefore parses this as:

| Token | Content | Notes |
|---|---|---|
| String literal | `"👵 "` | static text `👵 ` |
| String literal | `" {quote} "` | **`{quote}` is inside this string — literal text, not evaluated** |
| String literal | `""` | empty static text |

The `{quote}` block is never seen by the Leptos view! macro as a dynamic child expression; it is
an inert subsequence of bytes inside the second string literal. The runtime output is
`👵  {quote} ` (two spaces, then the literal braces and letters, then a trailing space).

The developer's intent was likely to wrap the quote in typographic curly-quote characters
(`"` U+201C and `"` U+201D). The fix is to move `{quote}` outside all string literals and use
Unicode escapes for the decorative quotes:

```rust
"👵 \u{201C}" {quote} "\u{201D}"
```

**All four Grandma quote moments are affected** because they all delegate display to
`GrandmaQuoteInline`:
- **Opening** — `GrandmaQuoteOverlay` (via `app.rs` Effect, shown on new game)
- **Closing** — `EndGame` (`src/components/end_game.rs`)
- **Sixzee** — `GameView` (`src/components/game_view.rs`)
- **Scratch** — `ConfirmZero` (`src/components/confirm_zero.rs`)

No similar pattern appears in `doc/lessons.md` — this is the first Leptos view! string-literal
collision recorded for this project. Related: Bug 002 documented a different view! macro pitfall
(untracked signal in a non-closure attribute).

**PRD coverage audit (quote scenarios):** All four PRD-specified quote moments (req 35–38) are
architecturally wired in code. No quote scenario from the PRD appears to be missing from the
implementation — the single broken component (`GrandmaQuoteInline`) is the root cause for all
four failing. No additional missing quote locations were identified.

---
<!-- The sections below are filled in during the fix phase -->

## Root Cause

The Rust tokenizer runs before the Leptos `view!` macro processes source. In
`src/components/grandma_quote.rs` line 80 (original), the expression was:

```rust
"👵 "" {quote} """
```

All six `"` characters are ASCII U+0022. The Rust tokenizer parsed this as three
adjacent string literals: `"👵 "`, `" {quote} "`, and `""`. The `{quote}` variable
was entirely inside the second string literal — the view! macro never saw it as a
dynamic expression block. No compile error or runtime warning was emitted; the DOM
simply rendered the literal characters `{quote}`.

## Fix

Changed line 80 of `src/components/grandma_quote.rs` to move `{quote}` outside all
string literals, using `\u{201C}` / `\u{201D}` Unicode escapes for the typographic
curly-quotes:

```rust
// Before (broken)
"👵 "" {quote} """

// After (fixed)
"👵 \u{201C}" {quote} "\u{201D}"
```

The `{quote}` token is now a standalone dynamic expression block that Leptos evaluates
at render time.

## Regression Test

Added `grandma_quote_inline_renders_quote_text` in `tests/integration.rs` (line 1177).
The test mounts `GrandmaQuoteInline` with a known string, then asserts:
1. The `.grandma-quote__text` span contains the verbatim quote string.
2. The rendered text does **not** contain the literal characters `{quote}`.

All 37 WASM integration tests pass (including the new regression test).

## Post-Mortem / Lessons Learned

**Lesson added:** `doc/lessons.md` L19 — *Leptos `view!` string literal tokenization:
`{expr}` inside a string literal is never evaluated.*

The key insight is that the Rust tokenizer runs before the Leptos macro, so a `{…}`
block enclosed in ASCII `"` delimiters is inert string content. The only symptom is
literal text in the browser — no compiler warning, no runtime warning. The safe
pattern is to always place dynamic expression blocks outside string literals and use
`\u{…}` escapes for any Unicode punctuation that shares visual similarity with Rust
string delimiters.
