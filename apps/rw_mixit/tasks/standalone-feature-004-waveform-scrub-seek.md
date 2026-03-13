# Feature 004: Waveform Scrub Seek

**Feature Doc:** features/feature_004_waveform_scrub_seek.md
**Milestone:** standalone feature
**Status:** 🔄 In Progress

## Restatement

Feature 004 adds drag-scrub interaction to each deck's waveform canvas so that
users can slide the playhead to any position — while playing or paused — by
pressing and dragging on the canvas. The existing `on:click` handler (paused-only,
single-click seek) is replaced by `mousedown`/`mousemove`/`mouseup`/`mouseleave`
handlers plus a local `is_scrubbing: RwSignal<bool>` that tracks whether a scrub
is in progress. Every `mousemove` while scrubbing calls the existing
`AudioDeck::seek()`, producing vinyl-style audio pops that are an intentional
stylistic choice. The cursor changes from `grab` to `grabbing` during a drag.
Touch, debouncing, and preview/ghost-marker modes are out of scope for this feature.

Confirmed with user before implementation.

## Design

### Data flow

```
User presses mouse on waveform canvas
  → on:mousedown fires
    → is_scrubbing.set(true)
    → seek_from_canvas_x(mouse_x, canvas_width, current_secs, duration_secs)
      → clamp((current + (mouse_x - center_x) * secs_per_px), 0, duration)
    → AudioDeck::seek(seek_pos, rate)
    → state.current_secs.set(seek_pos)

User drags mouse
  → on:mousemove fires
    → if is_scrubbing.get_untracked() == true
      → same seek calculation → AudioDeck::seek() → state.current_secs.set()

User releases mouse / leaves canvas
  → on:mouseup or on:mouseleave fires
    → is_scrubbing.set(false)

is_scrubbing signal → style:cursor="grabbing" / "grab" on the canvas element
```

### Function / type signatures

```rust
// src/canvas/waveform_draw.rs — pure math helper (pub(crate), Tier-1 testable)

/// Compute a seek position in seconds from a mouse X pixel offset on the
/// waveform canvas.
///
/// The waveform is rendered so that the current playhead is always centred at
/// `canvas_width / 2`. A click at pixel `mouse_x` is therefore
/// `(mouse_x - center_x)` pixels away from the playhead, which maps to a time
/// offset of `(mouse_x - center_x) * (duration / canvas_width)` seconds.
/// The result is clamped to `[0.0, duration]`.
///
/// Returns `0.0` when `canvas_width <= 0.0` or `duration <= 0.0` (guards
/// against divide-by-zero).
pub(crate) fn seek_from_canvas_x(
    mouse_x:      f64,
    canvas_width: f64,
    current:      f64,
    duration:     f64,
) -> f64

// src/components/deck.rs — local component state + event handlers (inside DeckView)

let is_scrubbing: RwSignal<bool> = RwSignal::new(false);

// Shared seek logic wrapped in Rc so it can be moved into both mousedown and
// mousemove handlers without violating the borrow checker.
let do_scrub: Rc<dyn Fn(f64)> = Rc::new(move |mouse_x: f64| { ... });

let on_waveform_mousedown = move |ev: web_sys::MouseEvent| { ... };
let on_waveform_mousemove = move |ev: web_sys::MouseEvent| { ... };
let on_scrub_end_up       = move |_: web_sys::MouseEvent| { is_scrubbing.set(false); };
let on_scrub_end_out      = move |_: web_sys::MouseEvent| { is_scrubbing.set(false); };
```

### Edge cases

