# Feature 002 — Abyssal Depths Theme

## Status
Proposed

## Summary
Replace the Devil Rock dice theme with a new "Abyssal Depths" theme inspired by
mysterious deep-sea environments. The theme uses a midnight blue / teal / bioluminescent
aqua palette with 'Cinzel' display typography, and jellyfish-shaped SVG pip symbols on
each die face. The count of themes remains at 6.

## Problem Statement
Devil Rock is too similar in mood to the existing Horror theme — both lean into dark,
ominous aesthetics. Having two near-identical vibes in a 6-theme palette wastes a slot.
Abyssal Depths replaces Devil Rock with a visually distinct deep-sea adventure theme,
giving the theme picker meaningful variety: one dark-nature theme (Horror) and one
dark-mystery theme (Abyssal Depths) with completely different colour languages and pip
symbols.

## Goals
- The Theme enum has exactly 6 variants; `DevilRock` is removed and `AbyssalDepths` replaces it.
- Abyssal Depths die faces render jellyfish-shaped SVG pips (bell + trailing tentacles)
  in the theme's bioluminescent accent colour.
- The CSS palette covers at least: background, surface, accent, text, held-border, and
  preview colour variables, all using midnight-blue / teal / dark-indigo tones.
- The 'Cinzel' Google Font is loaded and applied as `--font-display` for this theme.
- Users who had "devil_rock" saved in localStorage are silently reset to the default
  theme (NordicMinimal) on next load — no error, no prompt.
- All existing unit tests and integration tests pass after the rename.

## Non-Goals
- **No CSS animations.** Bubble, plankton, and light-ray animations are explicitly
  deferred — consistent with the PRD's "no animations" policy.
- **No new background SVG scene.** Coral silhouettes, floating particles, and light
  rays are purely aesthetic atmosphere; implementing them as background elements is
  out of scope for this feature.
- **No migration prompt.** Users whose saved theme silently falls back to the default
  will not receive a notification.
- **No changes to other themes.** Horror, Nordic Minimal, Borg, Renaissance, and
  Pacific NW are untouched.
- **No new theme slots.** The count stays at 6; this is purely a swap.

## User Stories
- As a player, I want to select "Abyssal Depths" in Settings so that my dice display
  a deep-sea aesthetic while I play.
- As a player viewing the Settings theme picker, I want to see a preview die face with
  a jellyfish pip so that I can identify the Abyssal Depths theme at a glance.
- As a player who previously used Devil Rock and returns after the update, I want the
  app to load cleanly (falling back to the default theme) so that no error or broken
  state is shown.

## Functional Requirements
1. `Theme::AbyssalDepths` shall exist in the `Theme` enum; `Theme::DevilRock` shall be
   removed.
2. `Theme::all()` shall return exactly 6 variants, with `AbyssalDepths` occupying the
   position previously held by `DevilRock`.
3. `Theme::as_data_attr_value()` for `AbyssalDepths` shall return `"abyssal_depths"`.
4. `Theme::display_name()` for `AbyssalDepths` shall return `"Abyssal Depths"`.
5. `Theme::from_data_attr("abyssal_depths")` shall return `Some(Theme::AbyssalDepths)`.
6. `Theme::from_data_attr("devil_rock")` shall return `None` (handled by the existing
   default fallback in the storage layer — no special migration code needed).
7. `DiceFace` component with `theme = Theme::AbyssalDepths` shall dispatch to
   `abyssal_depths::face(value)`.
8. `abyssal_depths::face(value)` shall render a 100×100 SVG with one jellyfish pip per
   canonical pip position (using `pip_positions(value)`).
9. Each jellyfish pip shall consist of a circular bell and at least 3 trailing tentacle
   lines, all using `var(--color-accent)`.
10. `[data-theme="abyssal_depths"]` CSS block shall define all 6 custom properties:
    `--color-bg`, `--color-surface`, `--color-accent`, `--color-text`,
    `--color-held-border`, `--color-preview`, `--font-body`, `--font-display`.
