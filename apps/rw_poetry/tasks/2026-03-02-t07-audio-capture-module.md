# Task: T07 · Audio Capture Module

## Status
In Progress

## Goal
Implement the `audio_capture` module: microphone permission request, MIME type negotiation, MediaRecorder start/stop, blob collection, and microphone release.

## Scope
- `MicError` enum with display strings
- `request_mic()` — navigator.mediaDevices.getUserMedia
- `pick_mime_type()` — test fallback order, return best supported type
- `AudioRecorder` struct: start/stop, chunk collection
- `AudioBlob { data, mime_type, duration_ms }`
- Release mic tracks on stop

**Out of scope:** UI (T08).

## Design
Uses web_sys MediaDevices, MediaRecorder. Since MIME type checking
(MediaRecorder.isTypeSupported) is sync and available at any time, pick_mime_type
can be called before recording starts. start() creates the MediaRecorder and sets up
ondataavailable handler via wasm_bindgen Closure. stop() resolves a Promise via
wasm_bindgen_futures.

## Implementation Plan
1. [x] Define MicError and AudioBlob
2. [x] Implement request_mic() using JsFuture
3. [x] Implement pick_mime_type() with isTypeSupported fallback chain
4. [x] Implement AudioRecorder start/stop with Closure for data collection
5. [x] cargo check passes
6. [x] cargo clippy clean

## Testing Plan
- cargo check passes (WASM-only code, tests via wasm-pack test in browser)

## Notes / Decisions
- Closure::forget() used for ondataavailable — acceptable because the MediaRecorder
  lifetime is bounded by the recording session and we clean up on stop
- Error mapping follows spec section 16.5 error name strings
