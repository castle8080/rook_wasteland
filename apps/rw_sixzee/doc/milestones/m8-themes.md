# M8 — Themes & SVG Dice

<!-- MILESTONE: M8 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
**Depends on:** [M5 — Core Game UI](m5-core-game-ui.md)
**Required by:** [M10 — Polish & Mobile](m10-polish-mobile.md)

---

## Overview

Replace the plain numeric die faces with themed SVG art and implement the full theme system: 6 themes, CSS custom
property overrides, a settings screen with live preview, and localStorage persistence of the selected theme. Theme
switching applies instantly without a page reload.

---

## Success Criteria

- [ ] Each of the 6 themes displays correctly: background colours, surface colours, accent, text colour, and display
  font all change when the theme is selected
- [ ] All 5 dice display SVG faces (not plain numbers) using the active theme's symbol set
- [ ] Each SVG face clearly communicates pip count (1 through 6) despite custom symbols
- [ ] All 6 die faces (1–6) exist for every theme (36 SVG components total)
- [ ] The Settings screen shows a 2-column grid of 6 theme cards, each with a representative die face preview
  and colour swatch; the active theme card shows a ✓ indicator
- [ ] Clicking a theme card applies the theme immediately (before any page reload)
- [ ] The selected theme persists across browser sessions (stored in `rw_sixzee.theme` localStorage key)
- [ ] On fresh install (no stored preference), Nordic Minimal is the default
- [ ] Held-state double-border styling is visible and legible on all 6 themes
- [ ] Score preview colours (`.scorecard__cell--preview`) are readable on all 6 theme backgrounds

---

## Tasks

### Theme Enum & Application

- [ ] Define `Theme` enum with 6 variants: `DevilRock`, `Borg`, `Horror`, `Renaissance`,
  `NordicMinimal`, `PacificNorthwest`
- [ ] Implement `Theme::as_data_attr_value() -> &'static str` mapping to the `data-theme` attribute strings
- [ ] Implement `Theme::default() -> Theme` returning `NordicMinimal`
- [ ] In `App`, hold `RwSignal<Theme>`; on change, call `document.body.dataset().set("theme", …)` and
  call `storage::save_theme()`; provide via context

### CSS — 6 Theme Blocks

For each theme, add a `[data-theme="..."]` block in `style/main.css` overriding:
`--color-bg`, `--color-surface`, `--color-accent`, `--color-text`, `--color-held-border`,
`--color-preview`, `--font-body`, `--font-display`

- [ ] **Nordic Minimal** (`nordic_minimal`): off-white bg, slate grey surface, moss/rust accent,
  dark text, neutral held border, blue-grey preview; sans-serif fonts
- [ ] **Devil Rock** (`devil_rock`): near-black bg `#0a0a0a`, dark red surface, neon red accent `#ff2020`,
  acid yellow text `#f5e642`, neon red held border; Metal Mania or similar gothic display font
- [ ] **Borg** (`borg`): dark charcoal bg, steel surface, cold cyan accent, cyan/green text,
  cyan held border, monospace body font
- [ ] **Horror** (`horror`): deep black bg, dark crimson surface, sickly green accent, near-white text,
  dripping-red held border; serif display font
- [ ] **Renaissance** (`renaissance`): warm parchment bg `#f5e6c8`, burnished gold surface, deep ultramarine
  accent, near-black text, gold held border; calligraphic serif display font
- [ ] **Pacific Northwest** (`pacific_northwest`): forest green bg, driftwood tan surface, slate stone accent,
  dark earthy text, cedar-tone held border; earthy ink-wash aesthetic sans-serif

### SVG Dice Module Structure (`src/dice_svg/`)

- [ ] Create `src/dice_svg/mod.rs` with `DiceFace` component:
  ```rust
  #[component]
  pub fn DiceFace(theme: Theme, value: u8) -> impl IntoView
  ```
  dispatching to the correct per-theme face component
- [ ] Each theme module exports `Face1` through `Face6` (or a single `face(v: u8)` dispatcher)
- [ ] All faces use `viewBox="0 0 100 100"` and are inline SVG (no external files)
- [ ] Replace the placeholder numeric die render in `dice_row.rs` with `<DiceFace theme=... value=... />`

### SVG Die Faces — per Theme

Each theme: 6 face components with custom symbols arranged to communicate pip count clearly.
Symbol descriptions (from PRD §6.35):

- [ ] **Nordic Minimal:** geometric dots styled as runes/snowflakes; stark, precise; pip count readable
- [ ] **Devil Rock:** pentagrams, inverted crosses, flames; gothic numerals (face 1 = single pentagram, etc.)
- [ ] **Borg:** hexagonal circuit nodes, assimilation glyphs, binary tally marks
- [ ] **Horror:** skulls, dripping blood, claw marks, eyeballs; pip positions preserved
- [ ] **Renaissance:** illuminated manuscript flourishes, gilded rosettes, Roman numerals
- [ ] **Pacific Northwest:** cedar rings, salal leaves, salmon silhouettes, mountain outlines

### Settings Screen (`src/components/settings.rs`)

- [ ] Render a "THEME" section header
- [ ] 2-column responsive grid of 6 theme cards
- [ ] Each card contains:
  - `DiceFace` for face 6 (representative) in that theme's style (rendered with `data-theme` scoped styles)
  - Colour swatch bar (small `<div>` with `background-color: var(--color-bg)`)
  - Theme name label
  - ✓ indicator when this theme is active (`.settings__theme-card--active`)
- [ ] Clicking/tapping a card sets `RwSignal<Theme>` in context, applies immediately
- [ ] "Theme applies instantly — no reload needed." informational note

### CSS

- [ ] Add `.settings`, `.settings__theme-grid`, `.settings__theme-card`, `.settings__theme-card--active`
- [ ] Each theme card has `min-height: 120px`; grid is `repeat(2, 1fr)` on mobile,
  `repeat(3, 1fr)` on wider viewports
- [ ] Active card uses accent border (`.settings__theme-card--active { border: 2px solid var(--color-accent); }`)
- [ ] **E2E smoke test** (`e2e/smoke.spec.ts`): navigate to `#/settings`, verify theme grid renders
  with at least one card; click a theme card, verify `data-theme` attribute updates on `<body>`;
  reload page, verify theme persists (confirms localStorage save + load round-trip)

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
