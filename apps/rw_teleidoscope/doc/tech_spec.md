# rw_teleidoscope — Technical Specification

**Status:** Draft  
**Last updated:** 2026-03-06  
**Reference:** [Product Requirements Document](prd.md)

---

## 1. Stack

| Concern | Technology |
|---|---|
| Language | Rust (edition 2021) |
| Compile target | `wasm32-unknown-unknown` |
| UI / reactivity | Leptos 0.8 (CSR) |
| GPU rendering | WebGL 2 via the `glow` crate |
| Build toolchain | Trunk |
| Deployment base | `/rw_teleidoscope/` |
| Styling | Plain CSS, custom properties in `style/main.css` |

Rendering is **event-driven**: the WebGL canvas re-renders only when a Leptos
signal changes or the user drags a canvas control. There is no continuous rAF
loop unless animation mode is added in a future milestone.

---

## 2. Cargo.toml

```toml
[package]
name = "rw_teleidoscope"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos             = { version = "0.8", features = ["csr"] }
glow               = { version = "0.13", default-features = false }
wasm-bindgen       = "0.2"
wasm-bindgen-futures = "0.4"
web-sys            = { version = "0.3", features = [
  # DOM / Canvas
  "Window", "Document", "HtmlCanvasElement", "HtmlVideoElement",
  "HtmlInputElement", "CanvasRenderingContext2d", "Element",
  # File / Camera
  "File", "FileList", "FileReader", "Blob", "Url",
  "MediaDevices", "MediaStream", "MediaStreamConstraints",
  "MediaStreamTrack",
  # Events
  "Event", "MouseEvent", "PointerEvent", "DragEvent",
  "ProgressEvent", "EventTarget",
  # WebGL (used only to obtain the raw context for glow)
  "WebGl2RenderingContext",
  # Misc
  "Navigator", "Location", "Performance",
] }
js-sys             = "0.3"
gloo-events        = "0.2"
console_error_panic_hook = "0.1"

[dev-dependencies]
wasm-bindgen-test  = "0.3"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
```

`glow` wraps the raw `WebGl2RenderingContext` in a safe, ergonomic API.
The raw context is obtained once via `web-sys` and handed to `glow::Context::from_webgl2_context`.

---

## 3. Trunk.toml

```toml
[build]
target     = "index.html"
dist       = "dist"
public_url = "/rw_teleidoscope/"

[watch]
ignore = ["dist", "doc"]
```

All asset paths (WASM binary, JS glue, CSS, and shader files) are injected by
Trunk as absolute paths rooted at `/rw_teleidoscope/`, matching the deployment
subdirectory. Shader `.glsl` files live in `assets/shaders/` and are declared
with `<link data-trunk rel="copy-dir" href="./assets/shaders"/>` in `index.html`
so Trunk copies them to `dist/assets/shaders/` verbatim. `shader.rs` fetches
them at WASM startup via a JS `fetch()` call before the first render.

---

## 4. Source Module Layout

```
src/
├── lib.rs              # WASM entry point; #[wasm_bindgen(start)]; lint attrs
├── app.rs              # Root App component; provides AppState via context
├── routing.rs          # Hash-based Route enum (no leptos_router)
│
├── state/
│   ├── mod.rs
│   ├── params.rs       # KaleidoscopeParams — all RwSignals for renderer uniforms
│   └── app_state.rs    # AppState — image-loaded flag, camera-open flag, etc.
│
├── components/
│   ├── mod.rs
│   ├── header.rs       # App title + Load Image + Use Camera buttons
│   ├── controls_panel.rs  # Collapsible panel; assembles all sub-controls
│   ├── canvas_view.rs  # <canvas> element + WebGL init + signal→redraw Effect
│   ├── camera_overlay.rs  # getUserMedia overlay, capture, error state
│   └── export_menu.rs  # Download format picker + trigger
│
├── renderer/
│   ├── mod.rs          # Re-exports; Renderer struct
│   ├── context.rs      # Obtain WebGL2 context from canvas NodeRef; wrap with glow
│   ├── shader.rs       # GLSL source strings; compile/link program
│   ├── texture.rs      # Upload ImageData / video frame → GL texture
│   ├── uniforms.rs     # Uniform location cache; upload KaleidoscopeParams → GPU
│   └── draw.rs         # draw() — bind texture, upload uniforms, draw quad
│
├── camera.rs           # getUserMedia, live preview, snapshot capture
└── utils.rs            # Shared helpers (e.g. f32 clamping, JS promise wrappers)
```

---

## 5. State Architecture

