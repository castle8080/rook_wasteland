# M6 — Color Transforms

**Status:** ⬜ Pending  
**Depends on:** [M4 — Mirror Symmetry Core](m4-mirror-symmetry.md)  
**Unlocks:** [M10 — Steampunk Polish](m10-steampunk-polish.md) (after M9)

---

## Goal

Add all five color transform operations to the fragment shader as post-processing
steps applied after the kaleidoscope sample. Wire each to a control in the panel.
All transforms are stackable and fully independent.

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | **Hue rotation** — implement `hue_rotate(colour, degrees)` GLSL function using RGB→HSV→RGB conversion; add `u_hue_shift` uniform (0.0–360.0) | ⬜ |
| 2 | **Saturation** — implement `saturate_rgb(colour, amount)` GLSL function (interpolate between luminance and full colour); add `u_saturation` uniform (0.0–2.0) | ⬜ |
| 3 | **Brightness** — multiply `colour.rgb *= u_brightness`; add `u_brightness` uniform (0.0–2.0) | ⬜ |
| 4 | **Posterize** — implement `posterize(colour, levels)` GLSL function: `floor(colour * levels) / levels`; add `u_posterize` int uniform (0 = off, 2–16 levels) | ⬜ |
| 5 | **Invert** — `if (u_invert) colour.rgb = 1.0 - colour.rgb`; add `u_invert` bool uniform | ⬜ |
| 6 | Add all five uniforms to `UniformLocations` and `upload()` in `uniforms.rs` | ⬜ |
| 7 | Add all five fields to `KaleidoscopeParams` and `ParamsSnapshot` | ⬜ |
| 8 | Add hue slider (0–360) to the color section of the controls panel | ⬜ |
| 9 | Add saturation slider (0–200%) to panel | ⬜ |
| 10 | Add brightness slider (0–200%) to panel | ⬜ |
| 11 | Add posterize control to panel — a slider from 0 (off) to 16; display "Off" when 0 | ⬜ |
| 12 | Add invert toggle to panel | ⬜ |
| 13 | Write `#[test]` native unit tests for hue rotate and posterize math (pure Rust equivalents in `utils.rs`) | ⬜ |
| 14 | Verify `python make.py build`, `python make.py lint`, and `python make.py test` all pass | ⬜ |

---

## Manual Test Checklist

- [ ] Hue slider at 0 → original colours; at 180 → colours inverted around spectrum
- [ ] Saturation at 0 → greyscale output; at 2.0 → hyper-saturated
- [ ] Brightness at 0 → black canvas; at 2.0 → blown-out whites
- [ ] Posterize at 2 → clearly banded stained-glass look; at 16 → subtle banding
- [ ] Posterize slider shows "Off" at position 0
- [ ] Invert toggle → colours complement correctly
- [ ] All transforms stack correctly (hue + posterize + invert simultaneously)
- [ ] No console errors

---

## Notes

- Apply color transforms in this shader order (after texture sample): hue → sat →
  brightness → posterize → invert. This matches the pipeline in tech spec Section 6.4.
- HSV-based hue rotate is the most accurate method. Standard GLSL hue rotate
  matrix is an alternative if HSV conversion is too expensive.
- Posterize should clamp input to 0..1 before flooring to avoid artefacts on
  HDR-ish textures.
- `u_posterize = 0` means off — check for this in the shader with `if (u_posterize > 1)`.
