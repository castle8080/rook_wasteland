# rw_sixzee — Project Plan

<!-- PROJECT_STATUS: IN_PROGRESS -->

## Overview

**rw_sixzee** is a client-side, solitaire, 6-column variant of Sixzee built with Leptos 0.8 (CSR) + Trunk, compiled to
WebAssembly. There is no server; all game logic, persistence, and AI advisory computation run entirely in the browser.

The project is structured as ten milestones. Each milestone delivers a testable, manually-verifiable slice of
functionality. Milestones are ordered by implementation dependency: foundational infrastructure first, scoring engine
and its DP validation early to prove the approach, UI and persistence next, and themes/history/polish last.

---

## Milestone Summary

| ID  | Name                          | Status         | Detail |
|-----|-------------------------------|----------------|--------|
| M1  | Project Bootstrap             | ✅ COMPLETE    | [doc/milestones/m1-bootstrap.md](milestones/m1-bootstrap.md) |
| M2  | Game State & Scoring Engine   | ✅ COMPLETE    | [doc/milestones/m2-scoring-engine.md](milestones/m2-scoring-engine.md) |
| M3  | Grandma's Sayings             | ✅ COMPLETE    | [doc/milestones/m3-grandma-sayings.md](milestones/m3-grandma-sayings.md) |
| M4  | DP Precomputation             | ✅ COMPLETE    | [doc/milestones/m4-dp-precomputation.md](milestones/m4-dp-precomputation.md) |
| M5  | Core Game UI                  | 🔲 NOT STARTED | [doc/milestones/m5-core-game-ui.md](milestones/m5-core-game-ui.md) |
| M6  | Persistence & Resume          | 🔲 NOT STARTED | [doc/milestones/m6-persistence.md](milestones/m6-persistence.md) |
| M7  | Ask Grandma                   | 🔲 NOT STARTED | [doc/milestones/m7-ask-grandma.md](milestones/m7-ask-grandma.md) |
| M8  | Themes & SVG Dice             | 🔲 NOT STARTED | [doc/milestones/m8-themes.md](milestones/m8-themes.md) |
| M9  | History Screen                | 🔲 NOT STARTED | [doc/milestones/m9-history.md](milestones/m9-history.md) |
| M10 | Polish & Mobile               | 🔲 NOT STARTED | [doc/milestones/m10-polish-mobile.md](milestones/m10-polish-mobile.md) |

**Status values:** 🔲 NOT STARTED · 🔄 IN PROGRESS · ✅ COMPLETE · 🚫 BLOCKED

---

## Dependency Graph

```
M1 (Bootstrap)
 └─► M2 (Scoring Engine)
      ├─► M3 (Grandma's Sayings)  ← content; can be drafted any time after M2
      │    └─► (feeds M5 quote display components)
      ├─► M4 (DP Precomputation)   ← validate scoring approach early
      │    └─► M7 (Ask Grandma)
      └─► M5 (Core Game UI)
           └─► M6 (Persistence & Resume)
                └─► M9 (History Screen)
M5 ──────────────► M8 (Themes & SVG Dice)
M6, M7, M8, M9 ──► M10 (Polish & Mobile)
```

---

## Ordering Rationale

**M1 first** — the crate skeleton, build tooling, routing, CSS architecture, and error types are prerequisites for
everything else. Nothing else can be built or tested without this.

**M2 second** — all scoring logic (pure Rust, no WASM needed) enables native `cargo test` validation of game rules
immediately. Game state and scoring functions are consumed by M4, M5, M6, and M7.

**M3 (Grandma's Sayings)** — this is a content creation milestone, not a
development task. It exists to establish Grandma's voice and generate the full
quote pools through an agent-assisted creative process. It is placed immediately
after M2 because quote content can be drafted and refined at any point once the
game rules are understood — and having it early means the content is ready when
the quote-loading infrastructure is built in M5. The milestone has no blocking
dependencies on M4 or later milestones.

**M4 third (DP Precomputation)**— the offline DP solver and generated `V_COL` table are placed early to validate the
core scoring-estimation approach before any UI depends on it. The offline tool stands alone and can be verified
numerically (see M4 success criteria). This de-risks the Ask Grandma feature (M7) before building its UI.

**M5 fourth** — the playable game UI (dice, scorecard, roll/hold/score) integrates M2 game state with Leptos
components. This is the first milestone that produces a playable browser build.

**M6 fifth** — persistence (auto-save, resume, history append) builds directly on M5's game flow and is required before
M9 and parts of M10.

**M7 sixth** — Ask Grandma uses the DP table from M4 and game state from M2. The Web Worker infrastructure is
isolated enough to build after the playable core is stable.

**M8 seventh** — themes and SVG dice are purely additive visual work with no game logic dependencies. Placed after the
game is playable to avoid re-theming partially built UI.

**M9 eighth** — History screen requires completed game records (M6) and the read-only scorecard layout (M5).

**M10 last** — responsive CSS, mobile touch polish, error handling completeness, and full integration tests are applied
once all features exist.

---

## Key Technical Decisions

| Decision | Choice |
|----------|--------|
| UI framework | Leptos 0.8 CSR (no SSR) |
| Build system | Trunk (WASM) |
| Routing | Hash-based (`#/game`, `#/history`, `#/settings`) — no leptos_router |
| Persistence | `localStorage` via `web-sys`; keys prefixed `rw_sixzee.` |
| Ask Grandma computation | Precomputed DP table (32 KB, embedded in Worker WASM binary) + MC sampling |
| Error handling | `AppError`/`AppResult` (thiserror); `unwrap()` denied; `expect()` only at 3 permitted sites |
| CSS | Single flat `style/main.css` with BEM naming + CSS custom properties for theming |
| Default theme | Nordic Minimal |
| SVG dice | Inline Leptos components; 6 faces per theme; no runtime asset fetch |
| Worker bundling | Separate Trunk WASM binary for Ask Grandma; Worker JS shim in `dist/` |
| rw_index launcher | `apps/rw_index/apps.json` updated in M1 with `"status": "coming_soon"`; changed to `"live"` on final deployment |
| Grandma Quotes | `assets/grandma_quotes.json` fetched at runtime; `QuoteBank` stored in context; failure → `Degraded` (silent) |
