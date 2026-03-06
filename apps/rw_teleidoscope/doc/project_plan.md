# rw_teleidoscope — Project Plan

**Status:** In Progress  
**Reference docs:** [PRD](prd.md) · [Tech Spec](tech_spec.md) · [Wireframes](wireframes.md)

---

## Overview

The project is divided into 10 milestones. Each milestone produces a working,
manually testable increment. Milestones must be completed in order — each builds
on the previous one.

Implementation is underway. M1–M7 are complete.

---

## Milestones

| # | Milestone | Description | Status |
|---|---|---|---|
| M1 | [Project Scaffold](milestones/m1-scaffold.md) | Cargo.toml, index.html, Trunk.toml, make.py, empty app shell. Trunk build succeeds and page loads. | ✅ Complete |
| M2 | [WebGL Canvas & Basic Renderer](milestones/m2-webgl-renderer.md) | glow context, full-screen quad, .glsl files via Trunk, solid colour output. WebGL works end-to-end. | ✅ Complete |
| M3 | [Image Input & Texture Display](milestones/m3-image-input.md) | File picker, drag-and-drop, 800×800 resize, texture upload, passthrough shader shows image on canvas. | ✅ Complete |
| M4 | [Mirror Symmetry Core](milestones/m4-mirror-symmetry.md) | Polar coords, mirror fold, segments/rotation/zoom controls, canvas drag for center. Real kaleidoscope visible. | ✅ Complete |
| M5 | [Visual Effects](milestones/m5-visual-effects.md) | Spiral twist, radial fold, lens distortion, angular ripple, Möbius mirror, recursive reflection (FBO). | ✅ Complete |
| M6 | [Color Transforms](milestones/m6-color-transforms.md) | Hue rotation, saturation, brightness, posterize, invert — all wired to panel controls. | ✅ Complete |
| M7 | [Camera Input](milestones/m7-camera-input.md) | getUserMedia, live preview overlay, capture frame, permission-denied error state. | ✅ Complete |
| M8 | [Export / Download](milestones/m8-export.md) | canvas.toBlob() download, PNG/JPEG/WebP format selector, metadata in filename. | ⬜ Pending |
| M9 | [Randomize](milestones/m9-randomize.md) | "Surprise Me" button randomizes segments, rotation, center, effects, and color transforms. | ⬜ Pending |
| M10 | [Steampunk Polish](milestones/m10-steampunk-polish.md) | Full steampunk CSS, fonts, styled sliders and buttons, collapsible panel, final layout. | ⬜ Pending |

---

## Dependency Order

```
M1 (Scaffold)
  └─ M2 (WebGL Renderer)
       └─ M3 (Image Input)
            └─ M4 (Mirror Symmetry)   ← core kaleidoscope complete here
                 ├─ M5 (Visual Effects)
                 ├─ M6 (Color Transforms)
                 ├─ M7 (Camera Input)
                 ├─ M8 (Export)
                 └─ M9 (Randomize)
                      └─ M10 (Steampunk Polish)
```

M5–M9 all depend on M4 but are otherwise independent of each other and could
be worked on in parallel if multiple contributors are available.

---

## Status Key

| Symbol | Meaning |
|---|---|
| ⬜ | Pending — not started |
| 🔄 | In progress |
| ✅ | Complete — manual test passed |
| 🚫 | Blocked |
