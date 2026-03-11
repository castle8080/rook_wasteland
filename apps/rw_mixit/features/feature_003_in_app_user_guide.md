# Feature 003 — In-App User Guide

## Status
Proposed

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
*To be determined*

## Spec Changes
*To be determined (list any doc/*.md files that will need updating)*

## Test Strategy
*To be determined*

## Decisions Made
*To be determined*

## Lessons / Highlights
*To be determined*