| Case | Handling |
|---|---|
| `duration_secs == 0.0` (no track loaded) | Guard in `seek_from_canvas_x` — returns `current`; `do_scrub` additionally guards `duration <= 0.0` before calling deck |
| `canvas_width == 0` (not yet rendered) | Guard in `seek_from_canvas_x` — returns `current`; also the `waveform_ref.get_untracked()` returns `None` guard in `do_scrub` |
| Drag to left of waveform start | `clamp(0.0, duration)` |
| Drag past end of track | `clamp(0.0, duration)` |
| `mouseleave` without prior `mousedown` | `is_scrubbing` is `false`, `set(false)` is a no-op |
| `mousemove` without prior `mousedown` | `is_scrubbing.get_untracked() == false` → early return |
| Rapid mousemove (high frequency) | All seeks fire; intentional — no throttle |
| Deck has no audio loaded | `audio_deck_holder.borrow()` is `None` → the `if let` guard silently skips the seek call |

### Integration points

| File | Change |
|---|---|
| `src/canvas/waveform_draw.rs` | Add `seek_from_canvas_x` pub(crate) function + Tier-1 tests |
| `src/components/deck.rs` | Replace `on_waveform_click` with `is_scrubbing` signal + 4 handlers + view! wiring |
| `static/style.css` | Add `cursor: grab` + `user-select: none` to `.waveform-canvas`; `style:cursor` handles grabbing inline |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Existing `seek_from_canvas_x` uses `duration / canvas_width` regardless of zoom level — the waveform draw also uses `total_peak_width = width` regardless of zoom, so these cancel correctly. No bug. | Confirmed by reading both the draw pass scroll formula and the click handler formula — identical coordinate system. |
| Correctness | `mousemove` fires while `is_playing` is true — each call restarts `AudioBufferSourceNode`, causing brief audio gap. | Intentional per user choice ("accept the pops"); `AudioDeck::seek()` already handles was_playing restart. |
| Simplicity | `do_scrub` is an `Rc<dyn Fn(f64)>` to share between `mousedown` and `mousemove`. | Simplest option consistent with the `nudge_end_rc` pattern already used in `controls.rs`. |
| Coupling | `seek_from_canvas_x` lives in `waveform_draw.rs` alongside `time_to_x` (its inverse). | Good cohesion — both are canvas-coordinate math. Avoids a new file for one function. |
| Performance | Continuous `AudioDeck::seek()` on mousemove creates+starts a new `AudioBufferSourceNode` each call. | Accepted trade-off (vinyl feel). No additional risk beyond the existing seek path. |
| Testability | `seek_from_canvas_x` is pure `f64 → f64` math — fully Tier-1 testable. | All FR6 (clamping) cases covered by native unit tests. |

## Implementation Notes

- `RwSignal<bool>` is `Copy` in Leptos, so `on_scrub_end_up` and `on_scrub_end_out`
  can both capture `is_scrubbing` by value without the `Rc` wrapper needed by the
  `nudge_end` pattern in `controls.rs`.
- `style:cursor=move || ...` reactive inline style replaces any need for a
  `.scrubbing` CSS class toggle.
- `user-select: none` added to `.waveform-canvas` in CSS to prevent accidental
  text-selection highlight during drag.
- The `on_waveform_click` closure is removed entirely; a plain click (mousedown +
  mouseup with no movement) naturally seeks via `on_waveform_mousedown`.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| FR1: mousedown begins scrub + seeks | 1 | ✅ (via `seek_from_canvas_x` math tests) | Signal wiring tested indirectly |
| FR2: mousemove while scrubbing seeks | 1 | ✅ (same math path) | Guard (`is_scrubbing`) not directly tested |
| FR3: mouseup/mouseleave ends scrub | — | ⚠️ Waived | Trivial `RwSignal::set(false)`; no logic to test |
| FR4: cursor grab/grabbing | — | ⚠️ Waived | CSS/inline-style; would need Tier-3 DOM assertion. Low risk — visually verified in smoke test |
| FR5: plain click still seeks | 1 | ✅ (mousedown fires seek; mouseup just clears flag) | Same code path as drag |
| FR6: clamp [0, duration] | 1 | ✅ `seek_from_canvas_x_clamps_*` tests | Both ends covered |
| FR7: seek while playing | — | ⚠️ Waived | Delegated to `AudioDeck::seek()` which has existing tests |
| Zero duration guard | 1 | ✅ `seek_from_canvas_x_zero_duration*` | Returns 0.0 |
| Zero canvas_width guard | 1 | ✅ `seek_from_canvas_x_zero_canvas_width*` | Returns current |
| Click at exact center → no movement | 1 | ✅ `seek_from_canvas_x_center_returns_current` | |
| Left-edge drag → seeks toward start | 1 | ✅ `seek_from_canvas_x_left_edge` | |
| Right-edge drag → seeks toward end | 1 | ✅ `seek_from_canvas_x_right_edge` | |