11. `--font-display` for this theme shall be `'Cinzel', serif`; the 'Cinzel' font shall
    be imported from Google Fonts in the CSS (or `index.html`).
12. The `[data-theme="devil_rock"]` CSS block shall be removed.
13. All references to `devil_rock.rs` in `src/dice_svg/mod.rs` shall be updated to
    `abyssal_depths`.

## UI / UX Notes
- The Settings theme picker shows a 56×56 px die face preview card per theme; the
  Abyssal Depths card will show a single jellyfish pip (face value = 1) centred on a
  deep-blue surface.
- Colour palette target values (adjust for contrast as needed):
  - `--color-bg`: `#050d1a` (near-black midnight blue)
  - `--color-surface`: `#0a1f2e` (deep dark indigo-blue)
  - `--color-accent`: `#00e5c8` (bioluminescent teal/aqua)
  - `--color-text`: `#b2f0e8` (pale aqua, high contrast on dark)
  - `--color-held-border`: `#00bfa5` (medium teal)
  - `--color-preview`: `#38b2ac` (teal glow)
  - `--font-body`: `sans-serif`
  - `--font-display`: `'Cinzel', serif`
- 'Cinzel' requires a Google Fonts `<link>` in `index.html` (same pattern as
  'Metal Mania' used by the Devil Rock theme).
- Jellyfish pip design: bell is an ellipse (~6×5 radius) with flat bottom and a
  convex top arc; 3–4 sinuous tentacles using SVG `<path>` quadratic bezier curves
  drooping from the base, all stroked/filled with `var(--color-accent)`.
  Keep it recognisable at 56×56 and 20×20 (small preview).

## Architecture Fit
**Existing files touched:**
- `src/state/theme.rs` — rename enum variant, update `as_data_attr_value()`,
  `display_name()`, `from_data_attr()`, `all()`, and the 4 unit tests that assert
  count / round-trip.
- `src/dice_svg/mod.rs` — replace `pub mod devil_rock` with `pub mod abyssal_depths`;
  update `DiceFace` match arm.
- `style/main.css` — replace `[data-theme="devil_rock"]` block with
  `[data-theme="abyssal_depths"]` block.
- `index.html` — replace 'Metal Mania' Google Fonts link with 'Cinzel' link (or add
  alongside it if the font tag is shared).
- `tests/integration.rs` — update any test that hardcodes `"devil_rock"` or
  `Theme::DevilRock`.

**New files introduced:**
- `src/dice_svg/abyssal_depths.rs` — jellyfish pip renderer (`face(value)` function,
  same interface as all other theme modules).

**No new state, persistence, or worker changes required.** The storage layer's
`from_data_attr` → `None` fallback already gracefully handles the removed key.

## Open Questions
~~- Should the jellyfish tentacles use `<line>` elements (simple) or short `<path>`
  quadratic beziers (more organic)? Decision needed before implementation.~~
**Resolved:** Use quadratic bezier `<path>` elements for sinuous, organic-looking tentacles.

~~- Should 'Cinzel' be added to `index.html` alongside the current Google Fonts link,
  or replace it entirely (removing 'Metal Mania' since Devil Rock is gone)?~~
**Resolved:** Replace the 'Metal Mania' Google Fonts link with 'Cinzel'. Since Devil Rock
is removed, Metal Mania has no remaining consumers.

## Out of Scope / Future Work
- CSS keyframe animations for bubbles, plankton particles, and bioluminescent pulsing
  (deferred pending PRD revision on the "no animations" policy).
- Coral silhouette background elements.
- Light-ray overlay effects.
- A dedicated Abyssal Depths atmospheric background scene.

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan
*To be determined*

## Spec Changes
*To be determined (list any doc/*.md files that will need updating)*

## Test Strategy
*To be determined*

## Decisions Made
*To be determined*

## Lessons / Highlights
*To be determined*
