# rw_teleidoscope — Product Requirements Document

**Status:** Draft  
**Last updated:** 2026-03-06

---

## 1. Problem Statement

Creative individuals want to produce beautiful, symmetrical, psychedelic patterns from their own photos — but doing so currently requires expensive desktop software or cloud-based tools that process images on a remote server. There is no simple, fast, browser-native tool that lets someone upload a photo and immediately explore kaleidoscope art without installation, sign-up, or data leaving their device.

---

## 2. Goals

1. A user can upload a photo and see a compelling kaleidoscope pattern within 5 seconds.
2. Mirror count, rotation, and center position are adjustable in real time with no perceptible lag on images up to 1024×1024.
3. The app runs entirely in-browser with zero server dependencies — no uploads, no accounts.
4. The user can download the current view as a PNG or JPEG.
5. A "Randomize" button produces a varied, visually interesting result every press.

---

## 3. Non-Goals

- No server-side processing of any kind.
- No user accounts, saved sessions, or cloud storage.
- No video file input in v1 (live camera is supported; pre-recorded video files are not).
- No audio-reactive transformations in v1.
- No animation / auto-evolve mode in v1.
- No ZIP batch export in v1.
- WebGL is not required for v1 — WASM pixel transforms are sufficient for the target image size.
- No mobile-specific touch UI in v1 (desktop browser is the primary target). *(Mobile responsive layout added in Feature 001 / M11.)*

---

## 6. Technical Architecture

**Rendering:** WebGL fragment shaders own all pixel-level computation — symmetry math, effects, and color transforms are expressed in GLSL. This is a hard v1 requirement; there is no WASM-only fallback path.

**WASM role:** Rust/WASM manages parameter state, image decoding, texture uploads to the GPU, and exposes control functions via wasm-bindgen. It does not do per-pixel work.

**Leptos role:** Reactive UI signals drive parameter changes. When a signal updates, the corresponding WebGL uniform is updated and the canvas redraws.

---

## 4. User Stories

**Core flow**

- As a user, I want to drag-and-drop (or click to upload) a photo so I can start immediately without hunting for a button.
- As a user, I want to change the number of mirror segments (2–10) and see the pattern update instantly.
- As a user, I want to drag the center of symmetry around the canvas to explore different regions of my photo.
- As a user, I want to spin the pattern with a rotation slider so I can find the most interesting angle.
- As a user, I want to adjust hue, saturation, and brightness so I can control the mood of the output.
- As a user, I want to click "Randomize" to get a surprising result when I don't know where to start.
- As a user, I want to download the current canvas as a PNG or JPEG so I can share or print it.

**Weird effects**

- As a user, I want to apply a spiral twist so the pattern spirals inward like a vortex.
- As a user, I want a glass sphere / lens distortion to make the center bulge like a fisheye.
- As a user, I want radial folding to produce crystalline ring structures.

---

## 5. Functional Requirements

### 5.1 Image Input

- FR-1: The app shall accept PNG, JPEG, and WebP image files via a file-picker button.
- FR-2: The app shall accept image files dropped onto the canvas area.
- FR-3: On load, the image shall be decoded and stored as a WebGL texture for the fragment shader.

#### Camera Input

- FR-4: The app shall provide a "Use Camera" button that requests `getUserMedia({ video: true })` permission and opens the device camera.
- FR-5: While the camera is active, a live video preview shall be shown in a small overlay on the canvas so the user can frame their shot.
- FR-6: A "Capture" button shall freeze the current camera frame and use it as the source image, closing the preview overlay.
- FR-7: A "Cancel" button shall dismiss the camera preview without changing the current source image.
- FR-8: If camera permission is denied, the app shall display a clear inline message explaining why the feature is unavailable; it shall not crash or show a browser alert.
- FR-9: The camera input path shall feed the captured frame into the same WebGL texture pipeline as FR-3, requiring no separate code path for the kaleidoscope engine.

> **Implementation note:** `HTMLVideoElement` can be passed directly to `texImage2D` for live streaming, but v1 uses the capture-then-freeze pattern (snapshot) to keep the interaction model simple. Live continuous streaming from the camera is deferred to a future version.

