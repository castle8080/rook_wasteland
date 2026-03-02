# RW Poetry — Implementation Task Plan

> **Status note:** Milestone 0 (poetry corpus buildout) is **complete**. This plan covers
> Milestones 1–4: the full Leptos/WASM application, from blank Cargo project to a
> polished, acceptance-tested v1.

Refer to `doc/poetry_spec.md` for all functional requirements, data contracts, and UI spec.  
Refer to `doc/leptos_technical_design_principles_and_api_practices.md` for all Leptos API decisions.  
Refer to `doc/building.md` for the process rules every task must follow (task docs, checks, commits).

---

## Overview — Milestone Map

| Milestone | Theme | Tasks |
|---|---|---|
| 1 | Poem Reader Foundation | T01–T05 |
| 2 | Local Recording | T06–T08 |
| 3 | Readings Library | T09–T11 |
| 4 | Polish and Hardening | T12–T14 |

Dependencies flow top-to-bottom within each milestone. Cross-milestone: M2 depends on M1, M3 depends on M2, M4 depends on M1–M3.

---

## Milestone 1 — Poem Reader Foundation

---

### T01 · Project Scaffold

**Goal:** Stand up the Rust/Leptos/Trunk project with correct structure, dependencies, and a "hello world" build.

**Scope:**
- Create `Cargo.toml` with all crates from spec section 4.2
- Create `index.html` with Trunk directives including `copy-dir` for poems
- Create `src/main.rs` that mounts the Leptos app
- Verify `trunk build` and `trunk serve` produce a working page
- Set up `src/` module layout matching spec section 12

**Out of scope:** Any real UI or logic beyond a blank app shell.

**Module layout to create:**
```
src/
  main.rs               -- app entry point, mount
  app.rs                -- top-level App component + router
  poem_repository/
    mod.rs
  audio_capture/
    mod.rs
  recording_store/
    mod.rs
  ui/
    mod.rs
    reader.rs
    recordings_list.rs
    recording_detail.rs
    components/
      mod.rs
```

**Checks before commit:**
- `cargo check` passes
- `trunk build` succeeds with no errors
- Browser opens, shows blank page without JS console errors

**Task doc:** `tasks/YYYY-MM-DD-t01-project-scaffold.md`

---

### T02 · Core Data Types and Poem Repository Module

**Goal:** Define all shared Rust types and implement the two-step poem fetch flow (index → poem JSON).

**Scope:**
- Define `PoemIndexEntry { id, path, title, author }` with `serde::Deserialize`
- Define `PoemIndex { version, poems: Vec<PoemIndexEntry> }` with `serde::Deserialize`
- Define `PoemDetail { id, title, author, content, date?, source?, tags? }` with `serde::Deserialize`
- Implement `poem_repository` module:
  - `async fn fetch_index() -> Result<PoemIndex, String>` — `GET /poems/poems_index.json` via `gloo-net`
  - `async fn fetch_poem(path: &str) -> Result<PoemDetail, String>` — `GET {path}` via `gloo-net`
  - `fn pick_random(index: &PoemIndex, exclude_id: Option<&str>) -> Option<&PoemIndexEntry>` — uniform random, with optional exclude-current
- Unit tests:
  - `pick_random` on a constructed index: uniform distribution, exclude logic
  - Deserialize a sample `PoemIndex` JSON string
  - Deserialize a sample `PoemDetail` JSON string

**Checks before commit:**
- `cargo test` all tests pass
- `cargo clippy -- -D warnings` clean

**Task doc:** `tasks/YYYY-MM-DD-t02-data-types-poem-repository.md`

---

### T03 · App Shell, Routing, and Top Bar

**Goal:** Wire up Leptos router with two routes (reader `"/"` and readings list `"/readings"`), render the navigation top bar.

**Scope:**
- `App` component with `leptos_router::Router` and two `Route` entries
- `TopBar` component: app name left, `Recordings` link right
- Apply base CSS: custom properties for light mode palette (spec section 8.4), base typography, body layout
- `index.html` sets viewport meta, loads compiled WASM
- Navigation top bar is present on both routes

**CSS approach:**
- Use CSS custom properties (`--color-bg`, `--color-text`, `--color-accent`, etc.) set on `:root` for light mode; override in `[data-theme="dark"]` for dark mode
- One stylesheet, two themes controlled by a root attribute

**Out of scope:** Dark mode toggle interaction (T14), actual page content in either view (T04, T09).

