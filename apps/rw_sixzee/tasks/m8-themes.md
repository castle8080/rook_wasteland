# Task M8: Themes & SVG Dice

**Milestone:** M8 — Themes & SVG Dice
**Status:** ✅ Done

## Restatement

This task replaces the plain numeric die faces with per-theme SVG art and implements the
full theme system: 6 themes, CSS custom-property overrides, a Settings screen with live
preview, and localStorage persistence. The `Theme` enum (6 variants) is defined in
`src/state/`, a `DiceFace` component dispatches to one of 6 per-theme SVG modules, the
dice row replaces text labels with `DiceFace`, and the Settings screen offers a 2-column
theme-picker grid. Theme switching is instant (reactive `RwSignal<Theme>`) with no reload.
Out of scope: animations, sound, accessibility beyond ARIA roles already present.

## Design

### Data flow

1. `App` loads theme string from `storage::load_theme()` → converts to `Theme` via
   `Theme::from_data_attr()` → creates `RwSignal<Theme>`.
2. An `Effect` in `App` watches the signal, calls `set_body_theme(t.as_data_attr_value())`
   and `storage::save_theme(...)` on every change (including initial mount).
3. `App` provides `ActiveTheme(RwSignal<Theme>)` via context.
4. `DiceRow` reads `ActiveTheme` from context, passes `theme` + `value` to `DiceFace`.
5. `DiceFace(theme, value)` dispatches to the correct per-theme `face(value)` function.
6. `SettingsView` reads `ActiveTheme` from context; each theme card has
   `data-theme="{theme_attr}"` on its wrapper div, which scopes CSS variables.
   Clicking a card calls `theme_signal.set(theme_variant)`.

### Function / type signatures

```rust
// src/state/theme.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme { NordicMinimal, DevilRock, Borg, Horror, Renaissance, PacificNorthwest }
impl Theme {
    pub fn as_data_attr_value(self) -> &'static str;
    pub fn label(self) -> &'static str;
    pub fn all() -> &'static [Theme];
    pub fn from_data_attr(s: &str) -> Option<Theme>;
}

// src/state/mod.rs
pub struct ActiveTheme(pub RwSignal<Theme>);

// src/dice_svg/mod.rs
#[component]
pub fn DiceFace(theme: Theme, value: u8) -> impl IntoView;

// src/dice_svg/{nordic,devil_rock,borg,horror,renaissance,pacific_nw}.rs
pub fn face(value: u8) -> impl IntoView;
```

### Edge cases

- `Theme::from_data_attr` returns `None` for unknown strings → caller uses `unwrap_or_default()`
- `DiceFace` value outside 1–6 → renders empty SVG (no pips)
- Settings card with `data-theme` wrapper overrides body's CSS vars for preview correctness
- On initial load with no stored theme → `NordicMinimal` default

### Integration points

- `src/state/theme.rs` (new), `src/state/mod.rs` (add `ActiveTheme`)
- `src/dice_svg/mod.rs` + 6 sub-modules (new)
- `src/components/dice_row.rs` (use `DiceFace`)
- `src/components/settings.rs` (full replacement)
- `src/app.rs` (signal + Effect)
- `style/main.css` (settings CSS + SVG sizing)
- `index.html` (Google Fonts)
- `e2e/smoke.spec.ts` (M8 tests)

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `data-theme` on settings card overrides body theme | Intended — CSS cascade makes it correct |
| Simplicity | 36 SVG face functions is verbose | Use shared `pip_positions()` helper; each theme only defines a `pip()` fn + 1 `face()` fn |
| Coupling | DiceFace takes `theme: Theme` explicitly (not read from context) | Allows settings preview to render off-theme faces without faking context |
| Performance | Effect runs `save_theme` on mount too | Idempotent save is harmless; simpler than skip-first-run logic |
| Testability | SVG rendering requires browser | Unit test Theme enum (native); integration test settings DOM (wasm-pack) |

## Implementation Notes

- SVG pip positions: TL(28,25) TR(72,25) ML(28,50) MR(72,50) BL(28,75) BR(72,75) C(50,50)
- Pip layout: 1→[C], 2→[TR,BL], 3→[TR,C,BL], 4→[TL,TR,BL,BR], 5→[4+C], 6→[TL,TR,ML,MR,BL,BR]
- All SVG fills use `var(--color-accent)` / `var(--color-surface)` so theme cards pick up correct colours
- Devil Rock: 10-point star polygon (alternating outer/inner radius) — no fill-rule tricks needed
- Hyphenated SVG attributes (stroke-width, fill-opacity) use CSS `style` attribute

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `Theme::as_data_attr_value` round-trips | 1 | ✅ | unit test |
| `Theme::from_data_attr` happy path | 1 | ✅ | unit test |
| `Theme::from_data_attr` unknown string | 1 | ✅ | unit test |
| `Theme::default()` = NordicMinimal | 1 | ✅ | unit test |
| `DiceFace` value 1–6 render without panic | 2 | ✅ | wasm-pack visual test |
| Settings grid renders 6 cards | 3 | ✅ | integration.rs |
| Active theme card has --active class | 3 | ✅ | integration.rs |
| Theme card click updates data-theme on body | E2E | ✅ | smoke.spec.ts |
| Theme persists across reload | E2E | ✅ | smoke.spec.ts |

## Test Results

- Native unit tests: **92 passed** (including 5 new `state::theme::tests::*`)
- Browser integration tests: **35 passed** (including 3 new M8 tests:
  `settings_renders_six_theme_cards`, `settings_card_click_changes_body_theme`,
  `settings_active_card_has_active_class`)
- `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings`: **0 warnings**
- `trunk build`: **✅ success**

## Review Notes

No issues found during self-review or code-review agent pass.

## Callouts / Gotchas

- SVG `viewBox` attribute: written as `viewBox=` in Leptos 0.8 view! macro — valid Rust identifier
- Hyphenated SVG attrs (`stroke-width`) use `style=` string to avoid identifier parse issues
- Settings card `data-theme` scoping: ensures preview colour fidelity for all 6 themes
- Existing `app_load_applies_saved_theme_to_body` test used `"sixzee_dark"` (not a valid M8
  theme); updated to `"devil_rock"` so it tests the real round-trip behaviour
- `roll_reveals_dice_values` integration test updated to check `aria-label` instead of
  `text_content()` because rolled dice now contain SVG (no text nodes)
- Pre-existing `clippy::assertions_on_constants` lint in `advisor.rs` suppressed with
  `#[allow]` on the specific test fn

## Callouts / Gotchas

- SVG `viewBox` attribute: written as `viewBox=` in Leptos 0.8 view! macro — valid Rust identifier
- Hyphenated SVG attrs use `style=` string to avoid identifier parse issues
- Settings card `data-theme` scoping: ensures preview colour fidelity for all 6 themes