### 5.2 Mirror Symmetry

- FR-4: The app shall support a configurable mirror segment count from 2 to 10 (integer steps).
- FR-5: Segment count shall be adjustable via a slider; the canvas shall update on every slider change.
- FR-6: The center of symmetry shall be draggable anywhere within the canvas.
- FR-7: A rotation control (0–360°) shall spin the symmetry pattern.
- FR-8: A zoom control shall scale the source sampling region.

### 5.3 Visual / Weird Effects

Each effect is independent and can be combined. All have an intensity slider (0 = off, 1 = full):

- FR-9: **Spiral twist** — rotate sample angle proportionally to radius distance from center.
- FR-10: **Radial folding** — repeat the radius inward to produce concentric ring structures.
- FR-11: **Glass sphere distortion** — apply a radial lens warp to emulate a convex surface.
- FR-12: **Angular ripple** — add sinusoidal distortion along the sample angle.
- FR-13: **Möbius mirror** — flip alternate mirror segments for a non-Euclidean feel.
- FR-14: **Recursive reflection** — fold the kaleidoscope into itself for fractal-like density (intensity controls recursion depth, max 3).

### 5.4 Color Transforms

- FR-15: Hue rotation (0–360°) shall shift all colors around the spectrum.
- FR-16: Saturation adjustment (0–200%) shall desaturate or oversaturate the output.
- FR-17: Brightness adjustment (0–200%) shall darken or lighten the output.
- FR-18: Posterization shall reduce the number of visible colors (2–16 levels).
- FR-19: Color inversion shall complement all RGB channels.

### 5.5 Randomization

- FR-20: A "Randomize" button shall randomize: mirror count, rotation, center position, one enabled weird effect (random intensity), hue rotation, and saturation.
- FR-21: Each press shall produce a visually distinct result from the previous press.

### 5.6 Download

- FR-22: A "Download" button shall export the current canvas view as PNG.
- FR-23: A format selector shall allow choosing PNG, JPEG, or WebP for the download.
- FR-24: The filename shall include the mirror count and a timestamp (e.g. `teleidoscope-6m-20260306.png`).

---

## 6. Image Processing Pipeline

```
Source image (decoded pixel buffer)
        ↓
  Lens distortions  (glass sphere, angular ripple)
        ↓
  Geometric effects (spiral twist, radial folding, Möbius)
        ↓
  Mirror symmetry   (segment count, center, rotation, zoom)
        ↓
  Recursive fold    (if enabled)
        ↓
  Color transforms  (hue, sat, brightness, posterize, invert)
        ↓
  Canvas display
```

The WASM engine owns all pixel-level computation. The Leptos UI owns signal state. The canvas draw is triggered reactively when any signal changes.

---

## 7. UI Layout

```
┌─────────────────────────────────────────────┐
│  [Load Image]   rw_teleidoscope             │  ← header
├──────────────┬──────────────────────────────┤
│              │                              │
│  Controls    │         Canvas               │
│  panel       │   (kaleidoscope output)      │
│              │                              │
│  Segments    │                              │
│  Rotation    │                              │
│  Zoom        │                              │
│  ─────────   │                              │
│  Effects     │                              │
│  (toggles +  │                              │
│   sliders)   │                              │
│  ─────────   │                              │
│  Hue         │                              │
│  Saturation  │                              │
│  Brightness  │                              │
│  Posterize   │                              │
│  ─────────   │                              │
│  [Randomize] │                              │
│  [Download ▾]│                              │
└──────────────┴──────────────────────────────┘
```

- Controls panel: fixed width, scrollable if needed.
- Controls panel is collapsible — a toggle button hides it so the canvas fills the full window width.
- Canvas: fills remaining width, square aspect ratio preferred.
- Drag interaction for center of symmetry happens directly on the canvas.

---

## 8. Visual Design — Steampunk Aesthetic

The app's UI chrome (everything outside the canvas) shall have a **steampunk aesthetic**. The kaleidoscope output itself remains free of UI chrome styling.