### 5.1 KaleidoscopeParams

All kaleidoscope parameters live as `RwSignal`s on a `KaleidoscopeParams` struct,
provided via Leptos context from the root `App`. Every UI control reads and writes
its corresponding signal directly.

```rust
/// All parameters that drive the WebGL renderer.
/// Provided via context; access with `expect_context::<KaleidoscopeParams>()`.
#[derive(Clone, Copy)]
pub struct KaleidoscopeParams {
    // Symmetry
    pub segments:   RwSignal<u32>,       // 2–10
    pub rotation:   RwSignal<f32>,       // 0.0–360.0 degrees
    pub zoom:       RwSignal<f32>,       // 0.1–4.0
    pub center:     RwSignal<(f32, f32)>,// normalised 0.0–1.0; updated by canvas PointerEvent drag

    // Effects (0.0 = off, 1.0 = full)
    pub spiral:     RwSignal<f32>,
    pub radial_fold: RwSignal<f32>,
    pub lens:       RwSignal<f32>,
    pub ripple:     RwSignal<f32>,
    pub mobius:     RwSignal<bool>,
    pub recursive_depth: RwSignal<u32>, // 0–3

    // Color transforms
    pub hue_shift:  RwSignal<f32>,       // 0.0–360.0
    pub saturation: RwSignal<f32>,       // 0.0–2.0  (1.0 = unchanged)
    pub brightness: RwSignal<f32>,       // 0.0–2.0
    pub posterize:  RwSignal<u32>,       // 0 = off, 2–16 = levels
    pub invert:     RwSignal<bool>,
}
```

### 5.2 AppState

```rust
/// Top-level application state (non-render concerns).
#[derive(Clone, Copy)]
pub struct AppState {
    pub image_loaded:  RwSignal<bool>,
    pub camera_open:   RwSignal<bool>,
    pub camera_error:  RwSignal<Option<String>>,
}
```

### 5.3 Signal → Render trigger

A single `Effect` in `canvas_view.rs` tracks all `KaleidoscopeParams` signals.
When any signal changes it calls `renderer.draw(params_snapshot)`.
The renderer is stored as `Rc<RefCell<Option<Renderer>>>` outside the signal
system (it contains a `glow::Context` which is `!Send`).

```rust
// In CanvasView component — sketch only
Effect::new(move |_| {
    let p = params.snapshot(); // reads all signals, registers all as deps
    if let Some(renderer) = renderer_ref.borrow().as_ref() {
        renderer.draw(&p);
    }
});
```

---

## 6. Rendering Pipeline

### 6.1 Overview

```
Source texture (image or camera snapshot)
        │
        ▼
┌───────────────────────────────────┐
│  Fragment Shader — Main Pass      │
│                                   │
│  1. Compute fragment polar coords │
│  2. Apply lens / ripple warp      │
│  3. Apply spiral twist            │
│  4. Apply mirror symmetry fold    │
│  5. Apply radial folding          │
│  6. Sample source texture         │
│  7. Apply Möbius segment flip     │
│  8. Apply color transforms        │
└───────────────────────────────────┘
        │
        ▼  (if recursive_depth > 0)
┌───────────────────────────────────┐
│  Framebuffer Pass(es)             │
│  Output of main pass fed back in  │
│  as input texture; repeated up    │
│  to 3 times.                      │
└───────────────────────────────────┘
        │
        ▼
   Canvas display
```

### 6.2 Geometry

A single full-screen quad (two triangles covering clip space −1..1) is drawn.
No vertex data changes between frames — the vertex buffer is uploaded once at
init. The fragment shader does all the work per-pixel.

### 6.3 Shader Uniforms

| Uniform | Type | Description |
|---|---|---|
| `u_image` | `sampler2D` | Source texture (image or camera snapshot) |
| `u_prev` | `sampler2D` | Previous recursive pass output (FBO texture) |
| `u_resolution` | `vec2` | Canvas size in pixels (800, 800) |
| `u_segments` | `int` | Mirror segment count (2–10) |
| `u_rotation` | `float` | Pattern rotation in radians |
| `u_zoom` | `float` | Source sampling scale |
| `u_center` | `vec2` | Center of symmetry (normalised 0..1) |
| `u_spiral` | `float` | Spiral twist intensity (0–1) |
| `u_radial_fold` | `float` | Radial fold intensity (0–1) |
| `u_lens` | `float` | Lens distortion intensity (0–1) |
| `u_ripple` | `float` | Angular ripple intensity (0–1) |
| `u_mobius` | `bool` | Möbius segment flip on/off |
| `u_recursive_depth` | `int` | Recursive reflection passes (0–3) |
| `u_hue_shift` | `float` | Hue rotation in degrees (0–360) |
| `u_saturation` | `float` | Saturation multiplier (0–2) |
| `u_brightness` | `float` | Brightness multiplier (0–2) |
| `u_posterize` | `int` | Posterize levels (0 = off, 2–16) |
| `u_invert` | `bool` | Colour inversion |

