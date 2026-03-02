# Task: T10 · Custom Audio Player Component

## Status
In Progress

## Goal
Build a reusable `AudioPlayer` component with custom controls (play/pause, seek bar, time display) backed by a hidden `<audio>` element.

## Scope
- Props: `src: String` (object URL already resolved)
- Hidden `<audio>` + `NodeRef<html::Audio>`
- Signals: `playing`, `current_time`, `duration`, `loaded`
- Events wired via Effect: `ontimeupdate`, `onloadedmetadata`, `onended`
- Custom controls: play/pause button, elapsed, seek bar, total time
- Cleanup: revoke object URL on_cleanup
- Unit tests: `format_duration(0.0)` → `"0:00"`, `format_duration(91.5)` → `"1:31"`, `format_duration(3661.0)` → `"61:01"`

## Notes
- Use `spawn_local + JsFuture` for `audio.play()`
- `on_cleanup` registered inside component to revoke URL
