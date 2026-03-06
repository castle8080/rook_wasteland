# M5 — Visual Effects

**Status:** ⬜ Pending  
**Depends on:** [M4 — Mirror Symmetry Core](m4-mirror-symmetry.md)  
**Unlocks:** [M10 — Steampunk Polish](m10-steampunk-polish.md) (after M9)

---

## Goal

Add all six "weird effect" transforms from the PRD to the fragment shader, wire
each to a slider or toggle in the controls panel, and implement the FBO-based
recursive reflection. Each effect must be independently toggleable (0 = off).

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | **Spiral twist** — add `u_spiral` uniform; in shader, offset angle by `u_spiral * r * TAU` before the mirror fold | ⬜ |
| 2 | **Angular ripple** — add `u_ripple` uniform; offset angle by `u_ripple * sin(r * 20.0)` | ⬜ |
| 3 | **Glass sphere / lens distortion** — add `u_lens` uniform; implement barrel warp `r = r / (1.0 - u_lens * r * r)` (clamp to avoid division by zero) | ⬜ |
| 4 | **Radial folding** — add `u_radial_fold` uniform; implement `r = abs(mod(r * (1.0 + u_radial_fold * 4.0), 2.0) - 1.0)` | ⬜ |
| 5 | **Möbius mirror** — add `u_mobius` bool uniform; after mirror fold, if segment index is odd, invert `r` (or flip UV) to produce the non-Euclidean alternating flip | ⬜ |
| 6 | Add all five new uniforms to `UniformLocations` cache and `upload()` in `uniforms.rs` | ⬜ |
| 7 | Add all five new fields to `KaleidoscopeParams` and `ParamsSnapshot` | ⬜ |
| 8 | Add spiral, ripple, lens, radial fold sliders (0.0–1.0) to controls panel | ⬜ |
| 9 | Add Möbius toggle to controls panel | ⬜ |
| 10 | **Recursive reflection — FBO setup** — in `Renderer::new()`, allocate one 800×800 FBO texture + framebuffer; add `u_recursive_depth` uniform | ⬜ |
| 11 | **Recursive reflection — multi-pass draw** — in `draw()`, if `recursive_depth > 0`: render main pass to FBO, bind FBO colour as `u_prev` / re-bind as `u_image`, repeat up to depth times, then final pass to default framebuffer | ⬜ |
| 12 | Add `recursive_depth` field to `KaleidoscopeParams` and `ParamsSnapshot`; add step-slider (0–3) to controls panel | ⬜ |
| 13 | Write `#[test]` unit tests for lens warp and radial fold formulas (pure Rust equivalents in `utils.rs`) | ⬜ |
| 14 | Verify `python make.py build`, `python make.py lint`, and `python make.py test` all pass | ⬜ |

---

## Manual Test Checklist

- [ ] Spiral slider at 0 → no change; at 1.0 → visible vortex/spiral
- [ ] Ripple slider at 0 → no change; at 1.0 → wavy distortions along angle
- [ ] Lens slider at 0 → no change; at 1.0 → visible barrel/fisheye bulge at center
- [ ] Radial fold at 0 → no change; at 1.0 → concentric crystalline rings visible
- [ ] Möbius toggle off → no change; on → alternate segments flip/invert
- [ ] Recursive depth 0 → normal output; depth 1 → fractal-like doubling; depth 3 → dense intricate output
- [ ] All effects combine without crashing (spiral + lens + recursive at the same time)
- [ ] No console errors at any setting

---

## Notes

- The lens distortion clamping is important: if `u_lens * r * r >= 1.0` the denominator
  reaches zero. Guard with `max(denominator, 0.001)`.
- The FBO texture must be the same size as the canvas (800×800). Allocate with
  `RGBA8` internal format.
- For Möbius: to determine segment index, compute `floor(original_angle / (2 * seg_angle))`
  before the fold. If `mod(index, 2) == 1`, flip the reconstructed UV.
- Effects are applied in shader pipeline order (see tech spec Section 6.1):
  lens → ripple → spiral → fold → Möbius → radial fold.
