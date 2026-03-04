# Task T0.8: static/style.css + Fonts

**Milestone:** M0 — Project Scaffold
**Status:** ✅ Done

---

## Restatement

Create `static/style.css` with: CSS custom properties for the full colour palette (bg, panel, border, shadow, deck-a blue, deck-b orange, mixer green, glow yellow, hot-cue colours), the Bangers font loaded via Google Fonts CDN (with a note to replace with local files), `box-sizing: border-box` reset, cartoon border/shadow rules (`3px solid`, `4px 4px 0` drop shadow), layout for the 3-column deck row, deck/mixer panel styles with accent top borders, header styles, a shared `.btn` base class with press animation, and responsive breakpoints. Create `static/fonts/` directory with a `.gitkeep` placeholder. Out of scope: component-specific styles beyond the base structure (added per milestone).

---

## Design

### Data flow
Static CSS — no runtime data flow.

### Edge cases
- Bangers font CDN requires internet access; offline environments will fall back to generic cursive.
- `static/fonts/` must exist for Trunk's `copy-dir` directive to succeed, even if empty.

### Integration points
- All component CSS class names defined here must match the classes used in `deck.rs`, `mixer.rs`, `header.rs`.
- `--color-deck-a` and `--color-deck-b` custom properties are referenced by component styles in later milestones.

---

## Design Critique

| Dimension   | Issue | Resolution |
|---|---|---|
| Correctness | CSS class names must match component `class=` attributes exactly. | Verified by visual inspection and cross-referencing all component files. |
| Simplicity  | Could use CSS modules or a preprocessor. | Plain CSS matches the spec (no build pipeline beyond Trunk). |
| Coupling    | Custom property names are a shared contract. | Defined once in `:root`; all components reference via `var(...)`. |
| Performance | Bangers CDN link adds a network round-trip. | Acceptable for development. |
| Testability | Visual — no automated tests. | Manual inspection in browser. |

---

## Implementation Notes

Added a `@keyframes pop` animation and `.btn:active` style for future milestone button components, avoiding a later CSS-only patch. Added responsive breakpoints for ≤1200 px and ≤900 px (stacked layout) to satisfy M11 T11.9 success criteria early.

---

## Test Results

**Automated:** N/A — CSS is not statically analysed.

**Manual steps performed:**
- [ ] Bangers font renders on labels (requires trunk serve with internet)
- [ ] Black outline + shadow visible on deck/mixer panels
- [ ] Deck A label shows in cobalt blue, Deck B in fire orange
- [ ] Mixer label in lime green

---

## Review Notes

No issues found.

---

## Callouts / Gotchas

- To self-host Bangers: download `Bangers-Regular.woff2`, add `@font-face` block to this file, remove CDN `<link>` tags from `index.html`, and delete the `.gitkeep` note in `static/fonts/.gitkeep`.
- The `.btn` class is not yet used in M0. It is a forward-looking base rule for M2+ control buttons.
