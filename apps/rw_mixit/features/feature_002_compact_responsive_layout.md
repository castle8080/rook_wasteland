# Feature 002 — Compact Responsive Layout

## Status
Proposed

## Summary
Redesign the CSS layout so that the full DJ mixer UI fits within the visible viewport on smaller laptop screens (down to 800×600) without requiring any vertical scrolling. A tiered approach is used: progressive CSS media queries shrink canvas elements and reduce spacing at each breakpoint, with a JavaScript-driven scale fallback for the smallest viewports where content still overflows.

## Problem Statement
On laptops with a viewport height below roughly 900px (e.g. a 13" MacBook at 1280×800 with a browser chrome), the deck columns exceed the visible height and require scrolling to reach lower controls (EQ, FX panel, BPM, Load Track). This makes the app awkward to use without a second monitor or a very large screen. The three-column layout is the right structure; the problem is that fixed-size canvas elements (platter at 240px, waveform at 80px) and generous padding consume too much vertical space.

## Goals
- All deck controls visible without scrolling at 1280×800 viewport (post-header).
- Acceptable usability at 1024×768.
- Best-effort fit at 800×600 via CSS scale fallback — no scrolling even at this size.
- No mobile support required; minimum supported width remains ~800px.
- No Rust or Leptos component changes needed — CSS + one small JS-driven scale Effect.

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
*To be determined*

## Spec Changes
*To be determined (list any doc/*.md files that will need updating)*

## Test Strategy
*To be determined*

## Decisions Made

| # | Question | Decision | Rationale |
|---|---|---|---|
| 1 | Scale factor: reactive signal vs. imperative? | **Imperative** — resize listener writes a CSS custom property (`--app-scale`) directly on `:root` | Scale is purely a visual side-effect and never drives conditional rendering; no reactive overhead needed. Same pattern as `hashchange` listener in `app.rs`. |
| 2 | `overflow: hidden` scope? | **Global always-on** — set `overflow: hidden` on `html, body` unconditionally in the stylesheet | The app is never intended to scroll at any viewport size; making it globally no-scroll is the correct contract and simpler than toggling a class. |
| 3 | Scale threshold: CSS var vs. Rust constant? | **Rust constant** — `const SCALE_THRESHOLD_PX: f64 = 580.0` in the resize helper | Simple to find and change; doesn't require a CSS read at runtime. Can be revisited if fine-tuning after testing reveals a better value. |
| 4 | Column-stacking breakpoint (currently 900px)? | **Raise to 600px** — three columns stay side-by-side down to 600px wide; stacking only at phone-width | At 800×600 (the minimum target) the three-column layout must remain; the 900px breakpoint would incorrectly stack them. 600px is a reasonable phone boundary. |

## Lessons / Highlights
*To be determined*
