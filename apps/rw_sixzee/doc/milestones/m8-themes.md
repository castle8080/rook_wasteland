# M8 — Themes & SVG Dice

<!-- MILESTONE: M8 -->
<!-- STATUS: DONE -->

**Status:** ✅ Done
**Depends on:** [M5 — Core Game UI](m5-core-game-ui.md)
**Required by:** [M10 — Polish & Mobile](m10-polish-mobile.md)

---

## Overview

Replace the plain numeric die faces with themed SVG art and implement the full theme system: 6 themes, CSS custom
property overrides, a settings screen with live preview, and localStorage persistence of the selected theme. Theme
switching applies instantly without a page reload.

---

## Success Criteria

- [x] Each of the 6 themes displays correctly: background colours, surface colours, accent, text colour, and display
  font all change when the theme is selected
- [x] All 5 dice display SVG faces (not plain numbers) using the active theme's symbol set
- [x] Each SVG face clearly communicates pip count (1 through 6) despite custom symbols
- [x] All 6 die faces (1–6) exist for every theme (36 SVG components total)
- [x] The Settings screen shows a 2-column grid of 6 theme cards, each with a representative die face preview
  and colour swatch; the active theme card shows a ✓ indicator
- [x] Clicking a theme card applies the theme immediately (before any page reload)
- [x] The selected theme persists across browser sessions (stored in `rw_sixzee.theme` localStorage key)
- [x] On fresh install (no stored preference), Nordic Minimal is the default
- [x] Held-state double-border styling is visible and legible on all 6 themes
- [x] Score preview colours (`.scorecard__cell--preview`) are readable on all 6 theme backgrounds

---

## Tasks

### Theme Enum & Application

- [x] Define `Theme` enum with 6 variants: `DevilRock`, `Borg`, `Horror`, `Renaissance`,
  `NordicMinimal`, `PacificNorthwest`
- [x] Implement `Theme::as_data_attr_value() -> &'static str` mapping to the `data-theme` attribute strings
- [x] Implement `Theme::default() -> Theme` returning `NordicMinimal`
- [x] In `App`, hold `RwSignal<Theme>`; on change, call `document.body.dataset().set("theme", …)` and
  call `storage::save_theme()`; provide via context

### CSS — 6 Theme Blocks

For each theme, add a `[data-theme="..."]` block in `style/main.css` overriding:
`--color-bg`, `--color-surface`, `--color-accent`, `--color-text`, `--color-held-border`,
`--color-preview`, `--font-body`, `--font-display`

- [x] **Nordic Minimal** (`nordic_minimal`): off-white bg, slate grey surface, moss/rust accent,
  dark text, neutral held border, blue-grey preview; sans-serif fonts
- [x] **Devil Rock** (`devil_rock`): near-black bg `#0a0a0a`, dark red surface, neon red accent `#ff2020`,
  acid yellow text `#f5e642`, neon red held border; Metal Mania or similar gothic display font
- [x] **Borg** (`borg`): dark charcoal bg, steel surface, cold cyan accent, cyan/green text,
  cyan held border, monospace body font
- [x] **Horror** (`horror`): deep black bg, dark crimson surface, sickly green accent, near-white text,
  dripping-red held border; serif display font
- [x] **Renaissance** (`renaissance`): warm parchment bg `#f5e6c8`, burnished gold surface, deep ultramarine
  accent, near-black text, gold held border; calligraphic serif display font
- [x] **Pacific Northwest** (`pacific_northwest`): forest green bg, driftwood tan surface, slate stone accent,
  dark earthy text, cedar-tone held border; earthy ink-wash aesthetic sans-serif

### SVG Dice Module Structure (`src/dice_svg/`)

- [x] Create `src/dice_svg/mod.rs` with `DiceFace` component:
  ```rust
  #[component]
  pub fn DiceFace(theme: Theme, value: u8) -> impl IntoView
  ```
  dispatching to the correct per-theme face component
- [x] Each theme module exports `Face1` through `Face6` (or a single `face(v: u8)` dispatcher)
- [x] All faces use `viewBox="0 0 100 100"` and are inline SVG (no external files)
- [x] Replace the placeholder numeric die render in `dice_row.rs` with `<DiceFace theme=... value=... />`

### SVG Die Faces — per Theme

Each theme: 6 face components with custom symbols arranged to communicate pip count clearly.

- [x] **Nordic Minimal:** geometric dots styled as runes/snowflakes; stark, precise; pip count readable
- [x] **Devil Rock:** pentagrams, inverted crosses, flames; gothic numerals (face 1 = single pentagram, etc.)
- [x] **Borg:** hexagonal circuit nodes, assimilation glyphs, binary tally marks
- [x] **Horror:** skulls, dripping blood, claw marks, eyeballs; pip positions preserved
- [x] **Renaissance:** illuminated manuscript flourishes, gilded rosettes, Roman numerals
- [x] **Pacific Northwest:** cedar rings, salal leaves, salmon silhouettes, mountain outlines

### Settings Screen (`src/components/settings.rs`)

- [x] Render a "THEME" section header
- [x] 2-column responsive grid of 6 theme cards
- [x] Each card contains:
  - `DiceFace` for face 6 (representative) in that theme's style (rendered with `data-theme` scoped styles)
  - Colour swatch bar (small `<div>` with `background-color: var(--color-bg)`)
  - Theme name label
  - ✓ indicator when this theme is active (`.settings__theme-card--active`)