**Checks before commit:**
- `trunk build` succeeds
- Browser: top bar renders, clicking `Recordings` navigates to `/readings`, browser back returns to `/`
- No console errors

**Task doc:** `tasks/YYYY-MM-DD-t03-app-shell-routing-topbar.md`

---

### T04 · Poem Reader View

**Goal:** Implement the full poem reader home view — load a random poem on arrival, display it, handle loading/error states, provide `New Poem` button.

**Scope:**
- `ReaderView` component:
  - On mount, call `poem_repository::fetch_index()` via `LocalResource`
  - Pick a random poem from the index, then call `poem_repository::fetch_poem(path)` via `LocalResource`
  - Render: title (`h1`), author + date (secondary text), poem body (`pre`-like with `white-space: pre-wrap`, serif font)
  - Loading state: inline spinner or placeholder text (`Loading poem…`)
  - Error state: error message + `Try again` button that re-triggers fetch
  - `New Poem` button: picks a new random poem excluding current, re-fetches
- Apply typography spec: `1.1rem`–`1.2rem`, line-height `1.7`–`1.8`, `max-width: 38em`, centered column
- Poem body uses proportional serif font; no monospace

**Out of scope:** Recording controls (T07). The recording control area can be a placeholder `div` for now.

**Checks before commit:**
- `trunk build` passes
- Browser: poem loads on open, `New Poem` loads a different poem, loading and error states visible

**Task doc:** `tasks/YYYY-MM-DD-t04-poem-reader-view.md`

---

### T05 · CSS Baseline and Design System

**Goal:** Complete the full CSS design system — light/dark palettes, all custom properties, typography scale, layout primitives, button styles — before recording or list UI is built.

**Scope:**
- All colors from spec section 8.4 as CSS custom properties for both modes
- Typography: body font stack (Georgia/serif fallback), UI chrome font stack (system-ui/sans-serif), scale
- Button styles: primary (accent), secondary (ghost/muted), icon-button — all inherit from a base `.btn` class
- Layout utilities: centered column, surface card, divider line, top bar layout
- Focus styles: visible keyboard focus ring using accent color
- Responsive: everything must be comfortable on narrow viewports (phone width ~375px)

**Out of scope:** Dark mode toggle behavior (T14). CSS is written; activation is wired in T14.

**Checks before commit:**
- `cargo clippy` clean (no Rust changes, but confirm)
- Visual review in browser: light mode renders correctly, all button variants visible

**Task doc:** `tasks/YYYY-MM-DD-t05-css-design-system.md`

---

## Milestone 2 — Local Recording

---

### T06 · IndexedDB Store Module

**Goal:** Implement the `recording_store` module — open `rw_poetry_db`, and provide async CRUD operations for recordings and audio blobs.

**Scope:**
- Database initialization: open `rw_poetry_db` v1 using `idb` crate; create `recordings` and `audio_blobs` stores with keyPaths and indices per spec section 7.4
- Define `RecordingMetadata` struct matching spec section 7.3 (all fields, `Serialize`/`Deserialize`)
- Implement:
  - `async fn save_recording(metadata: RecordingMetadata, audio_data: Vec<u8>) -> Result<(), StoreError>`
    - Write blob first (`audio_blobs`), then metadata (`recordings`). If either fails, clean up the other.
  - `async fn list_recordings() -> Result<Vec<RecordingMetadata>, StoreError>` — sorted newest-first by `recorded_at`
  - `async fn get_recording(recording_id: &str) -> Result<RecordingMetadata, StoreError>`
  - `async fn get_audio_blob(blob_key: &str) -> Result<Vec<u8>, StoreError>`
  - `async fn delete_recording(recording_id: &str, blob_key: &str) -> Result<(), StoreError>`
    - Delete blob first, then metadata.
- `StoreError` enum: `NotFound`, `StorageFull`, `Unexpected(String)`
- Unit tests: deserialize `RecordingMetadata` round-trip; `StoreError` display strings

**Note:** WASM integration tests for actual IndexedDB calls require `wasm-pack test`. The pure logic (struct serde, error types) can be tested with `cargo test`.

**Checks before commit:**
- `cargo test` passes (serde round-trips)
- `cargo clippy -- -D warnings` clean

**Task doc:** `tasks/YYYY-MM-DD-t06-indexeddb-store-module.md`

---

### T07 · Audio Capture Module

**Goal:** Implement the `audio_capture` module — mic permission request, MediaRecorder start/stop, MIME type negotiation, blob collection.

