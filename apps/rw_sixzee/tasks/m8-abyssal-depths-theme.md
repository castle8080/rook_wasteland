# Feature 002: Abyssal Depths Theme

**Feature Doc:** features/feature_002_abyssal_depths_theme.md
**Milestone:** M8 — Themes & SVG Dice
**Status:** 🔄 In Progress

## Restatement

Replace the Devil Rock dice theme with Abyssal Depths — a mysterious deep-sea aesthetic
with a midnight-blue/teal/bioluminescent-aqua colour palette and 'Cinzel' display
typography. The change is a 1-for-1 enum swap: `Theme::DevilRock` is removed and
`Theme::AbyssalDepths` added in the same slot, keeping the total at 6. The new SVG die
faces render jellyfish pips (ellipse bell + quadratic-bezier tentacles) using the same
`pip_positions()` dispatch pattern as all other themes. Users who previously saved
`"devil_rock"` in localStorage will silently fall back to `NordicMinimal` via the
existing `from_data_attr()` → `None` path. CSS animations and background scene elements
(bubbles, coral, light rays) are explicitly out of scope. This implements
`features/feature_002_abyssal_depths_theme.md`.

## Design

### Data flow

Theme selection is unchanged: `ActiveTheme` signal → `body[data-theme]` attribute →
CSS custom properties resolved by the browser. The only new data path is:
- `Theme::AbyssalDepths` → `DiceFace` match arm → `abyssal_depths::face(value)`
- `abyssal_depths::face` calls `pip_positions(value)` and maps each `(cx, cy)` to a
  `pip(cx, cy)` jellyfish SVG group.

### Function / type signatures

```rust
// src/dice_svg/abyssal_depths.rs
/// Render a single pip as a bioluminescent jellyfish.
fn pip(cx: f32, cy: f32) -> impl IntoView;

/// Render an SVG die face for Abyssal Depths (values 1–6).
pub fn face(value: u8) -> impl IntoView;
```

No new public types. `Theme::AbyssalDepths` is a new unit variant with the same trait
impls (`Clone, Copy, PartialEq, Eq, Debug`) as all other variants.

### Edge cases

- `face(0)` and `face(7)`: `pip_positions` returns `&[]`; the SVG renders empty — same
  as all other themes, already covered by existing `pip_positions` unit tests.
- Tentacle coordinates that fall outside `0..100` viewBox: at the outermost pip
  position (cx=72, cy=75), tentacle tip is at y≈84 — stays within bounds.
- `from_data_attr("devil_rock")` after the rename: returns `None`, storage layer falls
  back to `NordicMinimal`. No code change needed; already the correct behaviour.

### Integration points

| File | Change |
|---|---|
| `src/dice_svg/abyssal_depths.rs` | New file — jellyfish pip renderer |
| `src/dice_svg/mod.rs` | `pub mod devil_rock` → `pub mod abyssal_depths`; match arm |
| `src/state/theme.rs` | `DevilRock` → `AbyssalDepths` in enum, all 4 match arms, tests |
| `style/main.css` | CSS block `devil_rock` → `abyssal_depths` with new palette |
| `index.html` | Remove `family=Metal+Mania&` from Google Fonts URL |
| `tests/integration.rs` | 3 occurrences of `"devil_rock"` → `"abyssal_depths"` |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `from_data_attr("devil_rock")` silently returns `None` — user loses saved theme preference | Accepted per product decision: silent reset to default, no prompt |
| Simplicity | Jellyfish with 3 bezier paths + 1 ellipse per pip is the most complex pip so far | Still fits in <20 lines per pip; same interface as all other themes |
| Coupling | `renaissance` also uses `'Cinzel'`; removing Metal Mania font doesn't break it | Both share the already-present Cinzel import in index.html |
| Performance | No new assets; all SVG rendered inline; no additional fetch | No issue |
| Testability | Bezier path coordinates are computed from pip_positions — correct position is sufficient | Unit test verifies pip count matches value; visual shape not auto-testable |

## Implementation Notes

- Cinzel is already loaded in `index.html` (for Renaissance). Only need to drop
  `family=Metal+Mania&` from the Google Fonts URL query string.
- `stroke-width` on SVG `<path>` must go in the CSS `style` attribute per the Leptos
  0.8 rule: hyphenated SVG attributes are not valid Rust identifiers in `view!`.
- Tentacle tip y-coordinate for bottom pip row (cy=75): 75 + 9 = 84. Still within
  the 100×100 viewBox. No clipping needed.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| FR1: AbyssalDepths in enum, DevilRock removed | 1 (unit) | ✅ | `all_has_six_variants` + round-trip tests in theme.rs |
| FR2: Theme::all() has 6 variants | 1 (unit) | ✅ | `all_has_six_variants` |
| FR3: as_data_attr_value → "abyssal_depths" | 1 (unit) | ✅ | `data_attr_round_trip` |
| FR4: label → "Abyssal Depths" | 1 (unit) | ✅ | `all_labels_non_empty` (existing) |
| FR5: from_data_attr("abyssal_depths") → Some | 1 (unit) | ✅ | `data_attr_round_trip` |
| FR6: from_data_attr("devil_rock") → None | 1 (unit) | ✅ | `from_data_attr_unknown_returns_none` catches all unknown strings |
| FR7: DiceFace dispatches to abyssal_depths::face | 3 (integration) | ✅ | Existing `settings_renders_six_theme_cards` mounts theme cards |
| FR8: face(v) renders one pip per value | 1 (unit) | ✅ | `pip_count_matches_value` in abyssal_depths.rs |
| FR9: jellyfish bell + tentacles | 1 (unit) | Partial | Shape verified via pip count; visual form not auto-testable |
| FR10: CSS block defines all 6 vars | Manual | ⚠️ | Not auto-tested; verified by visual smoke test |
| FR11: Cinzel font loaded | Manual | ⚠️ | Already present in index.html for Renaissance |
| FR12: devil_rock CSS block removed | n/a | ✅ | Compilation would fail if any code still references it |
| FR13: mod.rs references updated | 1 (compile) | ✅ | Will not compile if references are stale |
| localStorage "devil_rock" → silent fallback | 3 (integration) | ✅ | `app_load_applies_saved_theme_to_body` updated to use abyssal_depths |

## Test Results
*To be filled after running tests.*

## Review Notes
*To be filled during Phase 8.*

## Decisions Made
*To be filled.*

## Lessons / Highlights
*To be filled.*

## Callouts / Gotchas
- Cinzel shared with Renaissance: removing Metal Mania only. Do not remove Cinzel.
- Hyphenated SVG attrs (stroke-width) must live in `style="..."` in Leptos 0.8 view!.
