# Bug 04 — Playback Slider and Duration Display Incorrect for Short Recordings

## Status
Needs Human Investigation — Multiple Automated Fix Attempts Have Failed

---

## Summary

When playing back a browser-recorded audio clip, the seek slider shows the wrong range and barely moves during playback, and the total duration display shows `0:00` throughout. The audio itself plays correctly to completion.

---

## Observed Behaviour (Original)

- A ~5-second recording plays back correctly from start to finish.
- The seek slider advances only 2–5% of its total width during playback.
- The slider can be manually dragged up to `1:40` (100 seconds), which is not valid for a 5-second clip.
- The duration label to the right of the slider shows `0:00` throughout.

## Observed Behaviour (After Fix Attempt 2)

- Duration label now shows the correct time.
- Slider still advances to only ~5% during playback.
- At the exact moment the 5-second audio ends, the slider jumps to 100%.

## Observed Behaviour (After Fix Attempt 3)

- Still not working correctly. Behaviour not fully characterised before further fixes were halted.

---

## Current State

Three automated fix attempts have been made. Each attempt was based on sound reasoning but the behaviour in the browser has not matched expectations. The slider is still not tracking playback position correctly.

**What is known:**
- The audio plays back correctly (sound is correct).
- The `duration_hint_secs` prop is now wired through from recording metadata — this should give the player a correct `max` value for the slider immediately on load. Whether this is actually reaching the slider correctly in the browser is unknown.
- The `fixing_duration` flag + `ontimeupdate` seek trick replaces the previous `ondurationchange` approach. Whether the seek to `1e9` is triggering `timeupdate` at all for this WebM file is unknown.
- `prop:value=` replaces `value=` on the range input — whether this is being applied correctly by Leptos is unknown.

**What is not known:**
- Which of the three fix layers (hint propagation, seek trick, `prop:value`) is actually having an effect.
- What signal values `duration`, `loaded`, and `current_time` actually hold at each stage in the browser.
- Whether the Leptos `prop:value=` binding on `<input type="range">` works as expected in this version.
- Whether `set_current_time(1e9)` triggers `timeupdate` at all for a short MediaRecorder WebM file.

---

## What Is Needed Next

This bug requires **human inspection with live browser tooling**. Automated reasoning about browser/WASM event timing is speculative without observability. Suggested approaches:

1. **Add `console_log!` / `web_sys::console::log_1` debug statements** at each key point in the audio player:
   - When `onloadedmetadata` fires: log `a.duration()`, `duration_hint_secs`, and what branch is taken
   - When `ontimeupdate` fires: log `fixing_duration`, `a.current_time()`, `a.duration()`
   - When `fixing_duration` branch is entered: log the extracted duration
   - After signal updates: log `duration.get_untracked()` and `loaded.get_untracked()`

2. **Inspect the browser DevTools:**
   - Check the `<input type="range">` `max` attribute and `value` property in the Elements panel during playback
   - Watch the `HTMLAudioElement` properties (`duration`, `currentTime`, `readyState`) in the Console
   - Observe which events actually fire on the `<audio>` element (use the Event Listeners panel)

3. **Isolate the Leptos reactive layer:**
   - Verify that `prop:value=move || current_time.get().to_string()` in Leptos 0.8 actually calls the DOM property setter — check in the Leptos 0.8 source or docs whether `prop:` on non-input elements works the same as on `<input>`



## Root Cause — Layer 1: WebM Infinity Duration

`MediaRecorder` WebM/Opus recordings always have `duration = Infinity`. The WebM muxer cannot write the final duration into the container header in real time. When `loadedmetadata` fires, `HTMLMediaElement.duration` returns `Infinity`, not the real duration.

This causes:
- `format_duration(Infinity)` → `"0:00"` (correctly guarded since Bug 03 fix)
- `Infinity.to_string()` = `"inf"` → invalid HTML attribute for `max` → browser defaults range to 0–100
- 5s / 100-range = 5% slider position

## Root Cause — Layer 2: Seek Trick Using `durationchange` Was Wrong

Fix attempt 2 added a seek-to-end trick (`set_current_time(1e9)`) to force the browser to scan to EOF and discover the real duration. The companion handler used `ondurationchange` to pick up the result.

**Problems with this approach:**

1. **Wrong detection event.** For MediaRecorder WebM files, Chrome may not fire `durationchange` from the 1e9 seek — these files have no EBML Cue points (the index structure that enables efficient WebM seeking). Without Cues, Chrome cannot seek efficiently; it ignores or defers the seek and only fires `durationchange` when it naturally reads to EOF during playback.