**Scope:**
- `async fn request_mic() -> Result<MediaStream, MicError>` — calls `navigator.mediaDevices.getUserMedia({ audio: true })`
- `fn pick_mime_type() -> &'static str` — test the fallback order from spec section 16.4 using `MediaRecorder.isTypeSupported()`; return best supported type
- `AudioRecorder` struct (or equivalent) managing a `MediaRecorder` instance:
  - `fn start(stream: &MediaStream, mime_type: &str) -> Result<Self, MicError>`
  - `fn stop(self) -> impl Future<Output = Result<AudioBlob, MicError>>` — collects all `dataavailable` chunks into a single `Vec<u8>`; captures final duration
- `AudioBlob { data: Vec<u8>, mime_type: String, duration_ms: Option<u64> }`
- `MicError` enum: `PermissionDenied`, `NoDevice`, `HardwareError`, `NotSupported`, `Unexpected(String)` — maps from `getUserMedia` / `MediaRecorder` error names per spec section 16.5
- After stop: call `track.stop()` on all stream tracks to release the microphone (spec section 16.8)

**Out of scope:** UI (T08). This module is pure logic + browser API bindings.

**Checks before commit:**
- `cargo check` passes
- `cargo clippy -- -D warnings` clean
- Manual smoke: `wasm-pack test --headless` (or manual browser test in T08)

**Task doc:** `tasks/YYYY-MM-DD-t07-audio-capture-module.md`

---

### T08 · Recording Controls UI

**Goal:** Add the record/stop/saved control bar to the reader view, wire it to T06 + T07, implement the three-state UI from spec section 8.6.

**Scope:**
- `RecordingControls` component, placed below poem text in `ReaderView`
- Receives current `poem_id`, `poem_title`, `poem_author` as props
- Three states driven by a local signal:
  - **Idle**: `⏺ Record` button (muted neutral style)
  - **Recording**: `⏹ Stop` button (accent color) + live elapsed timer updating every second via `gloo-timers`
  - **Saved**: `✓ Saved` inline confirmation for ~2s, then resets to Idle
- On `⏺ Record` click:
  1. Call `audio_capture::request_mic()` — if denied, show inline error per spec section 8.6; remain in Idle
  2. Start `AudioRecorder`, transition to Recording state
- On `⏹ Stop` click:
  1. Stop recorder → `AudioBlob`
  2. Generate UUIDs for `recording_id` and `audio_blob_key`
  3. Build `RecordingMetadata` (snapshot poem title/author, timestamp, mime_type, duration)
  4. Call `recording_store::save_recording(...)` — if quota error, show inline warning
  5. Transition to Saved state, then reset to Idle
- Mic permission denial: replace Record button with one-line message + link text (not a modal)

**Checks before commit:**
- `trunk build` passes
- Manual browser test: record a ~5s clip, reload page, no crash. Saved confirmation appears.
- `cargo clippy -- -D warnings` clean

**Task doc:** `tasks/YYYY-MM-DD-t08-recording-controls-ui.md`

---

## Milestone 3 — Readings Library

---

### T09 · Recordings List View

**Goal:** Implement the `RecordingsListView` at `/readings` — load all recordings from IndexedDB, render the journal-style list, support inline play and download.

**Scope:**
- `RecordingsListView` component:
  - On mount: `LocalResource` calling `recording_store::list_recordings()` sorted newest-first
  - Loading and error states (same pattern as poem reader)
  - Empty state: friendly message if no recordings yet
  - Each row (spec section 8.7):
    - Poem title (link → `/readings/{recording_id}`) — primary text
    - Date (formatted as `Mar 1, 2026`) + duration (formatted as `0:43`) — secondary muted text
    - `▶` play icon button — triggers inline playback (see below)
    - `↓` download icon button — triggers download without navigation
- Inline play in list:
  - Fetch audio blob for that row's `blob_key` on demand
  - Create object URL → pass to a shared mini `<audio>` element or a singleton `AudioPlayer` signal
  - Allow only one row to play at a time; clicking `▶` on a new row stops the previous
- Download action:
  - Fetch blob, create object URL, trigger `<a download href="...">` click programmatically, revoke URL
  - Filename per spec section 6.7: `poem-title_YYYY-MM-DD_HH-mm-ss.<ext>`

**Checks before commit:**
- `trunk build` passes
- Browser: list shows after recording in T08; play and download work; empty state renders

**Task doc:** `tasks/YYYY-MM-DD-t09-recordings-list-view.md`

---

### T10 · Custom Audio Player Component

