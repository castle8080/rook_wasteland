# Feature 004 — Waveform Scrub Seek

## Status
Implemented

## Summary
Users can drag left or right on the waveform canvas to scrub the playback position — while the track is playing or paused. Each pixel of drag movement continuously updates the position (and audio output), giving the feel of vinyl-style scrubbing. Single-click seek is also upgraded to work regardless of playback state.

## Problem Statement
The waveform canvas currently only supports click-to-seek when the deck is paused. There is no way to slide through a track to find a specific moment, and no way to reposition while playing. DJs naturally want to grab the waveform and drag to a new position — both to cue up a track and to ride the playhead during a live mix. This interaction is missing entirely from the current UI.

## Goals
- Click anywhere on the waveform to seek immediately, whether playing or paused.
- Press and drag horizontally on the waveform to continuously scrub position.
- Each `mousemove` during a drag triggers a seek, producing vinyl-style audio pops that feel intentional.
- The waveform display visually follows the drag in real time (playhead stays centered, waveform scrolls).
- The cursor changes to a grab/grabbing style during hover and drag to signal the interaction.

## Non-Goals
- No touch/gesture support (consistent with resolved decision: no touch gestures in v1).
- No "silent scrub" or preview mode — audio is always live.
- No ghost marker or separate visual overlay during drag (the scrolling playhead is sufficient feedback).
- No throttling or debouncing of mousemove seeks — accepts audio pops as a stylistic choice.
- No keyboard scrub shortcuts.

## User Stories
- As a DJ, I want to click anywhere on the waveform while a track is playing so that I can jump to a new position without stopping the music.
- As a DJ, I want to click and drag on the waveform so that I can scrub through the track to find the right moment.
- As a listener, I want the waveform to visually scroll in real time as I drag so that I can see exactly where I am in the track.
- As a user, I want the cursor to change when I hover over the waveform so that I understand it is draggable.

## Functional Requirements
1. A `mousedown` on the waveform canvas shall begin a scrub session, immediately seeking to the clicked position regardless of `is_playing` state.
2. A `mousemove` while the scrub session is active shall compute a new target time from the cursor's X position relative to the canvas center and call `AudioDeck::seek()`.
3. A `mouseup` or `mouseleave` event on the waveform canvas shall end the scrub session; no additional seek is performed.
4. The waveform canvas shall display `cursor: grab` on hover and `cursor: grabbing` during an active drag.
5. A plain click (mousedown + mouseup with no movement) shall seek to the clicked position — identical to the current behavior but without the paused-only restriction.
6. Scrubbing shall correctly clamp the target position to `[0.0, duration_secs]`.
7. Seeking while playing shall restart audio at the new position (reuses existing `AudioDeck::seek()` which already handles this).

## UI / UX Notes
- The waveform canvas element (`<canvas class="waveform-canvas">`) is the interaction surface.
- Add CSS: `cursor: grab` on `.waveform-canvas` and `cursor: grabbing` on `.waveform-canvas.scrubbing` (or via inline style toggled by the `is_scrubbing` signal).
- No new visible controls are introduced — the interaction is discoverable via the cursor change.
- The playhead remains centered; the waveform scrolls under it during drag, providing real-time position feedback.
- Consistent with the spec's "Zoom controls" note: drag-scrub should respect the current zoom level when calculating `secs_per_px` (same formula as the existing click handler).

## Architecture Fit

### Existing code touched
- **`src/components/deck.rs`** — `DeckView` component. The existing `on_waveform_click` closure (paused-only, single-click) is replaced by three event handlers: `on_mousedown`, `on_mousemove`, `on_mouseup`/`on_mouseleave`. A local `RwSignal<bool>` (`is_scrubbing`) is introduced as component-local state.
- **`static/style.css`** (or deck-specific styles) — add `cursor: grab / grabbing` rules for the waveform canvas.

### New code introduced
- Local `is_scrubbing: RwSignal<bool>` inside `DeckView` (no shared state needed; scrub is always per-deck).
- Helper closure `compute_seek_pos(canvas_el, mouse_x, current_secs, duration_secs) -> f64` to avoid duplication across the three handlers.

### No changes needed
- `AudioDeck::seek()` — already handles seek-while-playing and seek-while-paused.
- `DeckState` — no new signals required.
- `canvas/waveform_draw.rs` — no rendering changes needed (playhead and scrolling already correct).

## Open Questions
- Should `mousemove` during scrub be rate-limited (e.g., max one seek per animation frame via `request_animation_frame`)? The current decision is "no throttling — accept pops," but if implementation reveals excessive CPU cost this could be revisited.
- Should `mouseleave` trigger a final seek to the cursor's last known position, or simply end the scrub without a final seek? (Current plan: end without additional seek.)

## Out of Scope / Future Work
- Touch scrubbing (pinch/swipe) — deferred per the existing resolved decision on mobile touch support.
- A dedicated scrub position indicator (ghost line) — could be added as a follow-on if users find it helpful.
- Smooth audio during scrub (e.g., via Web Audio API playback-rate ramp) — requires AudioWorklet which is out of scope per the resolved stack decisions.

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan

### Files modified
- **`src/canvas/waveform_draw.rs`**: Added `pub(crate) fn seek_from_canvas_x()` — a pure math helper that converts a mouse X pixel offset to a seek time in seconds, using the same coordinate system as the existing `time_to_x` scroll formula. Co-locating both functions here groups all waveform coordinate math in one place. Also added 8 Tier-1 unit tests covering all meaningful cases.
- **`src/components/deck.rs`**: Replaced the old `on_waveform_click` closure (paused-only, single-click seek) with: `is_scrubbing: RwSignal<bool>` local signal, `do_scrub: Rc<dyn Fn(f64)>` shared seek closure, `on_waveform_mousedown`, `on_waveform_mousemove`, `on_scrub_end_up`, `on_scrub_end_out` event handlers. Updated the view! macro to wire up all four handlers and the reactive `style:cursor` attribute. Added import of `seek_from_canvas_x` from `waveform_draw`.
- **`static/style.css`**: Changed `.waveform-canvas` cursor from `crosshair` to `grab`; added `user-select: none` to prevent highlight artifacts during drag.

### Deviations from Architecture Fit section
- None. All changes landed exactly where the feature doc predicted.

## Spec Changes
- **`doc/rw_mixit_spec.md`** §6.5: updated waveform seek bullet to "while playing or paused"; added drag-scrub bullet.
- **`doc/ascii_wireframes.md`**: updated waveform wireframe caption and interaction table to reflect click/drag-to-seek (playing or paused) and grab cursor.
- **`doc/rw_mixit_tech_spec.md`** module tree: noted `seek_from_canvas_x` in the `waveform_draw.rs` entry.

## Test Strategy

### Tests added
- **File:** `src/canvas/waveform_draw.rs` — 8 Tier-1 native unit tests:
  - `seek_from_canvas_x_center_returns_current` — clicking at center maps to no offset
  - `seek_from_canvas_x_left_edge_seeks_backward` — left edge = half-duration backward
  - `seek_from_canvas_x_right_edge_seeks_forward` — right edge = half-duration forward
  - `seek_from_canvas_x_clamps_below_zero` — result never goes below 0.0
  - `seek_from_canvas_x_clamps_above_duration` — result never exceeds duration
  - `seek_from_canvas_x_zero_duration_returns_zero` — guard against divide-by-zero
  - `seek_from_canvas_x_zero_canvas_width_returns_zero` — guard against unrendered canvas
  - `seek_from_canvas_x_arbitrary_position` — mid-canvas click at a computed offset

### Coverage gaps (explicitly noted)
- Tier-2/3: No browser tests for the event handler wiring (`is_scrubbing` signal, DOM cursor style). These require simulated mouse events in a browser; waived as too complex relative to the simplicity of the wiring code.
- `AudioDeck::seek()` while playing: covered by the existing `AudioDeck` test suite; not repeated here.

## Decisions Made

### Decision: `seek_from_canvas_x` lives in `waveform_draw.rs`, not `deck.rs`
**Chosen:** Place the pure math function in `src/canvas/waveform_draw.rs` alongside `time_to_x` (the inverse operation).
**Alternatives considered:** Inline in `deck.rs`; extract to `src/utils/waveform.rs`.
**Rationale:** All waveform coordinate math is now in one file — `time_to_x` converts time→pixel and `seek_from_canvas_x` converts pixel→time. Testability is identical either way since it's pure Rust. Avoids a new file for a single function.

### Decision: `Rc<dyn Fn(f64)>` for shared seek closure
**Chosen:** Wrap `do_scrub` in `Rc<dyn Fn(f64)>` and clone for `mousedown` and `mousemove` handlers.
**Alternatives considered:** Duplicate the seek logic in both handlers; pass `is_scrubbing` into an extracted module function.
**Rationale:** Matches the established `nudge_end_rc` pattern in `controls.rs`. Avoids code duplication. The `Rc` overhead is negligible — it's created once per component mount and the closure runs at interactive frame rates.

### Decision: `mouseleave` ends scrub (consistent with nudge pattern)
**Chosen:** `mouseleave` ends the scrub session in addition to `mouseup`.
**Alternatives considered:** End only on `mouseup`; use pointer capture.
**Rationale:** Matches the existing `nudge` control pattern (`on:mouseup` + `on:mouseleave`) throughout the codebase. Prevents a "stuck scrubbing" state if the mouseup is missed (e.g. user lifts button outside window). A code review flagged this as potentially breaking fast drags — waived as consistent with existing UI patterns and acceptable in a fun DJ toy context.

### Decision: Remove paused-only restriction from click-seek
**Chosen:** Seeking now works in any playback state (as requested during feature submission).
**Alternatives considered:** Keep paused-only for click, add drag-only-while-playing.
**Rationale:** User explicitly chose "While playing AND paused" during the feature submission interview. Simplifies the code (removes the early-return guard).

## Lessons / Highlights

### `RwSignal<bool>` is `Copy` — no `Rc` needed for two-handler end patterns
When a handler closure only captures signals (which are `Copy` in Leptos), two separate closures that both capture the same signal can coexist without any `Rc` wrapper. This is a simpler pattern than the `nudge_end_rc` workaround and is safe whenever the closure captures only signals. Reserve `Rc<dyn Fn()>` for closures that also capture non-Copy values like `Rc<RefCell<...>>`.

### `style:cursor` reactive inline style replaces `class`-based cursor toggling
For cursor changes driven by component state, `style:cursor=move || if flag.get() { "grabbing" } else { "grab" }` in Leptos `view!` is simpler than toggling CSS classes. No extra CSS rule is needed and the cursor updates reactively without a DOM class mutation.

### Waveform seek and draw use the same coordinate system — zoom factor cancels out
The `draw_waveform` function uses `total_peak_width = canvas_width` regardless of zoom level (zoom only affects peak bar width, not the scroll formula). Therefore `seek_from_canvas_x` using `duration / canvas_width` is always correct at any zoom — both operations share the same pixel-to-time scale. No zoom correction is needed in the seek calculation.
