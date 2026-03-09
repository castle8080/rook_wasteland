# Feature 001 — Mobile / Responsive Layout

## Status
Implemented

## Summary
The app currently targets desktop browsers with a fixed side-by-side layout (controls panel on the
left, 800×800 canvas on the right) and no touch-specific interaction model. This feature introduces
a fully responsive layout that adapts to narrow screens: on mobile the canvas fills the viewport,
controls slide up from a persistent bottom drawer, the canvas dynamically resizes to fill the
available space, and touch gestures (single-finger drag for center repositioning, two-finger pinch
for zoom) replace mouse interactions.

## Problem Statement
Users on phones encounter a layout that was designed for a 1024 px+ desktop viewport: the controls
panel consumes nearly half the screen, the canvas is often clipped, sliders have tiny touch targets,
and dragging the center of symmetry produces browser scroll instead of the intended canvas
interaction. There is no pinch-to-zoom on mobile. The result is that core functionality — the main
draw of the app — is effectively unusable on a phone without deliberate, frustrating workarounds.
Making the app genuinely first-class on mobile significantly expands the audience and fits well with
the app's "no install, no sign-up, open on any device" value proposition.

## Goals
- A user on a 375 px wide phone can load an image and operate all core controls without pinching
  the browser page or scrolling horizontally.
- The canvas always fills the available viewport width on mobile; the WebGL render resolution
  matches the CSS display size.
- Single-finger touch drag on the canvas repositions the center of symmetry with the same
  responsiveness as the existing mouse drag.
- Two-finger pinch on the canvas adjusts the zoom parameter in real time.
- The controls drawer opens and closes smoothly; all sliders and toggles have touch-friendly hit
  targets (minimum 44×44 CSS px per WCAG 2.5.5).
- The feature introduces no regressions on desktop — the side-by-side layout remains unchanged
  above the responsive breakpoint.

## Non-Goals
- This feature does NOT add swipe-to-change-effect gestures or any other multi-touch interactions
  beyond single-finger drag and two-finger pinch.
- This feature does NOT add orientation-lock or a landscape-specific layout variant.
- This feature does NOT add a native app wrapper (PWA manifest icons, app install prompt, etc.),
  though the viewport meta tag change is a prerequisite for PWA eligibility in the future.
- This feature does NOT change the WebGL shaders or the renderer pipeline.
- This feature does NOT change any behavior on desktop (breakpoint > 768 px).
- This feature does NOT add a camera-flip (front/rear switch) button — that is a separate camera
  feature outside this scope.

## User Stories
- As a mobile user, I want the canvas to fill my phone screen so I can see the kaleidoscope
  clearly without pinching the browser.
- As a mobile user, I want a persistent handle at the bottom of the screen so I always know how
  to open the controls.
- As a mobile user, I want to drag one finger across the canvas to move the center of symmetry,
  just like I'd drag on a desktop with a mouse.
- As a mobile user, I want to pinch two fingers on the canvas to zoom in and out of the source
  image region.
- As a mobile user, I want to tap behind the open controls drawer to dismiss it so I can see the
  full canvas again.
- As a mobile user, I want the camera preview modal to fit on my phone screen without overflow or
  horizontal scrolling.
- As a desktop user, I want the app to look and behave exactly as it does today — the responsive
  changes must not affect the desktop layout.

## Functional Requirements

### Layout
1. **Breakpoint:** The app shall switch to mobile layout at a viewport width ≤ 768 CSS px, using a
   CSS media query. Above 768 px the layout is unchanged.
2. **Canvas sizing (mobile):** On mobile, the canvas element shall be sized to fill the available
   viewport width as a square (i.e. `width: 100vw; height: 100vw`), so the kaleidoscope fills the
   screen without overflow. The WebGL viewport (`gl.viewport`) and any internal size tracking shall
   update whenever the canvas CSS size changes (e.g. on orientation change or soft keyboard
   appearance).
3. **Bottom drawer:** On mobile, the controls panel shall be rendered inside a bottom drawer. The
   drawer shall occupy the full viewport width and slide up from the bottom edge of the screen when
   opened.
