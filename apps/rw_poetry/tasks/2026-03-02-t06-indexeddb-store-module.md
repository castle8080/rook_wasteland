# Task: T06 · IndexedDB Store Module

## Status
In Progress

## Goal
Implement the `recording_store` module: open `rw_poetry_db` v1, define `RecordingMetadata`, and provide async CRUD operations for recordings and audio blobs.

## Scope
- `RecordingMetadata` struct with Serialize/Deserialize
- `StoreError` enum: NotFound, StorageFull, Unexpected(String)
- `open_db()` — opens rw_poetry_db v1 with recordings + audio_blobs stores
- `save_recording(metadata, audio_data)` — write blob then metadata
- `list_recordings()` — sorted newest-first
- `get_recording(id)` — single metadata fetch
- `get_audio_blob(blob_key)` — fetch raw bytes
- `delete_recording(id, blob_key)` — blob first, then metadata

**Out of scope:** UI (T08).

## Design
Uses the `idb` crate for typed async IndexedDB access. Audio is stored as ArrayBuffer via `serde_wasm_bindgen`. The db is opened once and the handle reused via a helper. `serde_json` serializes RecordingMetadata to JsValue via serde_wasm_bindgen.

## Implementation Plan
1. [x] Define RecordingMetadata and StoreError in recording_store/mod.rs
2. [x] Implement open_db()
3. [x] Implement save_recording, list_recordings, get_recording
4. [x] Implement get_audio_blob, delete_recording
5. [x] Unit tests: RecordingMetadata serde round-trip, StoreError display
6. [x] cargo test passes
7. [x] cargo clippy clean

## Testing Plan
- Unit test: RecordingMetadata serde round-trip
- Unit test: StoreError display strings

## Notes / Decisions
- Audio stored as Vec<u8> on Rust side, converted to Uint8Array for IDB storage
- Using serde_wasm_bindgen for Rust↔JsValue conversion of RecordingMetadata
