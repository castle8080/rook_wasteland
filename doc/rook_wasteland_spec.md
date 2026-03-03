# Rook Wasteland — Project Specification

## Overview

**Rook Wasteland** is a collection of silly, frivolous, time-wasting browser apps — the kind of thing you open when you should be doing something else. Every app runs entirely client-side with no backend required, so the whole site can be hosted cheaply on any static file server.

The project is structured as a monorepo. Each app lives under `apps/` and builds to a `dist/` directory of static assets. A top-level index app ties them all together and serves as the entry point into the collection.

---

## Goals

- **Pure fun.** Apps should be silly, random, or whimsical. Utility is not required.
- **Client-side only.** No servers, no databases, no accounts. Everything runs in the browser.
- **Cheap to host.** A flat directory of static files, serveable from GitHub Pages, S3, Netlify, Cloudflare Pages, or any equivalent.
- **Independent apps.** Each app under `apps/` is self-contained and can be built, tested, and run on its own.
- **Combined deployment.** A top-level build step assembles all app `dist/` outputs into a single static directory tree, rooted at the index app.

---

## Repository Structure

```
rook_wasteland/
├── README.md
├── doc/
│   └── rook_wasteland_spec.md   ← this file
├── apps/
│   ├── rw_index/                ← (planned) top-level landing page / app navigator
│   ├── rw_chess/                ← chess vs. quirky AI personas
│   ├── rw_defender/             ← arcade vertical shooter
│   └── rw_poetry/               ← poetry reader + voice journal
└── rw_serve/                    ← (planned) build/assembly tooling
```

Each app directory follows the same conventions described below.

---

## App Conventions

### Directory Layout (Rust/WASM apps)

```
apps/<app_name>/
├── src/              # Rust source
├── dist/             # Build output (git-ignored); produced by `trunk build`
├── index.html        # HTML shell
├── Trunk.toml        # Trunk build config; sets output dir to dist/
├── Cargo.toml
├── README.md         # App-level docs: what it does, how to build, how to play
├── doc/              # Design specs, architecture notes (optional)
└── tasks/            # Per-task design docs (optional)
```

### Build Toolchain (Rust/WASM)

| Concern | Tool |
|---|---|
| Language | Rust (stable, 2021 or 2024 edition) |
| Compile target | `wasm32-unknown-unknown` |
| WASM bindings | `wasm-bindgen`, `web-sys`, `js-sys` |
| Bundler | [Trunk](https://trunkrs.dev/) |
| UI (optional) | [Leptos](https://leptos.dev/) 0.8 (CSR) or raw Canvas via `web-sys` |

### Commands (Rust/WASM apps)

```bash
trunk serve          # dev server with live reload
trunk build --release  # production build → dist/
cargo test           # native unit tests
cargo clippy         # lint
cargo fmt            # format
```

### Future App Types

Other technology stacks (e.g. React, plain HTML/JS, SvelteKit) may be added. Each should still:
- Produce a flat `dist/` of static assets via a standard build command
- Include a `README.md` with build instructions
- Be independently buildable without knowledge of sibling apps

---

## Combined Static Deployment

The final deployed site is a single flat directory tree:

```
/                    ← served from rw_index's dist/
/chess/              ← rw_chess dist/ contents
/defender/           ← rw_defender dist/ contents
/poetry/             ← rw_poetry dist/ contents
```

Each app is copied into a named subdirectory, with the exception of `rw_index`, whose contents are copied directly to the root of the combined output. Because Trunk-built apps use relative asset paths, apps must be configured (via `Trunk.toml` `public_url`) to match their deployment subpath.

The assembly step (in `rw_serve/` or a top-level script) is responsible for:
1. Running `trunk build --release` (or equivalent) for each app
2. Copying each app's `dist/` into the appropriate subdirectory of the combined output
3. Placing the index app at the root

---

## The Index App (`rw_index`)

The index app is the front door to the collection. It has not been built yet but should:

- Display a tile / card for each app with a name, short description, and link
- Reflect the silly, random spirit of the site — it should feel like a weird little home page, not a corporate product listing
- Be built with the same Rust/WASM/Leptos stack as the other apps (or plain HTML/CSS/JS — it needs no logic)
- Live at `apps/rw_index/` and deploy to the root `/` of the combined output

---

## Existing Apps

### ♜ rw_chess

**What it is:** A full chess game playable against an AI opponent with personality.

**Why it's fun:** Three difficulty levels, each backed by a ridiculous AI persona with in-game commentary.

| Persona | Difficulty | Depth |
|---|---|---|
| Pawndrew, The Pawn Who Got Promoted By Accident ♟ | Easy | 2 |
| Prof. Pompington III, Author of 47 Books Nobody Has Read 🎩 | Medium | 3 |
| Grandmaster Goblin, Ancient Chess Gremlin Escaped From The Machine 👺 | Hard | 4 |

**Tech:** Rust, Leptos 0.8, WASM, Trunk. Full chess rules including castling, en passant, pawn promotion, 50-move draw. Alpha-beta search with quiescence search.

**Deployment subpath:** `/chess/`

---

### 👾 rw_defender

**What it is:** A classic arcade vertical shooter in the spirit of Space Invaders and Galaga.

**Why it's fun:** Waves of enemies, power-ups, bosses every 5 waves, and a high score saved in `localStorage`.

**Tech:** Rust, raw `web-sys` Canvas 2D (no JS game framework), WASM, Trunk. Two-canvas compositing for parallax starfield. Procedurally generated pixel-art sprites.

**Deployment subpath:** `/defender/`

---

### 📜 rw_poetry

**What it is:** A poetry reader and voice journal. Browse public-domain poems, record yourself reading them, save and replay recordings locally.

**Why it's fun:** Surprisingly calming. Or embarrassing. Depends on how you sound.

**Tech:** Rust, Leptos 0.8, WASM, Trunk. IndexedDB for local recording storage, browser Media APIs for audio capture. Static JSON poem corpus under `public/poems/`.

**Deployment subpath:** `/poetry/`

---

## Non-Goals

- No user accounts, login, or server-side state
- No analytics or tracking
- No monetisation
- No "serious" productivity apps — this is a wasteland, not a workspace

---

## Adding a New App

1. Create `apps/<app_name>/` with the appropriate toolchain structure.
2. Add a `README.md` covering what it does, how to build it, and how to play/use it.
3. Configure `Trunk.toml` (or equivalent) so the production build outputs to `dist/` and is path-relative.
4. Add a card for it in `rw_index`.
5. Register it in the assembly step in `rw_serve/`.
6. Pick a deployment subpath under `/` and document it here.