4. **Drawer handle:** A persistent handle/tab strip shall remain visible at the bottom of the
   screen at all times on mobile (even when the drawer is closed), giving the user a clear
   affordance to open the controls. The strip shall display a ▲/▼ chevron indicating the
   open/closed state.
5. **Drawer open/close:** Tapping the drawer handle strip shall toggle the drawer open/closed.
   Tapping the dimmed canvas backdrop while the drawer is open shall close the drawer.
6. **Drawer state signal:** A new `drawer_open: RwSignal<bool>` shall be added to `AppState` and
   provided via Leptos context, so any component can read or toggle it.
7. **Touch targets:** On mobile, all slider tracks, toggle buttons, and action buttons shall have a
   minimum tap target of 44×44 CSS px, achieved via CSS `min-height`/`padding` adjustments scoped
   inside the mobile media query.
8. **Viewport meta tag:** `index.html` shall include
    `<meta name="viewport" content="width=device-width, initial-scale=1.0">` so mobile browsers do
    not scale the page down.
9. **No horizontal scroll:** On mobile, the app shall not produce horizontal scroll at any point
    during normal use.
10. **Desktop unchanged:** All behaviour above the 768 px breakpoint — panel width, canvas size,
    collapse/expand, drag interactions — shall remain identical to the current implementation.

### Touch interactions
11. **Touch drag (center):** The `canvas_view.rs` pointer event handlers shall work correctly with
    touch-generated pointer events (the Pointer Events API already maps touch to pointer events, but
    `preventDefault()` must be called during active touch drag to suppress native browser scroll).
    `touch-action: none` shall be set on the canvas element via CSS to suppress the browser's
    built-in scroll/pan handling.
12. **Pinch-to-zoom:** A two-pointer pinch gesture on the canvas shall update the `zoom` signal.
    The zoom delta shall be proportional to the ratio of the new pinch distance to the previous
    pinch distance (multiplicative scaling). The zoom signal shall remain clamped to its existing
    min/max range.

### Camera overlay
13. **Camera modal responsive width:** The `.camera-modal` CSS rule shall not enforce a fixed
    `min-width` that could exceed the viewport on narrow screens. The modal shall use
    `width: min(400px, 92vw)` (or equivalent) so it never causes horizontal scroll on any device.
14. **Camera video fill:** Inside the modal on mobile, the `<video>` element (`.camera-preview`)
    shall fill the full modal width so the live camera feed is as large as possible on a small
    screen.
15. **Camera action button touch targets:** The CAPTURE and CANCEL buttons inside the camera
    overlay shall meet the 44×44 CSS px minimum touch-target requirement on mobile (same as FR-7
    for other controls).

## UI / UX Notes

### Mobile layout (≤ 768 px)

```
┌──────────────────────────────────┐   ← 100vw
│  ⚙  TELEIDOSCOPE                │   ← header (compact height)
├──────────────────────────────────┤
│                                  │
│                                  │
│         WebGL canvas             │   ← square, fills remaining height
│     (touch drag = move center)   │
│     (pinch = zoom)               │
│                                  │
│                                  │
├──────────────────────────────────┤
│  ════ ⚙ CONTROLS  ▲  ═══════    │   ← persistent drawer handle strip
└──────────────────────────────────┘
```

When the drawer is open:

```
┌──────────────────────────────────┐
│  ⚙  TELEIDOSCOPE                │
├──────────────────────────────────┤
│▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒│   ← dimmed canvas (tap to close)
│▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒│
├──────────────────────────────────┤
│  ════ ⚙ CONTROLS  ▼  ═══════    │   ← handle (▼ = open, ▲ = closed)
├──────────────────────────────────┤
│  MIRRORS  ╞══●═══════╡  6       │
│  ROTATION ╞════●════╡  180°     │   ← scrollable controls, ~50vh
│  ZOOM     ╞══●═══════╡  1.0×    │
│  ── EFFECTS ──────────────────  │
│  ▣ Spiral  ▣ Radial  ▣ Lens     │
│  ...                            │
│  ── COLOR ────────────────────  │
│  HUE / SAT / BRIGHTNESS ...     │
│  ⚡ RANDOMIZE     ↓ EXPORT      │
└──────────────────────────────────┘
```