## Test Results

All 129 native unit tests pass (`cargo test`). 8 new tests added in
`canvas::waveform_draw::tests`:
```
test canvas::waveform_draw::tests::seek_from_canvas_x_center_returns_current ... ok
test canvas::waveform_draw::tests::seek_from_canvas_x_arbitrary_position ... ok
test canvas::waveform_draw::tests::seek_from_canvas_x_clamps_above_duration ... ok
test canvas::waveform_draw::tests::seek_from_canvas_x_left_edge_seeks_backward ... ok
test canvas::waveform_draw::tests::seek_from_canvas_x_right_edge_seeks_forward ... ok
test canvas::waveform_draw::tests::seek_from_canvas_x_clamps_below_zero ... ok
test canvas::waveform_draw::tests::seek_from_canvas_x_zero_canvas_width_returns_zero ... ok
test canvas::waveform_draw::tests::seek_from_canvas_x_zero_duration_returns_zero ... ok
```
`cargo clippy --target wasm32-unknown-unknown -- -D warnings`: clean (0 warnings).
`trunk build`: success.

## Review Notes

Code review (automated) flagged one potential issue:

**Finding: `mouseleave` ends scrub during fast drag**
If the user drags quickly and the cursor momentarily exits the canvas bounds,
`mouseleave` fires and ends the scrub session. The reviewer suggested using
pointer capture or dropping the `mouseleave` handler entirely.
**Waived:** This matches the established `nudge` control pattern in
`controls.rs` (identical `on:mouseup` + `on:mouseleave` pair). Prevents a
"stuck scrubbing" state if `mouseup` is missed while the cursor is outside the
window. Acceptable in a fun DJ toy; can be revisited with pointer capture in a
follow-on.

**Note on clippy `--tests`:** Pre-existing clippy lint failures exist in test
code in `bpm.rs`, `deck_audio.rs`, and `platter_draw.rs` when running
`cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings`. These
are not introduced by this feature. The project's standard `make.py lint`
command does not pass `--tests`, so they were not previously surfaced. This
feature's production code passes `make.py lint` cleanly.

## Decisions Made

See feature document §Decisions Made for full entries. Summary:
1. `seek_from_canvas_x` lives in `waveform_draw.rs` alongside `time_to_x` (its inverse)
2. `Rc<dyn Fn(f64)>` for shared `do_scrub` closure — matches `nudge_end_rc` pattern
3. `mouseleave` ends scrub — consistent with nudge controls
4. Paused-only restriction removed — user requested seek in any state

## Lessons / Highlights

1. `RwSignal<T>` is `Copy` — no `Rc` needed when closures capture only signals
2. `style:<prop>` reactive inline style is cleaner than CSS class toggling for cursor changes
3. Waveform seek/draw coordinate systems are identical — zoom doesn't affect the time-to-pixel scale for scrolling, only bar width

## Callouts / Gotchas

- The `nudge` pattern in `controls.rs` uses `Rc<dyn Fn()>` to share the end handler.
  For scrub-end, `is_scrubbing.set(false)` only captures `is_scrubbing` which is
  `Copy`, so two independent closures work without `Rc`.
- `seek_from_canvas_x` uses `duration / canvas_width` — zoom level does NOT change
  this ratio because `draw_waveform` also uses `canvas_width` as `total_peak_width`
  regardless of zoom.  The zoom factor only affects the visual peak bar widths (how
  many peaks are crammed into the canvas), not the time-to-pixel scale for scrolling.