### Palette
- Base tones: dark aged brass (#8B6914), oxidised copper (#4A7C59), soot black (#1A1A1A), and sepia parchment (#C4A35A) for backgrounds and panels.
- Accent highlights: bright burnished gold (#D4A017) for active controls and focus states.
- Text: off-white ivory (#F5F0E0) on dark surfaces; dark charcoal on parchment.

### Surfaces and Panels
- Control panel background: dark riveted-metal texture feel — achieved with CSS (dark base colour + subtle repeating dot or rivet pattern).
- Panel borders and dividers: thick, slightly irregular lines styled as bolted metal strapping.
- Buttons: chunky, bevelled appearance with a pressed/depressed state on click — like physical toggles or steam-valve handles.

### Typography
- Headings and labels: a serif or slab-serif font with a mechanical/Victorian character (e.g. a Google Fonts option like *Cinzel*, *Playfair Display*, or *Special Elite*).
- Body / numeric values: monospaced for readability, evoking instrument readouts.

### Sliders and Controls
- Slider tracks styled as gauge rails or pipe segments.
- Slider thumbs styled as gauge needles, valve handles, or cog-shaped knobs.
- Toggle switches styled as physical lever switches (brass + dark metal).

### Iconography
- Load image button: gear or cogwheel icon.
- Download button: pressure gauge or valve icon.
- Randomize button: lightning bolt or alchemical flask icon.
- Collapse/expand panel toggle: bellows or accordion icon.

### Implementation Notes
- All styling via plain CSS custom properties defined in `style/main.css`.
- No external UI component library — hand-crafted CSS consistent with the rest of the monorepo.
- Steampunk styling applies to the UI shell only; it must never bleed into the WebGL canvas output.

---

## 9. Out of Scope / Future Work

The following came up in requirements gathering and are deliberately deferred:

- **Animation / auto-evolve mode** — continuously varying parameters for a live screensaver-like experience.
- **Audio-reactive transformations** — mapping sound frequency bands to rotation, zoom, or segment count.
- **High-resolution / 4K export** — rendering at a resolution higher than the displayed canvas.
- **Seamless tile export** — generating a square tile for wallpaper/texture use.
- **Predefined color palettes** — quick artistic presets.
- **Segment color offset** — assigning different color shifts per mirror segment.
- **Radial/angular gradients** — color that varies by distance or angle from center.
- **Animated color cycling** — real-time hue animation.
- **Batch/ZIP export** — generating multiple random variants in one action.
- **Live camera streaming** — continuously feeding live camera frames into the kaleidoscope in real-time (v1 captures a snapshot; live streaming is deferred).
- **Mobile / touch support** — touch drag for center position. *(Implemented as Feature 001 / M11: bottom drawer, pinch zoom, responsive layout.)*

---

## 10. Open Questions

| # | Question | Decision |
|---|---|---|
| Q1 | What canvas resolution should the v1 target? | **800×800** — both display size and internal processing resolution. |
| Q2 | Should "Recursive reflection" be in v1? | **Yes — include in v1.** Max recursion depth: 3. |
| Q3 | Is WebGL required before launch? | **Yes — WebGL from the start.** The kaleidoscope math maps well to fragment shaders and the GPU parallelism is worth the added complexity. All pixel-level computation (symmetry, effects, color) goes through GLSL shaders; Rust/WASM owns parameter management and JS interop. |
| Q4 | Should the controls panel be collapsible? | **Yes — collapsible.** Panel collapses to give the canvas full width when exploring the output. |

---

## 11. Success Criteria

1. Upload a photo → see a kaleidoscope pattern → takes under 5 seconds total.
2. Dragging the center point updates the canvas with no visible stutter at 800×800.
3. Segment count, rotation, and at least hue/saturation/brightness controls all work correctly.
4. PNG download produces a file that opens correctly in a standard image viewer.
5. The "Randomize" button never produces the same-looking result twice in a row.
6. The app works entirely in-browser; no network requests are made after initial page load.
