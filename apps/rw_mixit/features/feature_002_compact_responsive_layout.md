# Feature 002 — Compact Responsive Layout

## Status
Implemented

## Summary
Redesign the CSS layout so that the full DJ mixer UI fits within the visible viewport on smaller laptop screens (down to 800×600) without requiring any vertical scrolling. A tiered approach is used: progressive CSS media queries shrink canvas elements and reduce spacing at each breakpoint, with a JavaScript-driven scale fallback for the smallest viewports where content still overflows.

## Problem Statement
On laptops with a viewport height below roughly 900px (e.g. a 13" MacBook at 1280×800 with a browser chrome), the deck columns exceed the visible height and require scrolling to reach lower controls (EQ, FX panel, BPM, Load Track). This makes the app awkward to use without a second monitor or a very large screen. The three-column layout is the right structure; the problem is that fixed-size canvas elements (platter at 240px, waveform at 80px) and generous padding consume too much vertical space.

## Goals
- All deck controls visible without scrolling at 1280×800 viewport (post-header).
- Acceptable usability at 1024×768.
- Best-effort fit at 800×600 via CSS scale fallback — no scrolling even at this size.
- No mobile support required; minimum supported width remains ~800px.
- CSS + one small JS-driven scale Effect are the primary vehicle; Rust/Leptos changes are acceptable if required for layout correctness.
- Vertical page-level scrolling is acceptable as a final fallback at extreme viewport sizes — content must never be silently clipped.

## Non-Goals
- True mobile layout (portrait phone, touch-first interaction).
- Collapsible / toggleable control groups — user prefers everything visible.
- Aggressive font shrinkage — text should remain readable at all supported sizes.
- Changing the three-column deck/mixer/deck structure.
- Rewriting fixed canvas pixel dimensions in Rust (CSS scaling is sufficient).

## User Stories
- As a user on a 1280×800 laptop, I want to load a track and access all controls — transport, loop, hot cues, EQ, FX, pitch, BPM — without scrolling, so I can DJ without hunting for controls.
- As a user on a 1024×768 screen, I want the UI to be compact but still usable, so I can use the app on an older or smaller machine.
- As a user who resizes the browser window mid-session, I want the layout to adapt smoothly to the new size without breaking the deck UI.

## Functional Requirements
1. At viewport height ≥ 900px: current layout unchanged (baseline).
2. At viewport height < 900px: deck gap and padding reduce by ~25%; platter `max-width` reduces from 240px to 200px; waveform `max-height` reduces from 80px to 65px.
3. At viewport height < 768px: deck gap/padding reduce by ~50%; platter `max-width` ≤ 160px; waveform `max-height` ≤ 50px; EQ knob size reduces.
4. At viewport height < 640px: further reduction — platter ≤ 130px; waveform ≤ 40px; section label font sizes shrink.
5. At viewport height < 580px (scale fallback): `#rw-mixit-root` receives `transform: scale(factor); transform-origin: top center;` where `factor = viewport_height / 580` clamped to `[0.6, 1.0]`, computed by a Leptos `Effect` that listens to `window.resize`.
6. Horizontal: at viewport width < 1280px, the existing 1200px media query continues to apply; no new horizontal breakpoints are needed unless layout testing reveals overflow.
7. The scale fallback must not cause visible jank on resize — debounce the resize handler by ~100ms.

## UI / UX Notes
- All controls remain visible at all target sizes — no collapsing or hiding.
- Canvas elements (platter, waveform, VU meter bar) are the primary candidates for size reduction, as they are visual ornaments that can tolerate scaling without losing functionality.
- Text labels and button text should remain at or above `0.7rem` to stay legible.
- The header (`★ rw_mixit ★` + nav links) can reduce its vertical padding at small heights (e.g. `0.4rem` instead of `0.75rem`).
- The deck column `gap` between sections (currently `0.75rem`) is a major source of wasted space and should compress first.
- The VU meter (currently `height: 80px`) can reduce to `height: 50px` at the smallest breakpoints.
- The BPM panel row (TAP / SYNC / MASTER labels) can compress to `font-size: 0.7rem` at small sizes.
- At the scale fallback level, the browser's scroll bars should not appear — use `overflow: hidden` on `html, body` only when scale is active.

## Architecture Fit
- **Touched files**: `static/style.css` (primary), `src/app.rs` (scale Effect + resize listener).
- **New CSS**: `@media (max-height: …)` blocks for four height breakpoints; CSS custom properties (`--platter-max`, `--waveform-max-h`, `--deck-gap`, `--deck-pad`) defined at `:root` and overridden at each breakpoint to avoid scattered rules.
- **New Rust**: A small `Effect` in `App` component (or a separate `use_viewport_scale` helper in `src/utils/`) that reads `window.innerHeight` via `web_sys::Window::inner_height`, computes the scale factor, and sets an inline style on `#rw-mixit-root` or a CSS custom property on `:root`. A `resize` event listener (same pattern as the existing `hashchange` listener) triggers recomputation.
- **No state changes**: The scale factor is presentation-only and does not need to persist or be shared via Leptos context.
- **Canvas draw functions** (`platter_draw.rs`, `waveform_draw.rs`): no changes needed — CSS display scaling is sufficient; the canvas renders at full internal resolution and the browser downscales it visually.

## Open Questions
*(All resolved — see Decisions Made below)*

## Out of Scope / Future Work
- Full mobile layout (portrait, touch-optimized controls).
- Persisting user's preferred layout density (e.g. a "compact mode" toggle in Settings).
- Per-user font scaling / accessibility zoom support.
- Horizontal scrolling prevention at widths below 800px.

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan

### Files modified
- **`static/style.css`** — `overflow-y: auto` on `body`; `min-height: 100vh` on `#rw-mixit-root`; ten CSS custom properties at `:root` (including `--fader-h`); `min-height: 0; overflow-y: auto` on `.deck` and `.mixer`; `min-width: 0` on `.crossfader` and `.master-vol`; `overflow: hidden` on `.mixer-section`; `transform: scale(var(--app-scale, 1))` on `#rw-mixit-root`; four `@media (max-height)` blocks (900, 768, 640, breakpoint-specific `--fader-h` overrides); column-stacking breakpoint raised 900px → 600px width; `.deck-header` / `.deck-header-spacer` / `.btn-load-icon` rules added; all baseline sizes reduced (gap `0.75→0.4rem`, pad `1→0.6rem`, header `0.75→0.35rem`, platter `240→200px`, waveform `60→44px`, eq-knob `64→44px`, hot-cue `2.6→2.2rem`, bpm-value `1.4→1.1rem`).
- **`src/app.rs`** — `update_viewport_scale()` on mount; debounced 100ms `resize` EventListener.
- **`src/utils/viewport_scale.rs`** — New module: `compute_scale_factor(f64) -> f64` (pure) + `update_viewport_scale()` (DOM side-effect); 5 native unit tests.
- **`src/utils/mod.rs`** — `pub mod viewport_scale;` added.
- **`src/components/deck.rs`** — Full-width Load Track button at deck bottom replaced with a `📂` icon button (1.8 rem square) placed in a new `.deck-header` flex row at the top of the deck, alongside the deck title. A `.deck-header-spacer` element mirrors the button width to keep the title centred.

### Deviations from Architecture Fit
- `SCALE_THRESHOLD_PX` is `640.0` (not `580.0` as originally proposed) - aligns with the lowest CSS breakpoint.
- Used `HtmlElement.style().set_property()` instead of `set_attribute("style", ...)` to avoid clobbering other inline styles on `<html>`.

## Spec Changes

- **`doc/implementation_plan.md`**: Added M12 row, marked Done.
- **`doc/rw_mixit_spec.md`**: No changes needed.
- **`doc/rw_mixit_tech_spec.md`**: No changes needed (Load Track icon button and `--fader-h` var not described at spec level).
- **`doc/ascii_wireframes.md`**: No changes needed.

## Test Strategy

Five Tier-1 native tests in `src/utils/viewport_scale.rs` cover: at-threshold → 1.0; above-threshold → 1.0; proportional below threshold; extreme small → MIN_SCALE clamp; just-below boundary. DOM side-effect of `update_viewport_scale()` and CSS visual correctness verified by manual smoke test.

Several overflow bugs (Load Track button clipped, crossfader pushing siblings off-screen) were discovered during manual browser testing and not caught by automated tests. These bugs existed in the initial committed implementation and required post-implementation follow-up commits to fix.

## Decisions Made

| # | Question | Decision | Rationale |
|---|---|---|---|
| 1 | Scale factor approach | **Imperative** - resize listener writes `--app-scale` CSS property directly on `:root` | Scale is purely visual; no reactive overhead needed. Same pattern as `hashchange` listener. |
| 2 | `overflow` / height strategy | **Reversed from initial implementation.** Final: `overflow-y: auto` on `body` + `min-height: 100vh` on `#rw-mixit-root` | Initially used `overflow: hidden` always-on + `min-height: 100vh`. This silently clipped content when the root grew taller than the viewport. Final approach allows page-level scroll as a last resort at extreme sizes; content is never hidden. |
| 3 | Scale threshold | **Rust constant** `SCALE_THRESHOLD_PX = 640.0` | Aligned with lowest CSS breakpoint for clean hand-off. |
| 4 | Column-stacking breakpoint | **Raised 900px → 600px** width | At 800x600 (minimum target) the three-column layout must stay intact. |
| 5 | Load Track button location | **Moved to deck header icon button** (`📂`, 1.8rem square, `.btn-load-icon`) in a `.deck-header` flex row | Full-width bottom-of-deck button consumed significant vertical space and was unreachable when the deck overflowed. Top-of-deck icon eliminates that overhead entirely. Required a Leptos component change (`deck.rs`). |
| 6 | Channel fader height | **CSS var `--fader-h`** (`80px` baseline, scaled down at each breakpoint) | Hardcoded `80px` prevented the mixer from shrinking. Using a custom property lets the same `@media (max-height)` blocks that control deck elements also control fader height. |
| 7 | Range input min-width in flex | **`min-width: 0`** on `.crossfader` and `.master-vol` | `input[type=range]` has a browser-default `min-width` (~129px). Without `min-width: 0` it overflows its flex container and pushes siblings outside the visible panel. |

## Lessons / Highlights

### cb.forget() must be inside the successful set_timeout branch
When wrapping a Closure for set_timeout_with_callback_and_timeout_and_arguments_0, cb.forget() must only be called inside the if let Ok(handle) branch. If set_timeout fails and forget() is called unconditionally, the closure leaks permanently on every resize event. Fix: move cb.forget() inside the success branch so a failed set_timeout drops the closure normally.

### `min-height: 100vh` + `overflow: hidden` silently clips bottom content
`min-height: 100vh` lets the root div grow taller than the viewport when its children overflow. Paired with `overflow: hidden` on `html/body`, the overflow is silently clipped — controls at the bottom are invisible and unreachable. Use `height: 100vh` (exact constraint) on the root if you want no-scroll, OR use `min-height: 100vh` + `overflow-y: auto` to allow page-level scroll as a fallback instead of clipping.

### `min-height: 0` must be on EVERY flex child in the height-constraint chain
Flex children default to `min-height: auto`, which prevents them from shrinking below their intrinsic content height. If even one element in the chain (`#root → .deck-row → .deck`) is missing `min-height: 0`, the height constraint silently breaks and content overflows the viewport.

### `margin-top: auto` in a scrollable flex column pushes to intrinsic bottom, not visible bottom
`margin-top: auto` in a flex column pushes the element to the bottom of the column's intrinsic content height, not the bottom of the visible scroll area. When the column is scrollable (`overflow-y: auto`), this places the element beyond the visible viewport. Remove `margin-top: auto` from elements that need to be visible without scrolling.

### `input[type=range]` has a browser-default `min-width` (~129px) that breaks flex layouts
When an `<input type="range">` is a `flex: 1` child without `min-width: 0`, its browser-default `min-width` (~129px) prevents it from shrinking. It overflows the flex container and pushes other siblings outside the visible panel. Fix: add `min-width: 0` to the range input (or its wrapping element) so the flex algorithm can shrink it properly.