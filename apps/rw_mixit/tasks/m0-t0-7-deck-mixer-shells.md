# Task T0.7: Deck + Mixer Shell Components

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Create placeholder `<DeckView>` (3-column flex layout), `<DeckPlaceholder side="A"|"B">` (empty deck column with label), and `<Mixer>` (empty center column with label) components. These are stubs that establish the CSS class names and layout structure used by all later milestones. No audio, no signals, no canvas — purely structural HTML. Out of scope: any actual deck content.

---

## Design

### Data flow
No reactive data flow. Static view composition: `DeckView` renders `DeckPlaceholder("A")`, `Mixer`, `DeckPlaceholder("B")` in a flex row.

### Function / type signatures
```rust
#[component] pub fn DeckView() -> impl IntoView
#[component] pub fn DeckPlaceholder(side: &'static str) -> impl IntoView
#[component] pub fn Mixer() -> impl IntoView
```

### Edge cases
- `side` must be `"A"` or `"B"` — no validation needed at scaffold stage; the type narrows misuse.
- CSS class `deck-a` / `deck-b` must match the custom properties `--color-deck-a` / `--color-deck-b` in style.css.

### Integration points
- `DeckView` rendered by `App` (T0.5) when route is `Main`.
- `Mixer` imported by `deck.rs` to keep the 3-column assembly in one place.

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | CSS class `deck-{side}` where `side` is lowercase "a"/"b" — if someone passes "A", the class becomes `deck-A` which won't match CSS. | `side.to_lowercase()` in the format string. |
| Simplicity  | Could use a single `<Deck>` component with a `DeckId` enum instead of `side: &'static str`. | `&'static str` is simpler for the scaffold. A `DeckId` enum will be introduced in M1 with `DeckState`. |
| Coupling    | `DeckPlaceholder` imports `Mixer` via `crate::components::mixer`. | Clean — each component owns its sub-layout. |
| Performance | N/A. | — |
| Testability | Visual layout — tested manually. | — |

---

## Implementation Notes

`deck.rs` imports `Mixer` directly rather than going through `components::mod.rs` re-exports to keep the import explicit and traceable.

---

## Test Results

**Automated:**
```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 warnings
```

**Manual steps performed:**
- [ ] Three columns visible with "DECK A", "MIXER", "DECK B" labels (requires trunk serve)
- [ ] Deck A column has blue top border; Deck B orange; Mixer green

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- In M1, `DeckPlaceholder` will be replaced by a full `<Deck deck_state=... audio_deck=...>` component. The placeholder is intentionally minimal.
- The `side: &'static str` prop will be replaced by `DeckId` (an enum) in M1 when `DeckState` is introduced.
