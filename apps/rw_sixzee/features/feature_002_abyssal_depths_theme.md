# Feature 002 — Abyssal Depths Theme

## Status
Implemented

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

### Files added
- `src/dice_svg/abyssal_depths.rs` — jellyfish pip renderer: `pip(cx, cy)` private
  function builds an SVG `<g>` of one `<ellipse>` bell (rx=5, ry=4, offset cy-2) and
  three quadratic-bezier `<path>` tentacles (left, centre, right). All strokes use
  `var(--color-accent)`. `stroke-width` lives in the CSS `style` attribute per
  Leptos 0.8 hyphenated-attribute restriction. Three unit tests added inline.

### Files modified
- `src/state/theme.rs` — `DevilRock` → `AbyssalDepths` in enum declaration, all four
  match arms (`as_data_attr_value`, `label`, `all`, `from_data_attr`), and the existing
  unit tests (5 tests, all continue to pass with the new variant name).
- `src/dice_svg/mod.rs` — `pub mod devil_rock` → `pub mod abyssal_depths`; match arm
  in `DiceFace` updated; module doc comment updated.
- `style/main.css` — `[data-theme="devil_rock"]` block replaced with
  `[data-theme="abyssal_depths"]` block (#050d1a bg / #0a1f2e surface / #00e5c8
  accent / #b2f0e8 text / #00bfa5 held-border / #38b2ac preview / 'Cinzel' display).
- `index.html` — `family=Metal+Mania&` removed from Google Fonts URL; Cinzel already
  present for Renaissance so no net change to loaded fonts.
- `tests/integration.rs` — three `"devil_rock"` literals → `"abyssal_depths"` in the
  storage round-trip test and the app-load-applies-theme integration test.

### Deviations from Architecture Fit section
None. All changes matched the spec exactly.

## Spec Changes
- **`doc/tech_spec.md`** — updated §2 repo layout (devil_rock.rs → abyssal_depths.rs),
  §8.2 CSS theme example block (devil_rock → abyssal_depths palette), and §9.1
  DiceFace match example (DevilRock → AbyssalDepths dispatch).
- **`doc/prd.md`** — updated FR 41 theme table row 1 (Devil Rock → Abyssal Depths
  with new vibe, dice symbols, and palette description).
- **`doc/wireframes.md`** — No changes needed; wireframes describe layout structure
  not theme names.
- **`doc/project_plan.md`** — No changes needed; this feature is within M8 scope and
  the milestone entry does not enumerate individual theme names.

## Test Strategy

### Tests added
- `src/dice_svg/abyssal_depths.rs` (Tier 1, native):
  - `pip_count_matches_value` — asserts `pip_positions(v).len() == v` for v in 1..=6
  - `out_of_range_values_produce_no_pips` — asserts `pip_positions(0)` and `pip_positions(7)` are empty
  - `tentacle_tips_stay_within_viewbox` — asserts `cy + 9.0 < 100.0` for all 6-pip positions

### Tests updated
- `tests/integration.rs`:
  - `storage_save_and_load_theme_round_trip` — now uses `"abyssal_depths"` string
  - `app_load_applies_saved_theme_to_body` — now saves and asserts `"abyssal_depths"`

### Coverage gaps
- The visual shape of jellyfish pips (bezier curvature, ellipse proportions) is not
  auto-testable. Covered by the manual smoke test: open Settings and verify dice faces
  look like jellyfish at 56×56 px.
- CSS custom property values are not auto-tested (no headless CSS inspector available
  in wasm-pack tests). Covered by smoke test.

## Decisions Made

### Decision: Cinzel shared with Renaissance
**Chosen:** Reuse the Cinzel Google Font import already present for Renaissance; only
remove Metal Mania from the fonts URL.
**Alternatives considered:** Add a separate Cinzel import tag, or use a different font.
**Rationale:** Cinzel was already loaded and gives the right mysterious/elegant register
for deep-sea themes. Sharing reduces HTTP requests; one font URL covers both themes.

### Decision: Silent fallback for devil_rock localStorage value
**Chosen:** `from_data_attr("devil_rock")` returns `None`; storage layer falls back to
`NordicMinimal`. No migration code, no user notification.
**Alternatives considered:** Explicit mapping `"devil_rock" → AbyssalDepths`.
**Rationale:** Product decision made during feature submission. The themes are visually
different enough that silently mapping would be confusing. A clean reset is simpler.

### Decision: Three tentacles via quadratic bezier paths
**Chosen:** Three `<path>` elements with `Q` (quadratic bezier) commands, each curving
left/right from the bell base.
**Alternatives considered:** `<line>` elements (straight), more tentacles (4–5).
**Rationale:** Bezier curves were requested by the user for organic feel. Three is
sufficient for legibility at 56×56 px without crowding the 100×100 viewBox pip slot.

## Lessons / Highlights

### Cinzel Already Loaded for Renaissance
Before checking `index.html`, the plan assumed Metal Mania would need replacing with a
separate Cinzel link. In practice Cinzel was already in the same font URL alongside
Metal Mania and MedievalSharp. Only Metal Mania needed removal. Always audit the actual
`index.html` before writing font migration notes — the file may already contain the font.

### Hyphenated SVG Attributes in Leptos 0.8 view!
`stroke-width` is hyphenated and cannot be used as a Leptos `view!` attribute directly.
It must be embedded in the CSS `style` attribute (`style="stroke-width:1.2;fill:none;"`).
This applies to any SVG attribute containing a hyphen. See `doc/lessons.md` §L18 for
the general rule covering all hyphenated attrs.
