# rw_index — Specification

## Overview

`rw_index` is the front door to the Rook Wasteland collection. It is a single-page app (plain HTML + CSS + vanilla JS — no build step, no framework) that reads a local `apps.json` file and presents the available apps in a browsable, lightly whimsical interface. Clicking an app card navigates to that app.

It deploys to `/rw_index/` in the combined static site, just like every other app. The top-level server redirects `/` → `/rw_index/`. Other apps are true siblings at `/rw_chess/`, `/rw_poetry/`, etc.

---

## Goals

- Give the collection a recognisable home that reflects its silly, wasteland spirit.
- Stay deliberately small: one HTML file, one CSS block, one JS block, one JSON data file.
- No build step. Files are static assets served as-is.
- Easy to extend: adding a new app means adding one entry to `apps.json`.

---

## File Layout

```
apps/rw_index/
├── index.html        ← entire app (HTML + embedded CSS + embedded JS)
├── apps.json         ← data file listing all apps
├── make.py           ← build script (mirrors other apps in the monorepo)
├── README.md
└── doc/
    └── rw_index_spec.md   ← this file
```

`apps.json` is a sibling to `index.html` and is fetched at runtime with a simple `fetch('apps.json')` call.

---

## make.py

`make.py` follows the same interface as other apps in the monorepo so the top-level assembly step can invoke it uniformly.

| Target | Behaviour |
|---|---|
| `build` | No-op (nothing to compile). Prints a confirmation message. |
| `test` | No-op (no tests). Prints a confirmation message. |
| `dist` | Copies `index.html` and `apps.json` into `dist/`, creating `dist/` if needed. |
| `help` | Prints usage. |

```bash
python make.py build   # → nothing to do
python make.py test    # → nothing to do
python make.py dist    # → copies assets to dist/
```

---

## apps.json Schema

```json
[
  {
    "name": "Chess",
    "slug": "rw_chess",
    "path": "/rw_chess/",
    "icon": "♜",
    "tagline": "Short punchy one-liner shown on the card.",
    "description": "A fuller sentence or two shown on hover / expanded state.",
    "status": "live"
  }
]
```

| Field | Type | Required | Notes |
|---|---|---|---|
| `name` | string | yes | Display name shown on the card title |
| `slug` | string | yes | Machine-readable key matching the app directory name; used for CSS class and aria attributes |
| `path` | string | yes | URL path to navigate to on click (e.g. `/rw_chess/`) |
| `icon` | string | yes | A single emoji or Unicode character used as the card's visual anchor |
| `tagline` | string | yes | Short one-liner (≤ 60 chars) shown directly on the card at rest |
| `description` | string | yes | A sentence or two shown on hover/focus expand |
| `status` | `"live"` \| `"coming-soon"` | yes | `"coming-soon"` cards are rendered but not clickable |

---

## UI Design

### Aesthetic

Dark background (near-black, slightly warm), off-white text, muted accent colours per-card. The feeling should be: a hand-painted sign at the entrance to something odd. Clean grid layout, no clutter, small intentional bits of personality (slightly uneven card borders, subtle noise texture, a footer quip that changes on load).

### Layout

```
┌────────────────────────────────────────┐
│  ♜ Rook Wasteland          [? Random] │  ← header bar
├────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────┐ │
│  │  ♜       │  │  👾      │  │  📜  │ │
│  │  Chess   │  │ Defender │  │Poetry│ │
│  │ tagline  │  │ tagline  │  │      │ │
│  └──────────┘  └──────────┘  └──────┘ │  ← card grid
├────────────────────────────────────────┤
│  a small silly footer quip             │  ← footer
└────────────────────────────────────────┘
```

Grid is CSS Grid, auto-fills columns with `minmax(220px, 1fr)`, so it reflows gracefully from 1 to N columns.

### Card States

| State | Behaviour |
|---|---|
| **Rest** | Icon, name, tagline visible. Subtle drop-shadow. |
| **Hover / Focus** | Card lifts (transform: translateY). Description fades in below the tagline. Cursor changes to `pointer`. |
| **Active (click)** | Brief scale-down "press" animation before navigation. |
| **Coming soon** | Card rendered at reduced opacity, dashed border, cursor `not-allowed`. No hover lift. Badge reading "coming soon" overlaid. |

### Random Button

A `[? Random]` button in the header picks a random **live** app from the list and navigates to it immediately. Good for indecision.

### Footer

A short rotating quip drawn from a small hardcoded array in the JS. Changes each page load. Examples:
- *"You're here. There's nowhere left to go."*
- *"This is fine."*
- *"Abandon productivity, all ye who enter."*
- *"No loitering. (But also, loiter as long as you want.)"*

---

## Behaviour

1. On `DOMContentLoaded`, fetch `apps.json`.
2. For each entry, create a card element and append it to the grid.
3. Live cards: clicking navigates to `entry.path` (`window.location.href = path`).
4. Coming-soon cards: clicking does nothing (no navigation, no error).
5. Random button: filters for `status === "live"`, picks one at random, navigates.
6. If `apps.json` fails to load, show a small inline error message in the grid area ("couldn't load the app list — try refreshing").

---

## Accessibility

- Cards are `<a>` elements (live) or `<div role="img" aria-label="...">` (coming soon) so keyboard navigation works naturally.
- All icons have `aria-hidden="true"`; the app name provides the accessible label.
- Focus ring is clearly visible (not suppressed).
- Colour contrast meets WCAG AA for text on card backgrounds.

---

## Deployment

`rw_index` has no compile step. `make.py dist` copies `index.html` and `apps.json` into `dist/`. The top-level assembly step then treats `rw_index` identically to every other app — copying its `dist/` into a named subdirectory:

```
dist/
├── rw_index/         ← from rw_index/dist/  (server redirects / → here)
├── rw_chess/         ← from rw_chess/dist/
├── rw_defender/      ← from rw_defender/dist/
└── rw_poetry/        ← from rw_poetry/dist/
```

Because `rw_index` lives at `/rw_index/`, links to sibling apps use absolute paths (e.g. `/rw_chess/`) so they resolve correctly regardless of how the server redirect is configured.

---

## Initial apps.json Content

```json
[
  {
    "name": "Chess",
    "slug": "rw_chess",
    "path": "/rw_chess/",
    "icon": "♜",
    "tagline": "Play chess against an AI with a personality problem.",
    "description": "Three difficulty levels, each backed by a ridiculous persona with live in-game commentary. Easy, medium, or hard — they're all weird.",
    "status": "live"
  },
  {
    "name": "Defender",
    "slug": "rw_defender",
    "path": "/rw_defender/",
    "icon": "👾",
    "tagline": "Shoot things. They shoot back.",
    "description": "Classic arcade vertical shooter with waves, bosses every 5 rounds, power-ups, and a high score saved locally. Enemies procedurally drawn in pixel art.",
    "status": "live"
  },
  {
    "name": "Poetry",
    "slug": "rw_poetry",
    "path": "/rw_poetry/",
    "icon": "📜",
    "tagline": "Read poems. Record yourself. Cringe later.",
    "description": "Browse public-domain poetry, record yourself reading aloud, save it locally, and replay it. Surprisingly calming, occasionally mortifying.",
    "status": "live"
  }
]
```

---

## Non-Goals

- No framework (React, Svelte, etc.) — this page needs zero build tooling.
- No server-side rendering — it is a static file.
- No search or filtering (there are not enough apps to need it yet).
- No theming controls, user settings, or preferences.
