# M10 — Steampunk Polish

**Status:** ✅ Complete  
**Depends on:** [M9 — Randomize](m9-randomize.md) (all features complete)  
**Unlocks:** nothing — this is the final milestone

---

## Goal

Apply the full steampunk visual design from PRD Section 8 to the UI chrome. All
controls should look like aged brass instruments and steam-valve hardware. The
controls panel should be collapsible. The final app should be visually compelling
and match the wireframes.

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Load *Cinzel* (headings) and *Courier Prime* (readouts) from Google Fonts via `<link>` in `index.html` | ✅ |
| 2 | Apply `--font-heading` and `--font-mono` tokens to all headings, labels, and numeric readouts in CSS | ✅ |
| 3 | Style the page background and controls panel with `--color-soot` and `--color-brass` tokens; add subtle CSS repeating-dot pattern to emulate riveted metal | ✅ |
| 4 | Style panel borders and section dividers as thick strapping lines (3px solid `--color-brass`, slight border-radius) | ✅ |
| 5 | Style all `<button>` elements — chunky bevelled look using `box-shadow` inset, `--color-brass-bright` on hover, depressed state on `:active` | ✅ |
| 6 | Style slider tracks as gauge rails — thin `--color-brass` background with `--color-soot` fill | ✅ |
| 7 | Style slider thumbs as valve/gauge knobs — circular, `--color-brass-bright`, slight shadow | ✅ |
| 8 | Style toggle switches as physical lever switches — CSS-only, `--color-brass` off / `--color-copper` on | ✅ |
| 9 | Add cogwheel/gear SVG icon to "Load Image" button | ✅ |
| 10 | Add valve/gauge SVG icon to "Download" button | ✅ |
| 11 | Add lightning bolt SVG icon to "Surprise Me" button | ✅ |
| 12 | Add bellows/accordion SVG icon to the panel collapse toggle | ✅ |
| 13 | Implement collapsible panel — toggle button hides/shows the panel; canvas expands to full window width when panel is hidden; animate with CSS `transition: width` | ✅ |
| 14 | Ensure layout is correct with panel open: canvas square + fixed-width panel side by side | ✅ |
| 15 | Ensure layout is correct with panel collapsed: canvas fills full width | ✅ |
| 16 | Cross-browser visual check in Firefox and Chrome | ⬜ |
| 17 | Verify `python make.py build`, `python make.py lint`, and `python make.py test` all pass | ✅ |

---

## Manual Test Checklist

- [ ] Fonts load correctly (Cinzel visible on title/labels; Courier Prime on value readouts)
- [ ] Dark soot background with brass panel borders visible
- [ ] Buttons have a physical, 3D-pressed appearance on click
- [ ] Sliders look like instrument gauges (rail + knob styling)
- [ ] Toggle switches look like lever switches
- [ ] All four buttons have their icons
- [ ] Panel collapse toggle hides the controls; canvas expands to fill width
- [ ] Panel re-expand toggle restores the side-by-side layout
- [ ] Collapse/expand has a smooth CSS animation (not a jump)
- [ ] Steampunk styling does **not** appear inside the WebGL canvas (canvas output unaffected)
- [ ] App looks correct in both Firefox and Chrome

---

## Notes

- All steampunk styling must use CSS custom properties only — no hardcoded colour
  values outside `style/main.css`.
- SVG icons can be inline SVG in Leptos `view!` macros or referenced as separate
  asset files in `assets/icons/`. Keep them simple (single-colour, scalable).
- The collapsible panel toggle should persist its state via the `AppState` signal
  (add `panel_open: RwSignal<bool>`) so it can be read by both the panel and
  the canvas layout calculation.
- Canvas `width` and `height` attributes must be kept at 800×800 always — CSS
  can make it visually larger or smaller for display, but the backing buffer must
  stay 800×800 or the WebGL output will be wrong.