### 6.4 Fragment Shader Algorithm (pseudocode)

```glsl
// 1. Map fragment to polar coordinates centred on u_center
vec2 uv   = (fragCoord / u_resolution) - u_center;
float r   = length(uv);
float a   = atan(uv.y, uv.x) + u_rotation;

// 2. Lens distortion (barrel warp)
if (u_lens > 0.0)  r = lens_warp(r, u_lens);

// 3. Angular ripple
if (u_ripple > 0.0)  a += u_ripple * sin(r * 20.0);

// 4. Spiral twist
a += u_spiral * r * TAU;

// 5. Mirror symmetry fold — reflect angle into [0, PI/segments]
float seg_angle = PI / float(u_segments);
a = mod(a, 2.0 * seg_angle);
if (a > seg_angle)  a = 2.0 * seg_angle - a;  // fold

// 6. Möbius flip — invert every other segment
if (u_mobius)  { /* flip alternate segments */ }

// 7. Radial fold
if (u_radial_fold > 0.0)  r = radial_fold(r, u_radial_fold);

// 8. Reconstruct Cartesian UV, apply zoom, sample texture
vec2 sample_uv = polar_to_uv(r, a) * u_zoom + 0.5;
vec4 colour    = texture(u_image, sample_uv);

// 9. Color transforms
colour = hue_rotate(colour, u_hue_shift);
colour.rgb *= u_brightness;
colour.rgb  = saturate_rgb(colour.rgb, u_saturation);
if (u_posterize > 0)  colour = posterize(colour, u_posterize);
if (u_invert)         colour.rgb = 1.0 - colour.rgb;

fragColor = colour;
```

### 6.5 Recursive Reflection

When `u_recursive_depth > 0`:

1. Render the main pass to a `glow` framebuffer object (FBO) at 800×800.
2. Bind the FBO colour attachment as `u_prev`.
3. Re-run the pass with `u_image = u_prev`, repeating up to 3 times.
4. Final pass renders directly to the canvas default framebuffer.

The FBO texture is allocated once at init and reused across frames.

---

## 7. Image Input Pipeline

Both file and camera paths produce the same output: an uploaded `glow` texture
bound to texture unit 0 as `u_image`.

### 7.1 File Input

```
<input type="file"> change event
  → FileReader.readAsArrayBuffer()
  → ProgressEvent "load"
  → decode bytes → draw to offscreen <canvas> (800×800, cropped/scaled)
  → ctx.get_image_data() → ImageData (always 800×800)
  → texture::upload_image_data(gl, &image_data)
  → AppState.image_loaded.set(true)
  → params Effect fires → renderer.draw()
```

### 7.2 Camera Snapshot

```
"Use Camera" button clicked
  → AppState.camera_open.set(true)
  → camera_overlay.rs mounts <video> element
  → navigator.mediaDevices().getUserMedia({ video: true })
      Ok(stream)  → video.set_src_object(stream)
      Err(e)      → AppState.camera_error.set(Some(message))

"Capture" button clicked
  → draw video frame onto offscreen <canvas> (800×800, cropped/scaled)
  → canvas.get_context("2d") → drawImage(video)
  → ctx.get_image_data() → ImageData (always 800×800)
  → texture::upload_image_data(gl, &image_data)
  → stop all MediaStreamTracks
  → AppState.camera_open.set(false)
  → AppState.image_loaded.set(true)
  → params Effect fires → renderer.draw()
```

---

## 8. Component Responsibilities

| Component | Responsibility |
|---|---|
| `App` | Create `KaleidoscopeParams` and `AppState`; provide via context; render layout |
| `Header` | Title; Load Image button (file picker trigger); Use Camera button |
| `ControlsPanel` | Collapsible wrapper; contains all sliders, toggles, action buttons |
| `CanvasView` | Render `<canvas>`; initialise `glow::Context`; register signal→render `Effect`; handle `PointerEvent` drag to update `center` signal |
| `CameraOverlay` | Conditional on `AppState.camera_open`; manages video element and capture flow |
| `ExportMenu` | Format picker dropdown; `canvas.toBlob()` → download link |

