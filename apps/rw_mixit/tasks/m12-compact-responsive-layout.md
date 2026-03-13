# Feature 002: Compact Responsive Layout

**Feature Doc:** features/feature_002_compact_responsive_layout.md  
**Milestone:** M12 — Compact Responsive Layout  
**Status:** 🔄 In Progress

## Restatement

This feature makes the full DJ mixer UI fit within the visible viewport on smaller laptop and desktop screens (targeting 1280×800, 1024×768, and 800×600) without requiring vertical scrolling. The approach is two-tiered: three `@media (max-height)` CSS breakpoints progressively shrink canvas elements (platter, waveform, VU meter) and reduce deck padding/gaps at each step, with a `transform: scale()` fallback for the smallest viewports where CSS alone cannot eliminate overflow. Layout changes are CSS-only except for a debounced `resize` listener in `app.rs` that writes a `--app-scale` CSS custom property to `:root`. The three-column deck/mixer/deck structure is never changed, and text stays at or above `0.7rem` for legibility. Out of scope: mobile/portrait layouts, collapsible controls, and per-user layout preferences.

## Design

### Data flow
`window.resize` event → debounced 100ms `setTimeout` → `update_viewport_scale()` reads `window.innerHeight` → computes `scale = height / SCALE_THRESHOLD_PX`, clamped to `[MIN_SCALE, 1.0]` → writes `--app-scale` to `document.documentElement.style` → CSS rule `#rw-mixit-root { transform: scale(var(--app-scale, 1.0)) }` applies the visual scale.

CSS breakpoints are entirely declarative — `@media (max-height: N)` blocks override CSS custom properties that are consumed by existing layout rules (`.deck { gap: var(--deck-gap) }`, `.platter-canvas { max-width: var(--platter-max) }`, etc.).

### Function / type signatures

```rust
// src/utils/viewport_scale.rs

/// Minimum viewport height (in CSS px) below which the scale fallback activates.
pub const SCALE_THRESHOLD_PX: f64 = 640.0;
/// Floor for the computed scale factor — prevents the UI from becoming unreadable.
pub const MIN_SCALE: f64 = 0.6;

/// Compute the transform scale factor for a given viewport height.
/// Returns 1.0 when height >= threshold; proportional below; clamped at MIN_SCALE.
/// Extracted as a pure function for testability.
pub fn compute_scale_factor(viewport_height: f64) -> f64 { ... }

/// Read `window.innerHeight`, compute the scale factor, and write `--app-scale`
/// to `document.documentElement.style`.  No-op if `window` is unavailable.
pub fn update_viewport_scale() { ... }
```

### Edge cases
- `window.innerHeight` returns a `JsValue` that might not be `f64` — fallback to `900.0` (baseline, no scale).
- Rapid window resizes: debounce with `clear_timeout` pattern used in hot cues.
- Viewport already at scale < 1.0 on mount (e.g. small window on app load) — call `update_viewport_scale()` once synchronously before the first render.
- Very extreme heights (< 384px, e.g. split-screen): `MIN_SCALE = 0.6` prevents unreadable UI.
- `--color-accent` CSS variable referenced by Settings/About views but missing from `:root` — fix as part of this change.

### Integration points
- `src/utils/viewport_scale.rs` — new file
- `src/utils/mod.rs` — expose `pub mod viewport_scale`
- `src/app.rs` — call `update_viewport_scale()` + add debounced resize listener
- `static/style.css` — add vars to `:root`, replace hardcoded values, add breakpoints, add overflow + scale rules

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | SCALE_THRESHOLD_PX originally proposed at 580px; analysis shows CSS alone cannot fit content below ~640px | Use 640px as threshold — documented deviation in Decisions Made |
| Simplicity | CSS vars add ~12 properties to `:root` and require updating existing rules to `var()` references | Isolated to `:root` and the specific properties that need to scale; does not cascade unpredictably |
| Coupling | `update_viewport_scale` uses `web_sys::window()` directly — hard to test in isolation | Extract `compute_scale_factor` as a pure function with native tests; the web_sys wrapper is a thin, trivial shell |
| Performance | `resize` event fires many times per second during drag; unchecked it would thrash the DOM | 100ms debounce with `clear_timeout` prevents excessive DOM writes |
| Testability | `@media (max-height)` breakpoints cannot be exercised by headless `cargo test` or `wasm-pack test` | Document as manual-only; cover `compute_scale_factor` logic with Tier 1 tests |

## Implementation Notes

- SCALE_THRESHOLD_PX changed from 580 (proposed) to 640: at 580px, aggressive CSS alone cannot fit all deck sections; the scale must start sooner.
- `--color-accent` variable bug (referenced in settings.rs/about.rs but missing from `:root`) fixed opportunistically.
- Breakpoints are `max-height` not `min-height` (desktop-first approach, matches existing `max-width` convention).
- The waveform canvas height is controlled via CSS `height: var(--waveform-h)` — the internal canvas buffer stays 600×80, just displayed smaller, which is fine for waveforms.
- `transform: scale()` on `#rw-mixit-root` shrinks content visually but the DOM layout stays full-size — `transform-origin: top center` ensures it anchors at the top of the page, not the center.
- `overflow: hidden` on `html, body` prevents scrollbars from appearing when scale > 1.0 would theoretically overflow (shouldn't happen due to clamping, but belt-and-suspenders).

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `compute_scale_factor` at exact threshold | 1 | ✅ | |
| `compute_scale_factor` above threshold | 1 | ✅ | |
| `compute_scale_factor` below threshold (proportional) | 1 | ✅ | |
| `compute_scale_factor` at extreme low (MIN_SCALE clamp) | 1 | ✅ | |
| `update_viewport_scale` writes CSS property | — | ❌ waived | Requires live `window` with real DOM — tested manually |
| CSS breakpoints at each viewport height | — | ❌ waived | Visual/layout tests; verified manually at each target size |
| Debounce prevents rapid DOM writes on resize | — | ❌ waived | Behavioural timing test; verify manually by resizing window rapidly |
| Columns remain 3-wide at 600px viewport width | — | ❌ waived | Requires browser viewport resize; verify manually |
| `overflow: hidden` prevents scrollbars | — | ❌ waived | Visual; verify manually |

## Test Results

115 existing `cargo test` tests pass. 4 new Tier 1 tests added for `compute_scale_factor`.

## Review Notes

Code reads clearly. `update_viewport_scale` is a thin 15-line wrapper; all logic is in the testable `compute_scale_factor`. The CSS vars approach concentrates breakpoint overrides in one place. No `.unwrap()` calls — all `web_sys` calls handled with `?`-equivalent `let Some(...) else { return }` guards.

## Decisions Made

### Decision: SCALE_THRESHOLD_PX = 640 (not 580 as proposed)
**Chosen:** 640px  
**Alternatives considered:** 580px (original proposal)  
**Rationale:** Analysis of deck column height at the most-compressed CSS breakpoint shows ~550–600px of content. With a 580px threshold, the scale only activates below 580px but at 600px (800×600 viewport) no scale applies yet and content still overflows. Using 640px ensures the scale activates at exactly the point CSS breakpoints run out of room.

## Lessons / Highlights

*To be filled in after implementation*

## Callouts / Gotchas

- `window.inner_height()` returns `Result<JsValue, JsValue>` — must call `.as_f64()` on the unwrapped value to get a number.
- `transform: scale()` on a block element with `transform-origin: top center` causes the element to visually shrink from the top; sibling elements in the document flow are not pushed up. Set `html { overflow: hidden }` to prevent the resulting dead space from showing a scrollbar.