2. **`ondurationchange` unconditionally reset the playhead.** The handler called `set_current_time(0.0)` every time it received a finite duration value — with no guard for "are we in seek trick mode?" When `durationchange` fired at natural EOF (during playback), it set the correct duration (5s) and then called `set_current_time(0.0)`. At that instant, the `current_time` signal still held the last `timeupdate` value (≈5s), so the slider showed 5/5 = 100% briefly before the next `timeupdate` reset it to 0. That is the source of the "jumps to 100%" symptom.

3. **`value=` attribute vs `prop:value=` property for the slider.** Using `value=...` in Leptos sets `setAttribute("value", ...)` on the DOM element. For `<input type="range">`, changing the `value` attribute does not reliably update the visible thumb position in all browsers after the element has been interacted with (the "dirty value flag" is set on first interaction). The correct approach is `prop:value=...`, which sets the DOM `.value` property directly.

---

## Fix (Attempt 3 — In Progress)

Three simultaneous changes:

### 1. Replace `ondurationchange` with a flag-guarded `ontimeupdate` seek trick

Use a `fixing_duration: RwSignal<bool>` flag shared between `onloadedmetadata` and `ontimeupdate`:

- In `onloadedmetadata`: if duration is Infinity, set `fixing_duration = true`, then call `set_current_time(1e9)`.
- In `ontimeupdate`: if `fixing_duration` is true, the browser has seeked to near EOF — use `currentTime` as the duration (with `duration()` as first preference if it's now finite), clear the flag, reset to 0. Otherwise update the position signal normally.

`ontimeupdate` is more reliable than `durationchange` for this: it fires when the browser has **actually moved** to a new playback position, regardless of whether the container has a seek index.

### 2. Pass `duration_hint_secs` from recording metadata

`RecordingMetadata.duration_ms` is the clock-measured duration from the recorder. It is accurate to within ~50ms and is available immediately. Passing it to `AudioPlayer` as an optional prop allows the player to display the correct duration and enable the correct slider range before the seek trick completes.

For a poetry reading app, 50ms precision is more than adequate.

### 3. Change `value=` to `prop:value=` on the seek slider

`prop:value=move || current_time.get().to_string()` sets the DOM `.value` property directly, correctly updating the slider thumb position in all browsers regardless of interaction history.

---

## Architecture Analysis

This component has required multiple fix iterations. The root causes reveal a pattern: the original implementation did not account for the specific properties of browser-recorded WebM audio. A correct HTML audio player for this use case requires:

| Concern | Correct approach |
|---------|-----------------|
| Duration for WebM | Hint from metadata (immediate) + seek trick (refinement) |
| Seek trick completion | `ontimeupdate` with guard flag, not `durationchange` |
| Slider position updates | `prop:value` (DOM property), not `value` (attribute) |
| Signal reads in DOM handlers | `.get_untracked()` only |
| Seek trick reset | Only reset playhead once, guarded by flag |

---

## Lessons Learned

### 1. MediaRecorder WebM files have no seek index

Chrome's MediaRecorder produces WebM files without EBML Cue entries. Cues are the seek index; without them, the browser cannot jump efficiently to an arbitrary position. `set_current_time(1e9)` may be silently deferred or ignored. The browser discovers the real duration only when it reads through the file sequentially to EOF.

**Rule:** For MediaRecorder WebM, don't rely on the seek trick alone. Always use a clock-measured hint from recording metadata as the primary source; use the seek trick only as a background refinement.

### 2. `durationchange` is not a reliable seek-completion signal for seekless WebM

`durationchange` fires when `HTMLMediaElement.duration` changes. For WebM without Cues, this may not fire until natural EOF during playback. Using it as the seek-trick completion signal creates a handler that fires during normal playback — where resetting `currentTime` breaks UX.

**Rule:** Use `timeupdate` with a boolean guard flag for seek-trick completion detection. `timeupdate` fires whenever the browser actually moves to a new position, including after seeks.

### 3. `value` attribute vs `prop:value` property on range inputs

HTML attribute `value` is the default/initial value. The DOM property `value` is the current displayed value. Once an input element is rendered, updating the attribute via `setAttribute` may not visually update the slider thumb in all browsers (especially after user interaction). Always use `prop:value` for reactive slider position tracking.

### 4. Browser-recorded media requires format-specific handling

Generic HTML audio player code assumes well-formed files (finite duration, seekable). Browser-recorded WebM/Opus from MediaRecorder is not well-formed by these standards. Any player designed to play back MediaRecorder output needs WebM-specific workarounds baked in, not added as patches.