---

## 9. Renderer Struct

```rust
pub struct Renderer {
    gl:              glow::Context,
    program:         glow::Program,
    vao:             glow::VertexArray,
    source_texture:  Option<glow::Texture>,
    fbo_texture:     glow::Texture,    // for recursive passes
    fbo:             glow::Framebuffer,
    uniform_locs:    UniformLocations, // cached at program link time
}

impl Renderer {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> { ... }
    pub fn upload_image(&mut self, image_data: &ImageData) { ... }
    pub fn draw(&self, params: &ParamsSnapshot) { ... }
}
```

`Renderer` is stored as `Rc<RefCell<Option<Renderer>>>` in `CanvasView`. It is
`!Send` due to `glow::Context`. It must not be placed inside a Leptos `RwSignal`.

---

## 10. Camera Module

```rust
// camera.rs

/// Request camera access and return the active MediaStream.
/// Errors are human-readable strings suitable for display in the UI.
pub async fn request_camera() -> Result<web_sys::MediaStream, String>

/// Capture a single frame from a live <video> element into an ImageData.
/// Draws to an offscreen 800×800 canvas (crop/scale to fit), always returns
/// exactly 800×800 pixels.
pub fn capture_frame(video: &HtmlVideoElement) -> Result<ImageData, String>

/// Stop all tracks on a MediaStream (releases the camera).
pub fn release_camera(stream: &web_sys::MediaStream)
```

---

## 11. CSS / Theming

All visual design tokens are CSS custom properties in `style/main.css`:

```css
:root {
  --color-brass:      #8B6914;
  --color-brass-bright: #D4A017;
  --color-copper:     #4A7C59;
  --color-soot:       #1A1A1A;
  --color-parchment:  #C4A35A;
  --color-ivory:      #F5F0E0;

  --font-heading:     'Cinzel', serif;         /* Victorian / mechanical */
  --font-mono:        'Courier Prime', monospace; /* instrument readouts */

  --panel-width:      260px;
  --canvas-size:      800px;
  --border-width:     2px;
}
```

Trunk loads the CSS via `<link data-trunk rel="css" href="./style/main.css"/>` in
`index.html`. No external UI component library.

---

## 12. Build & Lint Commands

`make.py` must be created at the project root during scaffolding. It follows the
same pattern as all sibling apps (`rw_mixit`, `rw_chess`, etc.):

```python
#!/usr/bin/env python3
"""
Build script for rw_teleidoscope.

Usage:
    python make.py <target>

Targets:
    build   Debug WASM build (trunk build)
    test    Run unit tests (cargo test) + WASM tests (wasm-pack test --headless --firefox)
    dist    Release WASM build (trunk build --release)
    lint    Run clippy for the WASM target (zero warnings enforced)
    help    Show this message
"""

import sys
import subprocess
from pathlib import Path

ROOT = Path(__file__).parent


def _run(*cmd):
    subprocess.run(cmd, cwd=ROOT, check=True)


def build():
    _run("trunk", "build")


def test():
    _run("cargo", "test")
    _run("wasm-pack", "test", "--headless", "--firefox")


def dist():
    _run("trunk", "build", "--release")


def lint():
    _run("cargo", "clippy", "--target", "wasm32-unknown-unknown", "--", "-D", "warnings")


def help():
    print(__doc__)


if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "help"
    fn = globals().get(target)
    if not callable(fn) or target.startswith("_"):
        available = [k for k, v in globals().items() if callable(v) and not k.startswith("_")]
        print(f"Unknown target: '{target}'. Available: {', '.join(sorted(available))}")
        sys.exit(1)
    fn()
```

Quick-reference:

```bash
python make.py build    # trunk build (debug)
python make.py dist     # trunk build --release  →  dist/
python make.py test     # cargo test + wasm-pack test --headless --firefox
python make.py lint     # clippy --target wasm32-unknown-unknown -D warnings
```

---

## 13. Testing Strategy

The project uses three tiers of tests, each with a different scope and runner:

### Tier 1 — Native unit tests (`#[test]`)

Run with `cargo test` (no browser, no WASM toolchain required).

Suitable for any **pure Rust logic** that has no browser or WebGL dependency.
Extract such logic into free functions in `utils.rs` or a dedicated module so
it can be tested natively.

| What | Module |
|---|---|
| `cover_rect` (CSS cover algorithm) | `src/utils.rs` |
| `is_accepted_image_type` (MIME validation) | `src/utils.rs` |
| Polar coordinate / mirror fold math | `src/utils.rs` |
| Color transform math (hue, saturation, posterize) | `src/utils.rs` |
| URL routing round-trips | `src/routing.rs` |