- [x] Clicking/tapping a card sets `RwSignal<Theme>` in context, applies immediately
- [x] "Theme applies instantly — no reload needed." informational note

### CSS

- [x] Add `.settings`, `.settings__theme-grid`, `.settings__theme-card`, `.settings__theme-card--active`
- [x] Each theme card has `min-height: 120px`; grid is `repeat(2, 1fr)` on mobile,
  `repeat(3, 1fr)` on wider viewports
- [x] Active card uses accent border (`.settings__theme-card--active { border: 2px solid var(--color-accent); }`)
- [x] **E2E smoke tests** (`e2e/smoke.spec.ts`): 4 tests — grid renders with 6 cards; clicking a card
  updates `data-theme` on `<body>`; theme persists across reload; active card shows ✓ indicator

---

## Notes & Risks

- **SVG symbols vs. pip positions:** Die faces must still communicate the pip count visually.
  Nordic Minimal can use the exact pip positions with styled dots. Other themes may use
  larger symbolic art but should still place symbols at the same relative positions so the
  count is intuitive (e.g. 6 = 3×2 grid of symbols).
- **Theme card SVG scoping:** Rendering all 6 theme card previews on the settings screen means
  each card must show SVG with that theme's visual style even if it isn't the active theme.
  Options: (a) apply a `data-theme` attribute to the card `<div>` wrapper and scope CSS accordingly,
  (b) hardcode preview colours in SVG `fill` attributes. Option (a) is cleaner.
- **Font loading:** Display fonts (Metal Mania for Devil Rock, etc.) must be loaded via Google Fonts or
  bundled. Add `<link>` tags to `index.html` for fonts not available system-wide.
  Fallback fonts (cursive, monospace, serif) are acceptable for initial implementation.
- **Held-border contrast:** The `.dice-row__die--held` double-border must be visually distinct from
  the surface colour on every theme. Verify all 6 themes manually.

---

## Implementation Summary

**Completed:** (before 2026-03-09 audit) · 4 Playwright E2E tests + wasm-pack browser integration tests

### Key Design Decisions

**`data-theme` attribute on each settings card `<button>` for CSS isolation**
The spec noted two options: scope CSS via `data-theme` on the card wrapper, or hardcode fill
colours in SVG attributes. Option (a) was chosen — each settings card `<button>` carries
`data-theme=attr_val`, so the SVG inside inherits the correct CSS custom properties without
any per-theme hardcoding in Rust. This means adding a new theme requires only a new CSS block
and a new Rust module, with zero changes to the card rendering logic.

**`pip_positions()` shared helper in `dice_svg/mod.rs`**
All 6 theme face modules call the same `pip_positions(value)` helper that returns canonical
`(x, y)` coordinates for each pip count on a `100×100` viewBox. This ensures pip layout is
consistent across themes while each theme's symbol shape (dot, pentagram, hex node, skull, etc.)
is independently defined. Adding a 7th theme only requires a new module that calls this helper.

**`ActiveTheme(RwSignal<Theme>)` newtype in context**
Following the project convention for `RwSignal<bool>` newtypes (see `ShowResume`, `GameActive`),
the theme signal is wrapped in `ActiveTheme` so its `TypeId` is distinct from any other
`RwSignal<Theme>` that might be added later. This is consistent with the rest of the context
architecture and prevents silent context collisions.

**Body attribute set synchronously before first render**
`set_body_theme(initial_theme.as_data_attr_value())` is called immediately after loading from
localStorage, before the Leptos `Effect` fires. This prevents a visible flash of the default
theme on pages where a non-default theme was saved.

**`aria-label` on settings cards is a reactive closure**
The `aria-label` on each theme card reads `is_active()` and formats `"Select X theme (active)"`
vs `"Select X theme"`. This is a tracked signal read inside a `move || ...` closure per the
project's Leptos reactivity convention (see `lessons.md`).

### Files Changed

| File | Change |
|---|---|
| `src/state/theme.rs` | New file — `Theme` enum, `as_data_attr_value()`, `label()`, `all()`, `from_data_attr()`, `Default` |
| `src/state/storage.rs` | Added `save_theme()`, `load_theme()`, `KEY_THEME` constant |
| `src/state/mod.rs` | Added `ActiveTheme(pub RwSignal<Theme>)` newtype |
| `src/app.rs` | Theme load on init, `set_body_theme()` sync call, `Effect` for persistence, `provide_context(ActiveTheme(...))` |
| `src/dice_svg/mod.rs` | New file — `DiceFace` component, `pip_positions()` helper |
| `src/dice_svg/nordic.rs` | 6 face functions (geometric dots) |
| `src/dice_svg/devil_rock.rs` | 6 face functions (pentagrams) |
| `src/dice_svg/borg.rs` | 6 face functions (hex circuit nodes) |
| `src/dice_svg/horror.rs` | 6 face functions (stylised skulls) |
| `src/dice_svg/renaissance.rs` | 6 face functions (illuminated rosettes) |
| `src/dice_svg/pacific_nw.rs` | 6 face functions (cedar rings) |
| `src/components/dice_row.rs` | Replaced plain number render with `<DiceFace theme=theme value=v />` |
| `src/components/settings.rs` | New file — `SettingsView` component, `theme_card` helper |
| `style/main.css` | 6 `[data-theme="..."]` blocks; `.settings__theme-*` CSS rules |
| `e2e/smoke.spec.ts` | `describe("M8 Themes")` block with 4 Playwright tests |