### Camera overlay on mobile
On mobile the camera modal already benefits from `max-width: 90vw`, but the `min-width: 340px`
rule must be removed/replaced so it doesn't overflow on ~320 px devices. The modal is
`position: fixed` and centered, so it behaves like a full-page dialog on small screens:

```
┌──────────────────────────────────┐
│  ══╡  📷  C A M E R A  ╞══      │
├──────────────────────────────────┤
│                                  │
│   ┌──────────────────────────┐   │
│   │                          │   │
│   │   live camera feed       │   │  ← fills modal width, 4:3
│   │   (<video> element)      │   │
│   │                          │   │
│   └──────────────────────────┘   │
│                                  │
│  ╔════════════╗ ╔═════════════╗  │
│  ║ 📷 CAPTURE ║ ║  ✕ CANCEL   ║  │  ← min 44px height
│  ╚════════════╝ ╚═════════════╝  │
└──────────────────────────────────┘
```

### Steampunk handle
The drawer handle strip should match the steampunk aesthetic: dark brass border top, riveted look,
label "CONTROLS" in the same slab-serif font as the rest of the panel. The ▲/▼ chevron indicates
open/closed state and should animate with a CSS transition.

### Accessibility
- The drawer handle must be keyboard-focusable and activatable with Enter/Space (required even
  though the primary interaction model is touch).
- The backdrop overlay should have `aria-hidden="true"` since it is a decorative interaction target.
- The drawer should have `role="region"` and `aria-label="Controls"`.

### Existing desktop wireframes
All wireframes in `doc/wireframes.md` (sections 1–7) depict the desktop layout and remain valid
above the breakpoint. New mobile wireframes (above) are additive.

## Architecture Fit

### Modules/components affected
| File | Change |
|---|---|
| `index.html` | Add `<meta name="viewport">` tag |
| `style/main.css` | Add `@media (max-width: 768px)` rules for drawer layout, canvas sizing, touch targets; CSS transition for drawer slide-up; fix `.camera-modal` `min-width` overflow; ensure `.camera-preview video` fills modal width on mobile |
| `src/state/mod.rs` | Add `drawer_open: RwSignal<bool>` to `AppState` |
| `src/app.rs` | On mobile (CSS class / signal), wrap `ControlsPanel` in a drawer shell; add backdrop div; wire `drawer_open` signal to CSS class |
| `src/components/controls_panel.rs` | Minor: ensure the panel content scrolls correctly inside the drawer container |
| `src/components/canvas_view.rs` | (1) Add `touch-action: none` via CSS to suppress browser pan/zoom; (2) confirm touch pointer events work with existing handlers; (3) add second-pointer tracking for pinch-to-zoom; (4) update canvas CSS size dynamically and call `gl.viewport` on resize |
| `src/components/camera_overlay.rs` | No Rust code changes needed; responsive behaviour is handled entirely in CSS |
| `src/components/header.rs` | Compact height variant for mobile via CSS |

### New state
- `drawer_open: RwSignal<bool>` in `AppState` — controls the mobile drawer CSS class.
- Pinch tracking state can be local to `canvas_view.rs` as a `Rc<RefCell<Option<PinchState>>>` or
  plain `Rc<RefCell<HashMap<i32, (f32, f32)>>>` mapping pointer ID → last position.

### No changes to
- WebGL shaders (`.glsl` files) — completely unaffected.
- The `Renderer` struct internals — only `gl.viewport` call site needs updating when canvas size
  changes.
- Routing, camera, export pipeline — unaffected.

## Open Questions
1. **Canvas resize trigger:** The canvas size change on mobile (e.g. when the soft keyboard appears
   or orientation changes) must call `gl.viewport`. The cleanest approach is a `ResizeObserver` on
   the canvas element. `web-sys` supports `ResizeObserver` behind the `"ResizeObserver"` feature
   flag — confirm whether this feature is already enabled in `Cargo.toml` before implementation.