**Goal:** Build the reusable `AudioPlayer` component per spec section 8.10 — hidden `<audio>` element + custom Leptos controls.

**Scope:**
- `AudioPlayer` component accepting a `blob_key: String` prop (fetches own audio data) or a `src: String` prop (object URL already resolved)
- Internal signals: `playing: bool`, `current_time: f64`, `duration: f64`, `loaded: bool`
- `NodeRef<html::Audio>` bound to a hidden `<audio>` element
- Wire DOM events to signals via `Effect`:
  - `ontimeupdate` → update `current_time`
  - `onloadedmetadata` → set `duration`; set `loaded = true`
  - `onended` → set `playing = false`
- Custom controls rendered by Leptos (spec section 8.10 visual layout):
  - Play/Pause button: calls `audio.play()` or `audio.pause()` via `spawn_local + JsFuture`
  - Elapsed time display: `format_duration(current_time)` e.g. `1:23`
  - Seek bar: `<input type="range">` — on input, calls `audio.set_current_time(value)`; value derived from `current_time` signal
  - Total time display: `format_duration(duration)` — shows `0:00` until metadata loads
- Cleanup on unmount: revoke object URL via `on_cleanup`
- Unit tests:
  - `format_duration(0.0)` → `"0:00"`
  - `format_duration(91.5)` → `"1:31"`
  - `format_duration(3661.0)` → `"61:01"` (no hour formatting needed for poems)

**Checks before commit:**
- `cargo test` (duration formatting)
- `trunk build` passes
- Browser: component renders, play/pause works, seek bar responds, duration displays correctly

**Task doc:** `tasks/YYYY-MM-DD-t10-audio-player-component.md`

---

### T11 · Recording Detail View

**Goal:** Implement `RecordingDetailView` at `/readings/{recording_id}` — poem text + recording metadata + audio player + download + back link.

