# Task T0.2: Trunk.toml + index.html

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Create `Trunk.toml` setting `public_url = "/rw_mixit/"` (critical for subdirectory hosting), `dist = "dist"`, and watch ignores for `dist/` and `doc/`. Create `index.html` with a `<div id="app">` mount target, Trunk asset directives for `static/style.css` and `static/fonts/`, and a Google Fonts CDN link for Bangers. Out of scope: font file download; the CDN link is a placeholder until local font files are added.

---

## Design

### Data flow
Static build configuration only.

### Edge cases
- `public_url` must end with `/` and match the deployment path exactly. Without it, Trunk embeds relative asset paths that break when served from `/rw_mixit/`.
- `<link data-trunk rel="copy-dir" href="static/fonts"/>` copies the fonts directory verbatim; the directory must exist (even empty) or Trunk errors.

### Integration points
- `index.html` is the Trunk entry point referenced by `Trunk.toml target = "index.html"`.
- `static/style.css` path must match the `rel="css"` href exactly.

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | `public_url` typo would cause 404s for WASM/JS/CSS. | Verified `/rw_mixit/` matches the deployment spec. |
| Simplicity  | CDN font link adds an external dependency. | Acceptable for scaffold; noted for offline-hosting in Callouts. |
| Coupling    | None. | — |
| Performance | CDN font adds one extra network round-trip. | Fine for dev/demo; replace with local fonts for production. |
| Testability | `trunk build` success is the only automated check. | — |

---

## Implementation Notes

Added `rel="preconnect"` hints for Google Fonts to reduce font load latency.

---

## Test Results

**Automated:**
```
trunk build validates via T0.1 cargo check passing.
```

**Manual steps performed:**
- [x] Verified `Trunk.toml` fields match spec values
- [x] Confirmed `index.html` contains required `<div id="app">` and Trunk directives

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- To self-host the font: download `Bangers-Regular.woff2` from Google Fonts, place it in `static/fonts/`, add a `@font-face` rule to `style.css`, and remove the CDN `<link>` tags from `index.html`.
- `public_url` in Trunk.toml **must** match the server deployment path exactly. If the app is ever moved to root (`/`), update this value.
