# Task: T11 · Recording Detail View

## Status
In Progress

## Goal
Implement `RecordingDetailView` at `/readings/:recording_id` — poem text, metadata, AudioPlayer, download, back links.

## Scope
- Route: `/readings/:recording_id`
- Fetch RecordingMetadata by id from IDB; error state if not found
- Attempt to fetch poem content via `fetch_poem`; if unavailable show title/author only
- Render AudioPlayer with object URL (bytes → Blob → URL)
- Date recorded, duration display
- Download button (same pattern as T09)
- `← Read this poem` navigates to `/`
- `← All readings` link navigates to `/readings`

## Notes
- Use `use_params_map()` from leptos_router to get recording_id param
- Pass poem path from metadata (stored in RecordingMetadata) — BUT RecordingMetadata only stores title/author, not path. So fetch_poem needs a path we may not have. Fall back gracefully.