2. **Drawer height on mobile:** Should the open drawer have a fixed height (e.g., 55vh) or fill all
   the space below the header dynamically? A fixed height ensures the canvas is still partially
   visible when the drawer is open, which is a nicer UX.
3. **`touch-action` CSS:** Setting `touch-action: none` on the canvas element prevents the browser
   from consuming touch events for scrolling/pinching the viewport, which is necessary for the
   pointer event handlers to fire. Verify this doesn't cause issues with `prevent_default()` calls
   already in `canvas_view.rs`.
4. **Pointer capture:** During a single-finger drag, `setPointerCapture` should be called on the
   canvas so events are not lost if the pointer moves outside the canvas boundary. Confirm this
   already exists or add it.

## Out of Scope / Future Work
- Swipe-to-open/close the drawer (swipe gesture on the handle strip).
- Landscape-specific layout (e.g., side panel at smaller width in landscape).
- PWA manifest, install prompt, or offline support.
- Live camera on mobile (separate getUserMedia UX considerations for mobile).
- Two-finger rotation gesture for the rotation parameter.
- High-DPI / `devicePixelRatio` rendering (render at 2× resolution on retina screens).

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan
*To be determined*

## Spec Changes
*To be determined (list any doc/*.md files that will need updating)*

## Test Strategy

### Constraint: responsive layout is not automatable via `wasm-pack test`

`wasm-pack test --headless --firefox` runs in a headless Firefox window with a
fixed, large viewport. There is no API to resize that window programmatically
from a Rust test. As a result, CSS media queries (the `@media (max-width: 768px)`
breakpoint) and their effects on layout never fire in the automated test suite.

This means:
- The **visual layout**, the **slide-up drawer animation**, and the **touch-target
  sizes** must be verified manually on a real or DevTools-simulated mobile device.
- The **logic layer** (signal state, math) is still automatable and should be.
- The manual test checklist in the M11 milestone doc is the **primary verification
  vehicle** for everything this feature does that can be seen or touched.

This mirrors the approach taken for M10 (Steampunk Polish): CSS and visual work is
verified manually; only the Rust logic layer gets automated tests.

---

### Tier 1 — Native unit tests (`cargo test`)

Pure Rust math with no browser dependency. Two functions warrant unit tests:

**`pinch_distance(ax, ay, bx, by) -> f32`**  
The Euclidean distance between two pointer positions. Used to compute the zoom
delta during a pinch gesture. Test cases: axis-aligned pairs, diagonal pairs,
identical points (distance = 0), known 3-4-5 right triangle.

**`pinch_zoom_delta(old_dist, new_dist, current_zoom, min, max) -> f32`**  
Computes `(new_dist / old_dist) * current_zoom`, clamped to `[min, max]`. Test
cases: pinch in (new < old), pinch out (new > old), clamp at minimum, clamp at
maximum, `old_dist` near zero should not panic (guard against divide-by-zero).

Both functions should live in `src/utils.rs` alongside the existing pure-Rust
helpers, following the established pattern.

---

### Tier 3 — Browser integration tests (`wasm-pack test --headless --firefox`)

New file: `tests/m11_mobile_layout.rs`

The headless viewport is large, so the `@media (max-width: 768px)` rules never
apply. Tests at this tier therefore verify **signal wiring and DOM class changes**
— the mechanism that CSS transitions are hooked to — not the visual appearance.

| Test | What it asserts |
|---|---|
| `drawer_open_defaults_false` | After mounting `App`, the `drawer_open` signal reads `false` and the drawer container does **not** have the `drawer--open` CSS class |
| `drawer_toggles_on_handle_click` | Dispatching a `click` event on the drawer handle element toggles `drawer_open` from `false` → `true` and adds the `drawer--open` class (tick, then assert) |
| `backdrop_click_closes_drawer` | Programmatically set `drawer_open = true`, tick, dispatch `click` on the backdrop element, tick, assert `drawer_open` is `false` and class removed |
| `app_mounts_with_drawer_signal` | Smoke test: `App` mounts without panic; `drawer_open` is in context; the drawer handle element is present in the DOM |

Note: these tests exercise the **reactive wiring** (signal → CSS class → DOM),
not the CSS media query. The handle element and backdrop will be present in the
DOM even at the large test viewport — they are just hidden/unstyled by media
query, but the DOM nodes and event listeners still exist and are testable.

---

### Manual test checklist (summarised here; full list lives in `doc/milestones/m11-mobile-layout.md`)

**Setup:** Open Chrome DevTools → Toggle device toolbar → iPhone SE (375 × 667).

Layout:
- [ ] Canvas fills the full 375 px width as a square; no horizontal scroll
- [ ] The drawer handle strip is visible at the bottom of the screen
- [ ] Header is compact and does not wrap or overflow

Drawer:
- [ ] Tapping the handle opens the drawer with a smooth slide-up animation
- [ ] The ▲/▼ chevron flips when the drawer opens and closes
- [ ] Tapping the dimmed canvas backdrop closes the drawer
- [ ] Controls panel inside the drawer is scrollable when content exceeds drawer height
- [ ] All sliders and buttons have tap targets ≥ 44 px tall (verify in DevTools inspector)

Touch interactions:
- [ ] Single-finger drag on the canvas repositions the center of symmetry; no page scroll occurs
- [ ] Two-finger pinch on the canvas changes the zoom value (verify via slider position updating)
- [ ] No "Unable to preventDefault inside passive event listener" warnings in console

Camera overlay:
- [ ] Camera modal fits within 375 px width without horizontal scroll
- [ ] Video preview fills the modal width
- [ ] CAPTURE and CANCEL buttons have tap targets ≥ 44 px

Desktop regression:
- [ ] At 1024 px wide, layout is unchanged: side-by-side panel + canvas
- [ ] Collapse/expand panel button still works on desktop
- [ ] No new console errors at any viewport width

## Decisions Made

### Decision: keep WebGL buffer at 800×800; use CSS scaling
**Chosen:** HTML `width`/`height` attributes stay `800`; CSS scales the canvas to fill the
viewport; `client_width()` used for pointer normalisation.  
**Alternatives considered:** Dynamically resize the WebGL viewport with ResizeObserver.  
**Rationale:** Simpler, no new web-sys features needed, and 800 px is sufficient quality.

### Decision: separate `drawer_open` signal from `panel_open`
**Chosen:** Added `drawer_open: RwSignal<bool>` alongside existing `panel_open`.  
**Alternatives considered:** Reuse `panel_open` for both desktop and mobile.  
**Rationale:** Keeps desktop collapse behaviour unchanged; avoids coupling the signals.

### Decision: drawer handle in `app.rs`, not inside `ControlsPanel`
**Chosen:** `<button class="drawer-handle">` and `<div class="drawer-backdrop">` in `app.rs`.  
**Rationale:** App-level layout concerns belong in the root component.

## Lessons / Highlights

### `(*canvas_el).clone()` needed for owned canvas handle in closures
`canvas_el: &web_sys::HtmlCanvasElement` is a borrow of a local. `canvas_el.clone()` copies
the reference (`&T`), not the owned value. Use `(*canvas_el).clone()` to get an owned
`web_sys::HtmlCanvasElement` that can be moved into `'static` closures. See lesson L16.

### `UnmountHandle<M>` cannot be named in helper function return types
The concrete `M` type returned by `mount_to` cannot be named with `impl Fn()`. Helpers that
return the mount handle cause a compiler error. Keep `let _handle = mount_to(...)` inline in
each test. See lesson L17.

### `touch-action: none` is more reliable than `preventDefault` alone
Setting `touch-action: none` directly in CSS prevents the browser from acquiring touch
handling at the CSS level, before any JS event dispatch. This is the recommended approach
alongside `preventDefault()` in pointer listeners.
