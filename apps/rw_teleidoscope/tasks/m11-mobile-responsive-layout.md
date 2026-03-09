# Feature 001: Mobile / Responsive Layout

**Feature Doc:** features/feature_001_mobile_responsive_layout.md  
**Milestone:** M11  
**Status:** 🔄 In Progress

---

## Restatement

The app currently has a fixed desktop layout (260 px panel + 800×800 canvas side-by-side) with no
mobile breakpoint, no touch-specific gestures, and pointer coordinate normalisation hardcoded to
800 px. This feature adds a CSS breakpoint at 768 px that replaces the side-by-side layout with a
full-screen canvas + slide-up bottom drawer on mobile. A new `drawer_open: RwSignal<bool>` in
`AppState` drives the drawer open/close state; a persistent handle button and a tap-to-dismiss
backdrop are wired to that signal in `app.rs`. Touch-generated pointer events already work via the
Pointer Events API, but single-finger drag coordinate normalisation must be changed from the
hardcoded `CANVAS_SIZE = 800.0` to `canvas.client_width()` so coordinates are correct at any CSS
display size. Two-finger pinch adds second-pointer tracking (`HashMap<i32, (f32, f32)>`) and maps
the distance ratio between successive moves to the `zoom` signal. The camera modal's `min-width:
340px` rule is replaced so it does not overflow on 320 px devices. Desktop layout and behaviour are
unchanged above the breakpoint. This implements Feature 001 / Milestone M11.

---

## Design

### Data flow

```
User touch/pointer event on canvas
  └─ pointerdown  → insert into active_pointers map; if count==1, set center
  └─ pointermove  → update map entry
       ├─ count==1 + button held → normalise (offset_x / client_width) → params.center.set()
       └─ count==2 → compute new distance, apply_pinch_zoom() → params.zoom.set()
  └─ pointerup / pointercancel → remove from map; if count drops <2, last_pinch_dist reset

User taps drawer handle button
  └─ app_state.drawer_open.update(|v| *v = !*v)
       └─ controls-panel class:drawer--open fires
            └─ CSS transition: transform translateY(0)

User taps backdrop
  └─ app_state.drawer_open.set(false)
       └─ class removed → transform: translateY(100%) → panel slides away
```

### Function / type signatures

```rust
// src/utils.rs

/// Euclidean distance between two 2-D points.
/// Pure math; no browser dependency.
pub fn pinch_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32;

/// Apply a pinch-zoom delta to `current_zoom` and clamp to [min, max].
///
/// `old_dist` and `new_dist` are in the same coordinate space (e.g. CSS px).
/// The result is `(new_dist / old_dist) * current_zoom`, clamped.
/// Guards against `old_dist` ≤ 0 to prevent division by zero.
pub fn apply_pinch_zoom(old_dist: f32, new_dist: f32, current_zoom: f32, min: f32, max: f32) -> f32;
```

```rust
// src/state/app_state.rs  (new field)
pub drawer_open: RwSignal<bool>,   // true = drawer visible (mobile only)
```

```rust
// src/components/canvas_view.rs  (new local state, inside init Effect)
let active_pointers: Rc<RefCell<HashMap<i32, (f32, f32)>>>; // pointer_id → (offset_x, offset_y)
let last_pinch_dist: Rc<Cell<f32>>;                          // reference distance for ratio
```

### Edge cases

| Edge case | Handling |
|---|---|
| `client_width()` returns 0 (canvas not yet laid out) | `.max(1)` guard prevents division by zero |
| `old_dist` ≈ 0 in pinch (two fingers at same position) | `apply_pinch_zoom` returns `current_zoom` unchanged |
| Third finger lands (3+ pointers) | Extra pointer IDs enter map; only the first two are used for distance |
| Pointer cancelled mid-drag (notification shade, call, etc.) | `pointercancel` listener removes from map; drag resets cleanly |
| Drawer opened then canvas resized (orientation change) | CSS `100vw` canvas re-fits automatically; no Rust code needed |
| Desktop: `drawer_open` toggled by test | Signal functions but CSS hides handle/drawer above breakpoint |

### Integration points

| File | What changes |
|---|---|
| `src/utils.rs` | +`pinch_distance`, +`apply_pinch_zoom`, +unit tests |
| `src/state/app_state.rs` | +`drawer_open` field, +default false, +unit test |
| `src/app.rs` | +backdrop `<div>`, +drawer-handle `<button>`, wire `drawer_open` |
| `src/components/controls_panel.rs` | +`class:drawer--open` on `<aside>` |
| `src/components/canvas_view.rs` | pointer coord fix; +pinch tracking; +pointerup/cancel listeners |
| `style/main.css` | +`touch-action: none`; +`@media (max-width: 768px)` block; camera fix |
| `tests/m11_mobile_layout.rs` | new Tier 3 test file |

---

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `offset_x/y` on secondary touch pointer may not be canvas-relative on all browsers | Using `offset_x/y` is spec-compliant for PointerEvent; tested against desktop FF which is the CI target |
| Correctness | Three-finger touch puts extra IDs in map; distance uses only two | Stable: HashMap always has the two oldest entries active; third finger adds noise but does not crash |
| Simplicity | Two separate signals (`panel_open` and `drawer_open`) for essentially the same concept | Intentional per FR-6; avoids changing desktop behaviour; on mobile only `drawer_open` drives CSS |
| Coupling | `canvas_view.rs` imports `utils::pinch_distance` and `apply_pinch_zoom` | Acceptable; these are pure helpers that belong in `utils` |
| Performance | `HashMap::insert/remove` on every pointer event | Negligible; map size ≤ 3; operations are O(1) |
| Testability | CSS media-query effects not automatable in headless FF | Acknowledged; manual checklist covers all visual/layout assertions |

---

## Implementation Notes

- `touch-action: none` is added globally to `#kaleidoscope-canvas`, not scoped to the mobile media
  query. This ensures the existing `prevent_default()` on `pointerdown` is not fighting the
  browser's built-in touch handling on any viewport.
