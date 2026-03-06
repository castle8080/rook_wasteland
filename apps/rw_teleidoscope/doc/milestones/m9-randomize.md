# M9 — Randomize

**Status:** ⬜ Pending  
**Depends on:** M5, M6 (all effects and color transforms implemented)  
**Unlocks:** [M10 — Steampunk Polish](m10-steampunk-polish.md)

---

## Goal

A "Surprise Me" button sets all kaleidoscope parameters to randomised values,
producing a visually interesting and distinct result every press. Randomization
must cover symmetry, effects, and color. The result must be different from the
previous press (no repeated-looking outputs).

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Implement `state::randomize(params: KaleidoscopeParams)` function — sets all signals to randomised values via `js_sys::Math::random()` | ⬜ |
| 2 | Randomize **segments** — random integer 2–10 | ⬜ |
| 3 | Randomize **rotation** — random f32 0.0–360.0 | ⬜ |
| 4 | Randomize **center** — random (f32, f32) both in 0.2–0.8 (avoid extreme edges) | ⬜ |
| 5 | Randomize **zoom** — random f32 0.3–2.5 | ⬜ |
| 6 | Randomize **effects** — pick one or two effects at random intensity 0.2–1.0; set others to 0.0 | ⬜ |
| 7 | Randomize **Möbius** — 30% chance of being enabled | ⬜ |
| 8 | Randomize **recursive depth** — 0 with 60% probability, 1–2 with 40% | ⬜ |
| 9 | Randomize **hue shift** — random f32 0.0–360.0 | ⬜ |
| 10 | Randomize **saturation** — random f32 0.8–1.8 (keep it roughly usable) | ⬜ |
| 11 | Randomize **brightness** — random f32 0.7–1.5 | ⬜ |
| 12 | Keep posterize and invert **off** by default in randomize (they can be extreme) | ⬜ |
| 13 | Add "Surprise Me" button to the controls panel (above the Download button per wireframe); wire to `randomize(params)` | ⬜ |
| 14 | Verify `python make.py build` and `python make.py lint` pass | ⬜ |

---

## Manual Test Checklist

- [ ] Load an image → click "Surprise Me" → pattern changes noticeably
- [ ] Click "Surprise Me" 5 times in a row → each result looks visually distinct
- [ ] After randomize, all sliders in the panel reflect the new values
- [ ] Randomized output is never all-black or all-white (usable ranges enforced)
- [ ] No console errors

---

## Notes

- `js_sys::Math::random()` returns an `f64` in `[0.0, 1.0)`. Scale and cast to
  the appropriate type for each parameter.
- Slider UI must update to reflect randomised signal values — this happens
  automatically because sliders read their signal values reactively.
- The constraint "different from the previous press" is satisfied probabilistically
  by the ranges chosen — no need for a history check in v1.
- Posterize and invert are excluded from randomize because they drastically reduce
  visual richness, making the output look broken rather than artistic.
