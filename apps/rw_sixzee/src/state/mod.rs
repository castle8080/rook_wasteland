//! Game state management — implemented across milestones M2–M8.

pub mod game;
pub mod quotes;
pub mod scoring;
pub mod theme;
#[cfg(target_arch = "wasm32")]
pub mod storage;

pub use theme::Theme;

// ── Context-signal newtypes ────────────────────────────────────────────────
//
// Leptos resolves `provide_context` / `use_context` by `TypeId`. Wrapping each
// `RwSignal<bool>` in its own newtype gives each a unique `TypeId`, preventing
// silent context collisions when multiple `bool` signals are in scope.

use leptos::prelude::RwSignal;

/// Newtype for the `game_active` signal — `true` while an active game is in
/// progress on the Game tab; `false` causes `GameView` to render the
/// idle/pre-game screen instead.
#[derive(Clone, Copy)]
pub struct GameActive(pub RwSignal<bool>);

/// Newtype for the `show_resume` overlay signal — `true` while the
/// Resume-vs-New prompt is visible.
#[derive(Clone, Copy)]
pub struct ShowResume(pub RwSignal<bool>);

/// Newtype for the `show_opening_quote` signal — `true` while the
/// Grandma opening-quote overlay should be displayed.
#[derive(Clone, Copy)]
pub struct ShowOpeningQuote(pub RwSignal<bool>);

/// Newtype for the `hide_tab_bar` signal — `true` while ANY overlay that
/// must obscure the tab bar (confirm-zero, opening-quote, grandma panel) is open.
#[derive(Clone, Copy)]
pub struct HideTabBar(pub RwSignal<bool>);

/// Newtype for the active `Theme` signal — the currently selected visual theme.
///
/// Provided at the `App` root; read by `DiceRow`, `SettingsView`, and any
/// component that needs theme-aware rendering.
#[derive(Clone, Copy)]
pub struct ActiveTheme(pub RwSignal<Theme>);
