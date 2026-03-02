# Task: T09 · Recordings List View

## Status
In Progress

## Goal
Implement `RecordingsListView` at `/readings` — load all recordings, render list, support inline play and download.

## Scope
- `LocalResource` calling `list_recordings()`, sorted newest-first
- Loading, error, empty states
- Each row: title link, date + duration, ▶ play button, ↓ download button
- Inline play: single active row at a time; fetch blob → object URL → AudioPlayer
- Download: fetch blob → `<a download>` click → revoke URL
- Filename: `poem-title_YYYY-MM-DD_HH-mm-ss.<ext>`

## Notes
- Uses T10 AudioPlayer component with `src` prop
- Active row stored in a signal; switching rows stops previous