**Scope:**
- Route: `/readings/:recording_id`
- `RecordingDetailView` component:
  - Fetch `RecordingMetadata` from IndexedDB by `recording_id` param
  - Render poem content: `poem_title` as `h1`, `poem_author` as secondary, full `content` from a second fetch to poem JSON via `poem_repository::fetch_poem`
    - If the poem JSON is unavailable (corpus reorganized), show the snapshotted title/author with a note that full text is unavailable — do not error the whole view
  - Render `AudioPlayer` component (T10) with `blob_key`
  - Recording metadata section: date recorded (formatted), duration
  - Download button (same logic as T09's download action)
  - `← Read this poem` link: navigates to `"/"` with the poem pre-loaded (pass poem path via router state or query param)
  - Error state if `recording_id` not found: friendly message + link back to `/readings`

**Checks before commit:**
- `trunk build` passes
- Browser: navigate from list to detail, poem text renders, player plays, download works, back link works

**Task doc:** `tasks/YYYY-MM-DD-t11-recording-detail-view.md`

---

## Milestone 4 — Polish and Hardening

---

### T12 · Error Handling Completeness

**Goal:** Audit every user-facing error path from spec section 11 and ensure all are implemented with the correct message and recovery action.

**Error paths to verify:**

| Condition | Expected behavior |
|---|---|
| `GET /poems/poems_index.json` fails | Retry button + `Unable to load poems. Check your connection.` |
| Poem JSON 404 or malformed | Skip to another random poem; brief inline warning |
| Mic `NotAllowedError` | Inline text: `Microphone access was denied. [Open browser settings]` |
| Mic `NotFoundError` | Inline: `No microphone found on this device.` |
| Mic `NotReadableError` | Inline: `Could not access the microphone. Another app may be using it.` |
| `MediaRecorder` `NotSupportedError` | Inline: `This browser doesn't support audio recording.` |
| IndexedDB write `QuotaExceededError` | Warning: `Storage full. Delete some recordings to free space.` |
| Audio blob missing on playback | Error in player: `Recording data unavailable.` + delete option |
| IndexedDB completely unavailable | Graceful fallback: poem reading still works; recordings are disabled with explanation |

**Scope:** No new features — only audit and fill in gaps in existing error handling from T06–T11.

**Checks before commit:**
- Manually trigger each error condition (DevTools: throttle network, block mic, clear IDB)
- `cargo clippy -- -D warnings` clean

**Task doc:** `tasks/YYYY-MM-DD-t12-error-handling-completeness.md`

---

### T13 · Accessibility Pass

**Goal:** Ensure the full app meets the accessibility requirements in spec section 8.9.

**Scope:**
- Audit all interactive controls for keyboard navigability (`Tab`, `Enter`, `Space`)
- Confirm all interactive elements have accessible labels (`aria-label` where the visible label is an icon only — e.g. `▶`, `↓`, `⏺`, `⏸`)
- Confirm semantic HTML: `<nav>`, `<main>`, `<h1>`/`<h2>` hierarchy, `<button>` for actions (not `<div>`)
- Confirm focus styles are visible (not suppressed)
- Confirm color contrast: body text and accent on both light and dark backgrounds must meet WCAG AA (4.5:1 for normal text)
- Confirm the recording elapsed timer announces state changes in a screen-reader friendly way (`aria-live`)
- Confirm poem content area has `lang="en"` (or inherits from `<html lang="en">`)

**Tools:** Browser DevTools accessibility tree, keyboard-only navigation test, contrast checker.

**Checks before commit:**
- Full keyboard navigation walkthrough without a mouse
- `cargo clippy -- -D warnings` clean

**Task doc:** `tasks/YYYY-MM-DD-t13-accessibility-pass.md`

---

### T14 · Dark Mode Toggle and Final Polish

**Goal:** Wire the dark mode toggle, validate offline behavior, final visual pass, acceptance criteria sign-off.

**Scope:**
- Dark mode toggle in `TopBar`:
  - Read initial preference from `gloo-storage::LocalStorage` key `"theme"` (fallback: `"light"`)
  - Toggle button (moon/sun icon or text label)
  - On toggle: update `[data-theme]` attribute on `<html>` element; persist to `LocalStorage`
  - CSS already written in T05 — this task only wires the interaction
- Offline behavior validation:
  - With DevTools offline mode: already-loaded poems still render; navigation between reader and recordings still works; recordings saved before going offline are accessible
  - App does not crash when fetch fails due to being offline (covered by T12 error handling)
- Visual final pass:
  - Comfortable rendering at 375px (phone), 768px (tablet), 1280px (desktop)
  - Poem text line lengths respect `max-width: 38em` at all sizes
  - No clipped text, no broken layouts, no misaligned controls
- Acceptance criteria sign-off (spec section 14): manually verify all 8 acceptance criteria pass
- Clean up: remove any `console_log!` debug calls, any placeholder/TODO comments in UI

**Checks before commit:**
- All 8 acceptance criteria manually verified
- `trunk build` release build: `trunk build --release`
- `cargo clippy -- -D warnings` clean
- `cargo fmt --check` clean

**Task doc:** `tasks/YYYY-MM-DD-t14-dark-mode-polish-acceptance.md`

---

## Task Dependency Graph

```
T01 (scaffold)
  └─► T02 (data types + repo)
        └─► T03 (shell + routing)
              ├─► T04 (reader view)
              │     └─── needs T02
              └─► T05 (CSS design system)
                    └─── feeds into T04, T08, T09, T10, T11

T04 + T05 complete → Milestone 1 done

T06 (IndexedDB store)         ← depends on T02 (RecordingMetadata type)
T07 (audio capture)           ← depends on T01 (project compiles)
T08 (recording controls UI)   ← depends on T04 (reader view) + T06 + T07

T06 + T07 + T08 complete → Milestone 2 done

T09 (recordings list)         ← depends on T06 + T08 + T10
T10 (audio player component)  ← depends on T05 (CSS) + T06
T11 (recording detail)        ← depends on T09 + T10 + T02

T09 + T10 + T11 complete → Milestone 3 done

T12 (error handling)          ← depends on M1 + M2 + M3
T13 (accessibility)           ← depends on M1 + M2 + M3
T14 (dark mode + polish)      ← depends on T12 + T13

T12 + T13 + T14 complete → Milestone 4 done → v1 ship
```

---

## Pre-Task Checklist (copy into each task doc)

Before writing any code for a task, confirm:
- [ ] Re-read the relevant spec sections
- [ ] Task document created in `tasks/` with Goal, Scope, Design, Implementation Plan, Testing Plan
- [ ] Dependencies (prior tasks) are complete and committed

Before committing:
- [ ] `cargo fmt`
- [ ] `cargo clippy -- -D warnings` → zero warnings
- [ ] `cargo test` → all pass
- [ ] `trunk build` → succeeds
- [ ] Self-critique checklist from `doc/building.md` section 5 completed
- [ ] Task doc updated to `Status: Complete` with Completion Notes
- [ ] Commit message uses conventional format with Co-authored-by trailer