**Rule:** any non-trivial formula or pure function **must** have a native unit
test before being ported to GLSL or wired into a component.

---

### Tier 2 — Browser wasm_bindgen tests (`#[wasm_bindgen_test]`, low-level)

Run with `wasm-pack test --headless --firefox` (Firefox required).  
File: `tests/m3_image_input.rs` and per-milestone `tests/mN_*.rs` files.

Suitable for **isolated WebGL / web-sys API calls** that need a real browser
but do not require a mounted component tree.

| What | Why browser needed |
|---|---|
| `HtmlImageElement` creation | web-sys DOM |
| Offscreen canvas → `ImageData` | Canvas 2D API |
| `texture::upload_image_data` | WebGL 2 |
| `resize_to_800` output dimensions | Canvas 2D + async onload |
| Camera API error paths (M7) | `getUserMedia` |

**Rule:** each milestone that touches WebGL or browser APIs should add at
least one Tier 2 test covering the lowest-level happy path.

---

### Tier 3 — Browser integration tests (`#[wasm_bindgen_test]`, component-level)

Run with `wasm-pack test --headless --firefox`.  
File: `tests/integration.rs`.

Suitable for **end-to-end wiring** — mounting a Leptos component (or the full
`App`) into the browser DOM and asserting observable effects: DOM structure,
reactive signal → DOM updates, full data pipelines.

**Patterns to follow** (see `tests/integration.rs` for working examples):

```rust
// Import path — NOT leptos::mount_to
use leptos::mount::mount_to;

// Give each test its own DOM container to avoid cross-test pollution.
fn fresh_container() -> web_sys::HtmlElement { ... }

// Yield to the microtask queue so Leptos effects can flush.
async fn tick() {
    wasm_bindgen_futures::JsFuture::from(
        js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL)
    ).await.unwrap();
}

// Scope DOM queries to the container, not the whole document.
container.query_selector(".foo").unwrap()
```

**Why integration tests are possible here:** shaders are embedded with
`include_str!()` (not fetched at runtime), so `Renderer::new()` is
synchronous and succeeds in the wasm-pack test environment, which does not
serve static asset files.

#### What should have an integration test

Add an integration test when a feature involves **reactive wiring between a
signal and a visible DOM change**, or when the correct behaviour requires
multiple components to be mounted together.  Concrete triggers:

| Situation | Integration test to write |
|---|---|
| A new signal gates a DOM element's visibility | Mount component, set signal, tick, assert element shown/hidden |
| A user action sets a signal that causes a redraw | Dispatch event or call function, tick, assert DOM/canvas state |
| A new component is added to `App` | Smoke-test that `App` still mounts and contains expected DOM landmarks |
| A new pipeline (file → GPU → signal) is wired | Drive the pipeline programmatically, assert terminal state |
| An error path should show UI feedback | Trigger the error, assert error message element appears |

**Do not** write integration tests for pure math, individual WebGL calls,
or anything already covered by a Tier 1 or Tier 2 test.  Integration tests
are expensive to maintain — keep them focused on wiring, not logic.

---

### Running all tests

```bash
python make.py test   # cargo test (Tier 1) + wasm-pack test --headless --firefox (Tier 2 & 3)
```

`cargo test` alone runs only Tier 1 (browser tests are gated with
`#![cfg(target_arch = "wasm32")]` and are silently skipped on native).

```bash
cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings
```

Run clippy with `--tests` to catch type errors in browser test files before
running the full browser suite.

---

## 14. Resolved Design Decisions

Previously open questions; recorded here for traceability.

| # | Topic | Decision |
|---|---|---|
| D1 | GLSL shader file organisation | **Separate `.glsl` files via Trunk asset pipeline.** Placed in `assets/shaders/`. Trunk copies them to `dist/`; `shader.rs` fetches them at runtime via `fetch()`. Easier to edit with editor syntax highlighting. |
| D2 | Image resize on upload | **Downscale to 800×800 on CPU before GPU upload.** Both the file input and camera capture paths pass through the same resize step (draw to offscreen 800×800 `<canvas>`, then `getImageData`). Keeps GPU texture memory predictable. |
| D3 | Drag UX for center point | **Canvas drag only.** `PointerEvent` listeners on the `<canvas>` element update the `center` `RwSignal` on `pointermove` while pointer is held. No X/Y sliders — keeps the controls panel uncluttered. |
