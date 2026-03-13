# Feature 003 — In-App User Guide

## Status
Implemented

## Summary
Add a dedicated Help page accessible via a `[Help]` link in the top navigation bar. The page presents a short, fun, numbered quick-start walkthrough that gets a first-time user mixing in under a minute. Content is hardcoded in a Leptos component styled like the existing About card.

## Problem Statement
New users arrive at a complex dual-deck mixer UI with no onboarding. The most critical first step — loading a track — is now a small icon button in the deck header that easy to miss. Without any in-app guidance, users may not discover how to start, get confused by the controls, and give up before they ever hear a beat drop. A short, energetic guide fixes the cold-start problem without cluttering the main UI.

## Goals
- A `[Help]` nav link appears in the header alongside `[Settings]` and `[About]`.
- Navigating to `#/help` renders the guide; the main mixer is hidden (same pattern as About/Settings).
- The guide is a numbered quick-start covering the essential path: load → play → level → crossfade.
- Copy is fun and on-brand (hip-hop / cartoon DJ aesthetic) — not a dry manual.
- No new dependencies, no external files, no runtime loading.

## Non-Goals
- Full controls reference (FX, loop, hot cues, keyboard shortcuts) — keep it short.
- Collapsible or accordion sections.
- Animated screenshots, GIFs, or video embeds.
- Localization / i18n.
- Persisting "have seen guide" state (no first-run auto-redirect).

## User Stories
- As a first-time user, I want to find a Help link in the header so I know where to look for guidance.
- As a first-time user, I want a quick numbered walkthrough so I can load a track and start mixing in under a minute without guessing.
- As a returning user, I want to navigate back to the mixer easily after reading the guide so I'm not stuck on the help page.

## Functional Requirements
1. `Route::Help` is added to `src/routing.rs`; the hash is `#/help`.
2. A `[Help]` anchor is added to the header nav, rendered between `[Settings]` and `[About]` (or after — match existing nav order convention).
3. Navigating to `#/help` shows `HelpView` and hides the main deck row (same `<Show>` gating as `SettingsView` and `AboutView`).
4. Clicking the logo or any other nav link returns the user to the appropriate view (existing routing logic handles this — no new work needed).
5. `HelpView` contains a numbered list of at least 4 quick-start steps covering: load a track on each deck, press play, adjust volume faders, move the crossfader.
6. Copy must match the app's cartoon / hip-hop personality — short sentences, energetic, emoji welcome.
7. Layout matches the existing `about-card` pattern (centered, max-width card, scrollable if the guide grows).

## UI / UX Notes
- Use the same `.about-view` / `.about-card` CSS classes (or `.help-view` / `.help-card` with identical styles) so the guide inherits the panel aesthetic automatically.
- Steps should use a numbered list (`<ol>`) with generous line-height for readability.
- Each step should have a bold action word (e.g. **Drop a track**) followed by a one-sentence explanation.
- A small footer note pointing to `[Settings]` for advanced options (reverb, crossfader curve) is welcome but optional.
- Emoji are encouraged for visual rhythm — 🎵 🎛️ 📂 🎚️ — but keep them tasteful.
- The `[Help]` header link should use the same `.rw-nav a` styling as the existing links.

## Architecture Fit
- **`src/routing.rs`**: Add `Help` variant to `Route` enum; add `"#/help"` parse arm and `to_hash()` arm.
- **`src/components/header.rs`**: Add `<a href="#/help">` anchor in the nav, same pattern as `[Settings]` and `[About]`.
- **`src/components/help.rs`**: New file. `HelpView` component — static `view!` macro with `.help-view` / `.help-card` layout and `<ol>` step list.
- **`src/components/mod.rs`**: Add `pub mod help;`.
- **`src/app.rs`**: Add `<Show when=move || current_route.get() == Route::Help> <HelpView/> </Show>` block.
- **`static/style.css`**: Add `.help-view` and `.help-card` rules (can simply mirror `.about-view` / `.about-card`).
- **No state changes**: `HelpView` is purely presentational; no signals, no context.
- **No audio changes**: The audio graph keeps running while the user is on the Help page (same as About/Settings).

## Open Questions
- None — all decisions resolved during triage.

## Out of Scope / Future Work
- A collapsible "Advanced Controls" section covering FX, loop, hot cues, BPM sync, and keyboard shortcuts.
- First-run detection (auto-show guide on first visit via `localStorage`).
- Keyboard shortcut cheat-sheet table.
- Dark/light theme awareness in guide layout.

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan

Six files were modified or created:

| File | Change | Reason |
|---|---|---|
| `src/routing.rs` | Added `Help` variant to `Route` enum; added `"#/help"` parse arm and `to_hash()` arm; added `from_hash_help` test; updated `round_trip_all_routes` to include `Route::Help` | Routing support for the new page |
| `src/components/header.rs` | Added `[Help]` anchor between `[Settings]` and `[About]`; added `help_link_sets_help_hash` WASM test; added `#![allow(clippy::let_unit_value, clippy::unwrap_used)]` to the wasm test module | Nav link and test coverage |
| `src/components/help.rs` | **New file.** `HelpView` component — purely presentational; `.help-view`/`.help-card` layout; `.help-steps` `<ol>` of 4 quick-start steps; hip-hop copy with emoji; footer link to `#/settings` | The guide page itself |
| `src/components/mod.rs` | Added `pub mod help;` | Expose the new module |
| `src/app.rs` | Imported `HelpView`; added `<Show when=move \|\| current_route.get() == Route::Help> <HelpView/> </Show>` | Wire the route to the view |
| `static/style.css` | Added `.help-view`, `.help-card`, `.help-title`, `.help-tagline`, `.help-steps`, `.help-step`, `.help-step-title`, `.help-footer`, `.help-link` rules | Dedicated styling for the guide card |

**Architectural note:** The same `<Show>`/hide pattern as `AboutView` and `SettingsView` is used — `DeckView` stays mounted while the user is on the Help page, so the audio graph keeps running (music continues playing). No state changes were needed; `HelpView` is purely presentational with no signals or context.

**Deviation from original plan:** The wasm test module in `header.rs` received `#![allow(clippy::let_unit_value, clippy::unwrap_used)]`. This was not in the original plan but was needed to suppress intentional test-code patterns in the new `help_link_sets_help_hash` test, and it also fixed 7 pre-existing clippy violations in the same module at no extra cost.

## Spec Changes

The following `doc/` files were updated as part of this feature:

- `doc/rw_mixit_spec.md` — Added `Route::Help` / `#/help` to the navigation routes table in §7.3.
- `doc/rw_mixit_tech_spec.md` — Added `Route::Help` to the `Route` enum and `from_hash`/`to_hash` examples in §6; added `<HelpView>` to the component tree in §10.1; added `help.rs` to the module layout in §3.
- `doc/implementation_plan.md` — Added Feature 003 as a completed entry.
- `doc/implementation_lessons_and_notes.md` — Added lesson on `round_trip_all_routes` as a route completeness guard.

## Test Strategy

**Tier 1 — native `cargo test`:**
- `from_hash_help` in `src/routing.rs` — verifies `"#/help"` parses to `Route::Help`.
- `round_trip_all_routes` in `src/routing.rs` — verifies every `Route` variant survives a `to_hash` → `from_hash` round-trip. The test array must include every variant, so adding `Route::Help` to the enum without updating the test is a compile-time failure.

**Tier 2 — `wasm-pack test --headless --firefox`:**
- `help_link_sets_help_hash` in `src/components/header.rs` — mounts the `Header` component, clicks the `[Help]` anchor, and asserts the URL hash is `#/help`.

**`HelpView` DOM rendering:** waived. The component is purely static HTML with no signals or event handlers. Manual smoke test (visually inspect at `#/help`) is sufficient.

## Decisions Made

1. **Dedicated `.help-*` CSS classes rather than reusing `.about-*`** — allows the guide's visual style to evolve independently of the About card without coupling them. The initial styles mirror `.about-*` for consistency.

2. **`[Help]` link placed between `[Settings]` and `[About]`** — `[Settings]` is the most frequently accessed secondary page and stays first; `[About]` is informational and naturally comes last; `[Help]` sits in the middle as the onboarding gateway.

3. **4 steps only (essentials path):** load → play → levels → crossfader. FX, loops, hot cues, and keyboard shortcuts are deferred to a footer hint that links to `[Settings]`. Keeping it to four steps means the guide fits on a single screen without scrolling on typical displays.

4. **Footer link to `#/settings`** — provides a natural continuation path for users who want to explore reverb, crossfader curve, and other advanced options after they have the basics down.

## Lessons / Highlights

### `round_trip_all_routes` as a route completeness guard

The `round_trip_all_routes` test in `src/routing.rs` keeps an explicit array of every `Route` variant and asserts that `from_hash(to_hash(route)) == route` for each one. Because the array is hand-maintained, adding a new variant to the enum without updating the test causes a compilation error (non-exhaustive pattern match if the match arm is missing) or a test failure (the new variant is not exercised). Either way, the breakage is caught immediately and locally — no runtime surprise. This pattern is worth replicating in any app where routing coverage is important: a small, explicit test that enumerates every variant doubles as a completeness contract.

### `#![allow(...)]` at module level in a `mod wasm_tests {}` block

Applying `#![allow(clippy::let_unit_value, clippy::unwrap_used)]` as an inner attribute at the top of a `#[cfg(test)] mod wasm_tests { ... }` block suppresses intentional test-code patterns (unit-value bindings from `view!`, `unwrap()` on DOM queries) without polluting production code with blanket lints. The inner `#!` syntax scopes the allow to the module only. This is cleaner than per-function `#[allow(...)]` attributes when the same lint fires in every test function in the module.
