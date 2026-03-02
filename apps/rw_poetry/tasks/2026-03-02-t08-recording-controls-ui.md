# Task: T08 · Recording Controls UI

## Status
In Progress

## Goal
Add the record/stop/saved control bar to the reader view, wired to the audio_capture and recording_store modules, with three-state UI (Idle, Recording, Saved).

## Scope
- `RecordingControls` component below poem text in ReaderView
- Props: `poem_id: String`, `poem_title: String`, `poem_author: String`
- Three states via local signal: Idle, Recording, Saved
- On Record: request_mic → start AudioRecorder → Recording state
- On Stop: stop recorder → save_recording → Saved state → reset to Idle
- Live elapsed timer in Recording state
- Mic denied: inline error message (no modal)
- Storage full: inline warning

**Out of scope:** Error handling completeness audit (T12).

## Implementation Plan
1. [x] Define RecordingState enum
2. [x] Implement RecordingControls component
3. [x] Wire into ReaderView replacing the placeholder div
4. [x] trunk build passes, cargo clippy clean

## Testing Plan
- Manual browser test: record ~5s clip, reload, no crash. Saved confirmation appears.

## Notes / Decisions
- AudioRecorder is not Send, so stored in Rc<RefCell<Option<AudioRecorder>>>
- Timer uses gloo_timers interval + spawn_local for elapsed display
- UUID generation via uuid crate with js feature