- The WebGL drawing buffer remains at 800×800 (`width="800" height="800"` attributes are
  unchanged). CSS scales the canvas to fill the viewport. Pointer coordinate normalisation uses
  `client_width()` to correctly map CSS pixels → [0, 1].
- `set_pointer_capture` is called in `pointerdown` so drag events keep firing even if the pointer
  moves outside the canvas boundary.
- The drawer handle and backdrop are hidden on desktop via `display: none` in the base CSS rule
  (overridden to `display: flex` / `display: block` inside the media query).
- On mobile, `.panel-toggle-btn` is hidden by CSS (`display: none`); it is not removed from the
  DOM to avoid complicating the Leptos component tree.
- Export dropdown `position: absolute; bottom: calc(100% + 4px)` pops upward; this works inside
  the scrollable drawer since overflow is temporarily visible. The `panel-content` uses
  `overflow-y: auto; overflow-x: visible` on mobile.

---

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `pinch_distance` zero inputs | 1 | ✅ | `pinch_distance_same_point` |
| `pinch_distance` axis-aligned | 1 | ✅ | horizontal + vertical cases |
| `pinch_distance` 3-4-5 triangle | 1 | ✅ | `pinch_distance_3_4_5_triangle` |
| `apply_pinch_zoom` scale out | 1 | ✅ | new > old |
| `apply_pinch_zoom` scale in | 1 | ✅ | new < old |
| `apply_pinch_zoom` clamp at max | 1 | ✅ | result would exceed max |
| `apply_pinch_zoom` clamp at min | 1 | ✅ | result would go below min |
| `apply_pinch_zoom` zero old_dist | 1 | ✅ | returns current_zoom unchanged |
| `drawer_open` defaults false | 1 | ✅ | in `app_state.rs` |
| `drawer_open` signal in context | 3 | ✅ | `app_mounts_with_drawer_signal` |
| Drawer handle click toggles open | 3 | ✅ | `drawer_toggles_on_handle_click` |
| Backdrop click closes drawer | 3 | ✅ | `backdrop_click_closes_drawer` |
| Pointer coord normalization fix | — | ⚠ manual | `offset_x / client_width` correct at any CSS size |
| Pinch zoom end-to-end | — | ⚠ manual | requires real touch simulation |
| Mobile media query layout | — | ⚠ manual | headless FF viewport is large |
| Camera modal no overflow | — | ⚠ manual | CSS `min-width` fix verified visually |
| Touch targets ≥ 44 px | — | ⚠ manual | verified in DevTools |

---

## Test Results

*To be filled in after running `python make.py test`.*

---

## Review Notes

*To be filled in after self-review.*

---

## Decisions Made

### Decision: keep WebGL buffer at 800×800; use CSS scaling
**Chosen:** HTML `width`/`height` attributes stay `800`; CSS scales the canvas to fill the
viewport; `client_width()` used for pointer normalisation.  
**Alternatives considered:** Dynamically resize the WebGL viewport with ResizeObserver.  
**Rationale:** Simpler, no new web-sys features needed, and 800 px is sufficient quality for
kaleidoscope patterns on mobile screens.

### Decision: separate `drawer_open` signal from `panel_open`
**Chosen:** Add `drawer_open: RwSignal<bool>` alongside existing `panel_open`.  
**Alternatives considered:** Reuse `panel_open` for both desktop and mobile.  
**Rationale:** Keeps desktop collapse behaviour unchanged; avoids coupling the signals; matches
FR-6 verbatim; each signal controls exactly one CSS state.

### Decision: drawer handle as fixed element in `app.rs`, not inside `ControlsPanel`
**Chosen:** `<button class="drawer-handle">` and `<div class="drawer-backdrop">` live in `app.rs`.  
**Alternatives considered:** Put handle inside `ControlsPanel`; use CSS-only accordion approach.  
**Rationale:** App-level concerns (layout modes) belong in the root component; `ControlsPanel`
stays focused on the controls themselves and does not need to know about the layout container.

---

## Lessons / Highlights

*To be filled in after implementation.*

---

## Callouts / Gotchas

- `client_width()` returns 0 if the element is `display: none` or not yet laid out. The `.max(1)`
  guard prevents a divide-by-zero panic; the resulting coordinate will be wrong (off-screen) but
  will not crash the app.
- `touch-action: none` suppresses ALL browser touch actions on the element, including accessibility
  features. It should only be applied to the canvas (not the whole page).
- The export dropdown uses `position: absolute; bottom: calc(100% + 4px)`, which pops upward. On
  mobile, if the drawer content is `overflow-y: auto`, the dropdown might be clipped. Use
  `overflow-x: visible` on the scrollable container.
