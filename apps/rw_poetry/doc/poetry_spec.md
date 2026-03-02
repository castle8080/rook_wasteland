# RW Poetry App Specification

## 1) Purpose
The RW Poetry app is a local-first poetry reader and voice-journal application.

Primary user value:
- Discover and read a random public-domain poem.
- Record a personal reading of a poem.
- Browse, replay, and export past recordings.

The app runs fully in the browser on the user’s machine (WASM frontend), with no required network services at runtime beyond loading static poem resources.

---

## 2) Product Goals
1. **Random poem discovery**: Let users quickly fetch and read a random poem.
2. **Simple reading experience**: Display poem metadata and clean formatted poem text.
3. **Voice recording**: Allow users to record audio while reading a selected poem.
4. **Persistent local library of readings**: Save recordings locally with poem reference + timestamp.
5. **Playback and export**: Replay recordings and download audio files.
6. **Offline-friendly behavior**: Operate locally after app assets/resources are available.

---

## 3) Non-Goals (Initial Version)
- User accounts or cloud sync.
- Social sharing, likes, comments, or collaboration.
- Audio transcription, AI analysis, or scoring.
- Editing/cropping audio recordings in-app.
- Server-side databases or APIs.

---

## 4) Target Tech Stack
- **Language**: Rust
- **Frontend framework**: Leptos
- **Runtime target**: WebAssembly (WASM)
- **Storage**: Browser local persistent storage (IndexedDB preferred for blob audio)
- **Audio capture/playback**: Browser media APIs (via Rust/WASM bindings)

---

## 4.1) Leptos Usage Guidance (Required Reference)
Implementation teams **must** use the guidance in:

- `doc/leptos_technical_design_principles_and_api_practices.md`

This reference document defines how Leptos should be used in this app, including technical design principles and API usage practices.

Notes:
1. Leptos APIs have been checked recently.
2. The reference file captures current best practices for using Leptos in this project.
3. Where this spec is less detailed, prefer the Leptos reference file for implementation-level decisions.

---

## 5) Core User Stories

### 5.1 Random Poem Reading
- As a user, I can request a random poem and immediately read it.
- As a user, I can see title, author, and date (if present), plus full poem text.

### 5.2 Recording a Reading
- As a user, I can record my voice while reading a poem.
- As a user, I can stop recording and save it.
- As a user, my recording remains available after page reload.

### 5.3 Reading History
- As a user, I can view a list of all saved readings.
- As a user, each entry shows poem title + recording date/time.
- As a user, selecting an entry shows details and the linked poem.

### 5.4 Playback and Download
- As a user, I can play back any saved recording.
- As a user, I can download the recording audio file locally.

---

## 6) Functional Requirements

### 6.1 Poem Catalog Loading
1. App loads a **poem index resource** containing available poem IDs and metadata pointers.
2. Canonical index location for the app: `GET /poems/poems_index.json`.
3. Index loading must be compatible with standard browser fetch/XHR semantics for static files.
4. Poem data loading must be local to the app deployment: resources are loaded from the same origin, using local/relative URLs where the app is served.
5. The app must not require external poem APIs or cross-origin poem fetches for core functionality.
6. App validates index shape and handles malformed entries gracefully.
7. App can randomly choose an item from the loaded index.
8. If index fails to load, app shows recoverable error UI and retry option.

### 6.2 Poem Detail Retrieval
1. Each poem is stored as a JSON resource retrievable by ID/path.
2. For a selected index item, app performs a second fetch/XHR call to the poem JSON path specified in the index.
3. Required poem fields:
   - `id` (string)
   - `title` (string)
   - `author` (string)
   - `content` (string or array of lines)
4. Optional fields:
   - `date` (string)
   - `source` (string)
   - `tags` (array of strings)
5. Poem rendering preserves line breaks/stanza spacing.

### 6.3 Randomization Behavior
1. “New Poem” action chooses a random poem from available index.
2. Random selection should be uniform across the active index.
3. App may avoid immediate repeat of the currently visible poem (nice-to-have).

### 6.4 Recording Flow
1. User can start recording only when a poem is selected.
2. App requests microphone permission using browser APIs.
3. On denial, app shows clear guidance and non-blocking fallback.
4. On stop, app stores audio blob + metadata locally.
5. Metadata includes:
   - recording ID
   - poem ID
   - poem title snapshot
   - recorded timestamp (ISO-8601)
   - duration (if available)
   - mime type
6. Multiple recordings per poem are supported.

### 6.5 Reading List and Detail
1. UI provides “Readings” list view.
2. List supports default sort by most recent first.
3. Each row shows poem title + recorded date/time.
4. Selecting a row opens detail view with:
   - poem metadata/content
   - recording metadata
   - playback controls
   - download action

### 6.6 Playback
1. Recording detail supports play/pause/seek (native control acceptable).
2. If blob missing/corrupt, app surfaces error and allows deletion.

### 6.7 Download/Export
1. User can download a stored recording as a local file.
2. Suggested filename format:
   - `poem-title_YYYY-MM-DD_HH-mm-ss.<ext>`
   - `<ext>` must match chosen recording MIME/container (e.g., `webm`, `mp4`, `ogg`).
3. Download uses browser object URL + anchor download mechanism.

### 6.8 Local Persistence
1. Audio and recording metadata persist across sessions.
2. Deleting browser storage removes recordings (expected behavior).
3. App should detect storage failures (quota/full/unsupported) and report them.

---

## 7) Suggested Data Contracts

### 7.1 Poem Index (example)
```json
{
  "version": 1,
  "poems": [
    {
      "id": "emily-dickinson-hope",
         "path": "/poems/authors/emily_dickinson/emily-dickinson-hope.json",
      "title": "Hope is the thing with feathers",
      "author": "Emily Dickinson"
    }
  ]
}
```

### 7.2 Poem JSON (example)
```json
{
  "id": "emily-dickinson-hope",
  "title": "Hope is the thing with feathers",
  "author": "Emily Dickinson",
  "date": "c. 1861",
  "content": "Hope is the thing with feathers...",
  "source": "Public Domain"
}
```

### 7.3 Recording Metadata (logical model)
```json
{
  "recording_id": "uuid",
  "poem_id": "emily-dickinson-hope",
  "poem_title": "Hope is the thing with feathers",
  "recorded_at": "2026-03-01T15:31:00Z",
  "duration_ms": 91342,
  "mime_type": "audio/webm",
  "audio_blob_key": "recording_blob_uuid"
}
```

### 7.4 Poetry Database Static Folder Layout (Technical Design)
The poetry corpus must be packaged as static JSON assets so the web app can retrieve data using browser HTTP requests (fetch/XHR) without any backend API.

Required layout:

```text
public/
   poems/
      poems_index.json
      authors/
         emily_dickinson/
            emily-dickinson-hope.json
         robert_frost/
            robert-frost-the-road-not-taken.json
```

**Trunk Integration Note:**
Trunk does not have a built-in "public" directory. To serve poem files at `/poems/...`, `index.html` must include:
```html
<link data-trunk rel="copy-dir" href="./public/poems"/>
```
This copies `public/poems/` into `dist/poems/`, making files available at `/poems/...`.
Using `href="./public"` instead would serve files at `/public/poems/...`, which breaks the required URLs.

Rules:
1. `poems_index.json` is the single entry point for poem discovery.
2. Every index entry includes a `path` value pointing to a poem JSON file under `/poems/...`.
3. The app data flow is two-step:
    - Step 1: `GET /poems/poems_index.json`
    - Step 2: `GET <entry.path>` for selected/random poem
4. All poem files must be individually addressable URLs so they can be requested independently.
5. Paths in index should be local to the app origin (same host as the app), and may be site-root absolute or relative, as long as they resolve from where the app is loaded.
6. File names should be stable, URL-safe slugs.
7. This structure must remain compatible with static hosting and local development servers.

Index example aligned to layout:

```json
{
   "version": 1,
   "poems": [
      {
         "id": "emily-dickinson-hope",
         "path": "/poems/authors/emily_dickinson/emily-dickinson-hope.json",
         "title": "Hope is the thing with feathers",
         "author": "Emily Dickinson"
      }
   ]
}
```

---

## 8) UI/UX Requirements

### 8.1 Main Views
1. **Home / Reader**
   - Current poem display
   - `New Poem` button
   - `Start Recording` / `Stop Recording`
2. **Readings List**
   - Table/list of readings by poem title and date
3. **Reading Detail**
   - Poem rendering
   - Recording info
   - Playback control
   - Download button

### 8.2 UX Behavior
- Show loading and error states for poem fetch.
- Show microphone permission state clearly.
- Show recording in-progress state (timer or indicator).
- Confirm destructive actions (e.g., delete recording, if implemented).

### 8.3 Accessibility
- Keyboard navigable controls.
- Semantic headings and landmarks.
- Visible focus indicators.
- Sufficient color contrast.
- Screen-reader friendly labels for recording and playback actions.

---

## 9) Privacy, Security, and Compliance
1. Audio recordings remain local by default.
2. No automatic upload or telemetry required for core function.
3. Poem corpus should be public-domain only; maintain attribution/source metadata where available.
4. Microphone access is only requested at recording time.

---

## 10) Performance and Reliability
1. Random poem display should feel instant after index load.
2. App should support at least hundreds of poem entries smoothly.
3. Recording save operation should handle transient failures gracefully.
4. Startup should recover previously stored recordings without blocking UI excessively.

---

## 11) Error Handling Requirements
- **Poem index unavailable**: show retry + message.
- **Poem JSON invalid/missing**: skip/bounce to another poem with warning.
- **Mic permission denied**: show actionable instructions.
- **Recording interrupted**: preserve what is available or show failure reason.
- **Storage quota exceeded**: warn user and suggest deleting recordings.

---

## 12) Suggested Rust/WASM Module Boundaries
1. `poem_repository`
   - Load/parse index
   - Fetch/parse poem JSON
   - Random selection logic
2. `audio_capture`
   - Mic permission
   - Start/stop recording
   - Blob conversion
3. `recording_store`
   - Persist metadata + blobs (IndexedDB)
   - Query list/detail
4. `ui`
   - Leptos components for reader/list/detail
   - State and event handling

---

## 13) Milestones

### Milestone 0: Poetry Database Buildout
- Define and create static poem folder layout under `public/poems/`
- Author `poems_index.json` as the authoritative index of all poems
- Add initial public-domain poem JSON files (one file per poem) organized under per-author subdirectories
- Validate that app can fetch index first, then fetch poem detail files via index `path`

### Milestone 1: Poem Reader Foundation
- Load poem index and poem JSON
- Render poem and metadata
- Add `New Poem` random action

### Milestone 2: Local Recording
- Implement mic access + start/stop recording
- Save recording metadata/blob locally

### Milestone 3: Readings Library
- Add list and detail views
- Add playback and download actions

### Milestone 4: Polish and Hardening
- Accessibility pass
- Error handling completeness
- Performance checks and offline behavior validation

---

## 14) Acceptance Criteria (v1)
1. User can open app and view a random poem from local/public-domain resources.
2. User can repeatedly request new random poems.
3. User can record voice for a poem and save recording.
4. Saved recording is present after reload.
5. User can view readings list grouped as entries with poem title/date.
6. User can open reading detail and play audio.
7. User can download audio for a reading.
8. Core experience works locally without backend services.

---

## 15) Future Extensions (Post-v1)
- Search poems by author/title/tag.
- Favorites and custom collections.
- Recording notes/tags.
- Import/export app backup bundle.
- Optional waveform visualization.
- Optional offline packaged poem corpus updates.

---

## 16) Feasibility and Standards Validation (Research Snapshot: 2026-03-01)

Conclusion: **Yes, this is feasible on modern browsers** for a local-first Leptos/WASM app.

### 16.1 Relevant Web Standards
- **Media Capture and Streams (`getUserMedia`)**: microphone access and audio stream capture.
- **MediaStream Recording (`MediaRecorder`)**: in-browser encoding of captured audio into blobs.
- **Permissions API**: optional preflight check of permission state (`granted` / `prompt` / `denied`).
- **Secure Contexts requirement**: media capture APIs are restricted to secure contexts.

Primary references:
- W3C Media Capture and Streams: https://www.w3.org/TR/mediacapture-streams/
- W3C MediaStream Recording: https://www.w3.org/TR/mediastream-recording/
- MDN `getUserMedia()`: https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia
- MDN `MediaRecorder`: https://developer.mozilla.org/en-US/docs/Web/API/MediaRecorder
- MDN Secure Contexts: https://developer.mozilla.org/en-US/docs/Web/Security/Defenses/Secure_Contexts
- MDN Permissions API: https://developer.mozilla.org/en-US/docs/Web/API/Permissions_API

### 16.2 Hard Requirements for Mic Capture
1. App must run in a **secure context**:
   - Production: `https://...`
   - Local dev: `http://localhost` / loopback is generally treated as trustworthy.
2. User must grant microphone permission; otherwise recording must fail gracefully.
3. If app is embedded in an iframe, host must allow `microphone` via permissions policy.
4. Permission prompts may be ignored by users (promise may remain pending); UI must support cancel/retry states.

### 16.3 Browser Capability Reality (High-level)
1. `getUserMedia({ audio: true })` is broadly available across major modern browsers.
2. `MediaRecorder` is broadly available in current Chrome/Edge/Firefox/Safari generations.
3. Codec/container support varies by browser; app must detect at runtime.

### 16.4 Required Runtime Compatibility Strategy
1. Before creating `MediaRecorder`, test preferred MIME types using `MediaRecorder.isTypeSupported()`.
2. Recommended fallback order for audio-only recording:
   - `audio/webm;codecs=opus`
   - `audio/webm`
   - `audio/mp4` (where supported)
   - `audio/ogg;codecs=opus` (where supported)
3. Persist the actual chosen MIME type in recording metadata.
4. Derive download file extension from chosen MIME type (`.webm`, `.mp4`, `.ogg`) instead of hardcoding `.webm`.

### 16.5 Error Cases Required by Spec and Platform Behavior
The app should explicitly handle these `getUserMedia()` failures:
- `NotAllowedError`: denied permission, insecure context, or policy block.
- `NotFoundError`: no input device matched requested constraints.
- `NotReadableError`: hardware/OS/browser-level device access failure.
- `OverconstrainedError`: impossible constraints.
- `AbortError` / `InvalidStateError` in edge cases.

For recording, handle `MediaRecorder` exceptions/events:
- `NotSupportedError` when requested MIME/container/codec is unsupported.
- Runtime `error` events (resource/encoder issues).

### 16.6 Local Storage Feasibility
1. Storing recorded audio blobs in **IndexedDB** is the practical standards-based approach.
2. Metadata may be in same IndexedDB store (recommended) keyed by `recording_id`.
3. Storage can fail due to quota; app must report and guide deletion/export.
4. Browser storage is user-clearable; this is expected and should be documented in UI/help.

### 16.7 Download/Export Feasibility
1. Downloading locally stored blobs is feasible via `URL.createObjectURL(blob)` and an `<a download>` flow.
2. “Download” in this project is therefore an **export from browser local storage to a user file**.
3. Filename should include poem title + timestamp + extension based on MIME type.

### 16.8 Privacy and UX Constraints to Encode in Product Behavior
1. Request microphone access only when user clicks **Start Recording**.
2. Show explicit state:
   - permission needed
   - recording active
   - recording stopped/saved
   - permission denied (with browser settings guidance)
3. Stop media tracks after recording (`track.stop()`) to release microphone and align with privacy indicators.

### 16.9 Rust/WASM + Leptos Implementation Feasibility Notes
1. Rust-to-browser bindings (`web-sys`/`wasm-bindgen`) can access `MediaDevices`, `MediaRecorder`, `Blob`, and IndexedDB APIs.
2. There is no standards blocker for implementing this fully client-side in Leptos WASM.
3. Main complexity is not capability, but robust browser-compat handling (MIME fallback + permission/error UX).

---

## 17) Poetry Corpus Reference

### 17.1 Public Domain Criteria

For this project, a poem is considered safe to include if it meets **either** of these conditions:

- **Published before January 1, 1928** in the United States (the current US public domain threshold for published works).
- **Author died more than 70 years ago** and the work was published during their lifetime (covers most international jurisdictions).

When in doubt, prefer works from authors who died before 1900, or explicitly pre-1928 publications. Always include source/attribution metadata in the poem JSON (`source`, `date` fields) to support compliance and attribution display.

---

### 17.2 Recommended Public Domain Sources

These sources can be used to obtain poem texts for corpus buildout:

| Source | URL | Notes |
|---|---|---|
| **Project Gutenberg** | https://www.gutenberg.org | Largest free ebook library; machine-readable plain text and HTML; covers most major poets; can bulk-download |
| **PoetryDB** | https://poetrydb.org | REST API returning JSON-formatted public domain poems; very convenient for seeding structured data; based on Gutenberg corpus |
| **Wikisource** | https://en.wikisource.org | Wiki-based collection; strong for British and American poetry; well-structured author pages |
| **Bartleby.com** | https://www.bartleby.com | Classic literature and poetry anthologies; includes Harvard Classics; good for browsing canonical collections |
| **Standard Ebooks** | https://standardebooks.org | High-quality, carefully typeset public domain ebooks; excellent for poets with full collected works |
| **Internet Archive** | https://archive.org | Scanned historical books; useful for rare or hard-to-find volumes |
| **A.V. Club / public anthology collections** | (various) | Pre-1928 anthologies like *The Oxford Book of English Verse* (1919 ed.) are themselves public domain and available on Gutenberg |

**PoetryDB is the most useful starting point** for Milestone 0 corpus work — it already returns structured JSON with title, author, and lines array, closely matching the poem data contract in section 7.2.

---

### 17.3 Considerations for Corpus Buildout

1. **Start with PoetryDB** as a seed source: query by author or title, transform the `lines` array into a `content` string, and write output to the static JSON layout.
2. **Normalize content format**: Poem text may arrive as an array of lines. Store as a single string with `\n` line breaks, or preserve the array — but pick one convention and apply it consistently across the corpus.
3. **Include attribution fields**: Always populate `author`, `date` (even approximate, e.g. `"c. 1819"`), and `source` (e.g. `"Project Gutenberg"` or `"PoetryDB"`).
4. **Use stable slug IDs**: Derive `id` and filename from author + title using URL-safe lowercased slugs (e.g. `john-keats-ode-to-a-nightingale`). Avoid numeric IDs that would break if order changes.
5. **Organize by author folder**: Even if only one poem exists for an author initially, use the per-author subdirectory structure from the start to keep the layout clean as corpus grows.
6. **Avoid very long poems for v1**: Multi-hundred-line poems (e.g. full *Song of Myself*, *The Rime of the Ancient Mariner*) are valid but make the reader UX heavier. Include short-to-medium poems preferentially; long poems can be added later.
7. **Aim for diversity**: Include a mix of tones (contemplative, joyful, melancholic), eras (Romantic, Victorian, American 19th-century), and authors to make random discovery interesting.

---

### 17.4 Seed Poem Reference List (~100 Poems)

This list is a reference guide for initial corpus population. All entries are believed to be US public domain (pre-1928 publication or author long deceased). Verify dates before publishing.

#### Emily Dickinson (1830–1886)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `emily-dickinson-hope-is-the-thing-with-feathers` | Hope is the thing with feathers | c. 1861 |
| `emily-dickinson-because-i-could-not-stop-for-death` | Because I could not stop for Death | c. 1863 |
| `emily-dickinson-i-heard-a-fly-buzz-when-i-died` | I heard a Fly buzz – when I died | c. 1862 |
| `emily-dickinson-im-nobody-who-are-you` | I'm Nobody! Who are you? | c. 1861 |
| `emily-dickinson-tell-all-the-truth-but-tell-it-slant` | Tell all the truth but tell it slant | c. 1868 |
| `emily-dickinson-this-is-my-letter-to-the-world` | This is my letter to the World | c. 1862 |
| `emily-dickinson-wild-nights-wild-nights` | Wild Nights – Wild Nights! | c. 1861 |
| `emily-dickinson-success-is-counted-sweetest` | Success is counted sweetest | c. 1859 |
| `emily-dickinson-after-great-pain-a-formal-feeling-comes` | After great pain, a formal feeling comes | c. 1862 |
| `emily-dickinson-the-brain-is-wider-than-the-sky` | The Brain—is wider than the Sky | c. 1862 |

#### Walt Whitman (1819–1892)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `walt-whitman-o-captain-my-captain` | O Captain! My Captain! | 1865 |
| `walt-whitman-a-noiseless-patient-spider` | A Noiseless Patient Spider | 1868 |
| `walt-whitman-beat-beat-drums` | Beat! Beat! Drums! | 1861 |
| `walt-whitman-crossing-brooklyn-ferry` | Crossing Brooklyn Ferry | 1856 |
| `walt-whitman-out-of-the-cradle-endlessly-rocking` | Out of the Cradle Endlessly Rocking | 1859 |
| `walt-whitman-i-sing-the-body-electric` | I Sing the Body Electric | 1855 |
| `walt-whitman-to-a-stranger` | To a Stranger | 1860 |

#### Robert Frost (1874–1963) — pre-1928 works only
| Slug ID | Title | Approx. Date |
|---|---|---|
| `robert-frost-the-road-not-taken` | The Road Not Taken | 1916 |
| `robert-frost-mending-wall` | Mending Wall | 1914 |
| `robert-frost-birches` | Birches | 1916 |
| `robert-frost-the-death-of-the-hired-man` | The Death of the Hired Man | 1914 |
| `robert-frost-after-apple-picking` | After Apple-Picking | 1914 |
| `robert-frost-fire-and-ice` | Fire and Ice | 1920 |
| `robert-frost-stopping-by-woods-on-a-snowy-evening` | Stopping by Woods on a Snowy Evening | 1923 |

#### Edgar Allan Poe (1809–1849)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `edgar-allan-poe-the-raven` | The Raven | 1845 |
| `edgar-allan-poe-annabel-lee` | Annabel Lee | 1849 |
| `edgar-allan-poe-the-bells` | The Bells | 1849 |
| `edgar-allan-poe-eldorado` | Eldorado | 1849 |
| `edgar-allan-poe-to-helen` | To Helen | 1831 |
| `edgar-allan-poe-a-dream-within-a-dream` | A Dream Within a Dream | 1849 |
| `edgar-allan-poe-ulalume` | Ulalume | 1847 |
| `edgar-allan-poe-the-haunted-palace` | The Haunted Palace | 1839 |

#### William Blake (1757–1827)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `william-blake-the-tyger` | The Tyger | 1794 |
| `william-blake-the-lamb` | The Lamb | 1789 |
| `william-blake-the-sick-rose` | The Sick Rose | 1794 |
| `william-blake-london` | London | 1794 |
| `william-blake-a-poison-tree` | A Poison Tree | 1794 |
| `william-blake-the-garden-of-love` | The Garden of Love | 1794 |
| `william-blake-auguries-of-innocence` | Auguries of Innocence | pub. 1863 |
| `william-blake-and-did-those-feet-in-ancient-time` | And did those feet in ancient time | 1808 |

#### John Keats (1795–1821)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `john-keats-ode-to-a-nightingale` | Ode to a Nightingale | 1819 |
| `john-keats-ode-on-a-grecian-urn` | Ode on a Grecian Urn | 1819 |
| `john-keats-to-autumn` | To Autumn | 1819 |
| `john-keats-la-belle-dame-sans-merci` | La Belle Dame sans Merci | 1819 |
| `john-keats-bright-star` | Bright Star | 1819 |
| `john-keats-ode-on-melancholy` | Ode on Melancholy | 1820 |
| `john-keats-when-i-have-fears-that-i-may-cease-to-be` | When I Have Fears That I May Cease to Be | 1818 |

#### Percy Bysshe Shelley (1792–1822)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `percy-shelley-ozymandias` | Ozymandias | 1818 |
| `percy-shelley-ode-to-the-west-wind` | Ode to the West Wind | 1820 |
| `percy-shelley-to-a-skylark` | To a Skylark | 1820 |
| `percy-shelley-the-cloud` | The Cloud | 1820 |
| `percy-shelley-loves-philosophy` | Love's Philosophy | 1820 |
| `percy-shelley-mutability` | Mutability | 1816 |

#### Lord Byron (1788–1824)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `lord-byron-she-walks-in-beauty` | She Walks in Beauty | 1814 |
| `lord-byron-when-we-two-parted` | When We Two Parted | 1816 |
| `lord-byron-so-well-go-no-more-a-roving` | So, We'll Go No More a Roving | 1830 |
| `lord-byron-the-destruction-of-sennacherib` | The Destruction of Sennacherib | 1815 |
| `lord-byron-darkness` | Darkness | 1816 |

#### Alfred Lord Tennyson (1809–1892)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `alfred-tennyson-the-charge-of-the-light-brigade` | The Charge of the Light Brigade | 1854 |
| `alfred-tennyson-ulysses` | Ulysses | 1842 |
| `alfred-tennyson-crossing-the-bar` | Crossing the Bar | 1889 |
| `alfred-tennyson-the-lady-of-shalott` | The Lady of Shalott | 1832 |
| `alfred-tennyson-break-break-break` | Break, Break, Break | 1842 |
| `alfred-tennyson-tears-idle-tears` | Tears, Idle Tears | 1847 |
| `alfred-tennyson-the-eagle` | The Eagle | 1851 |

#### William Wordsworth (1770–1850)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `william-wordsworth-i-wandered-lonely-as-a-cloud` | I Wandered Lonely as a Cloud | 1807 |
| `william-wordsworth-tintern-abbey` | Lines Written Above Tintern Abbey | 1798 |
| `william-wordsworth-composed-upon-westminster-bridge` | Composed upon Westminster Bridge | 1802 |
| `william-wordsworth-the-world-is-too-much-with-us` | The World Is Too Much with Us | 1807 |
| `william-wordsworth-she-dwelt-among-the-untrodden-ways` | She Dwelt Among the Untrodden Ways | 1800 |
| `william-wordsworth-lucy-gray` | Lucy Gray | 1800 |

#### Samuel Taylor Coleridge (1772–1834)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `samuel-coleridge-the-rime-of-the-ancient-mariner` | The Rime of the Ancient Mariner | 1798 |
| `samuel-coleridge-kubla-khan` | Kubla Khan | 1816 |
| `samuel-coleridge-frost-at-midnight` | Frost at Midnight | 1798 |

#### William Shakespeare (1564–1616) — Sonnets
| Slug ID | Title | Approx. Date |
|---|---|---|
| `william-shakespeare-sonnet-18` | Sonnet 18: Shall I compare thee to a summer's day? | 1609 |
| `william-shakespeare-sonnet-116` | Sonnet 116: Let me not to the marriage of true minds | 1609 |
| `william-shakespeare-sonnet-130` | Sonnet 130: My mistress' eyes are nothing like the sun | 1609 |
| `william-shakespeare-sonnet-73` | Sonnet 73: That time of year thou mayst in me behold | 1609 |
| `william-shakespeare-sonnet-29` | Sonnet 29: When, in disgrace with fortune and men's eyes | 1609 |

#### Henry Wadsworth Longfellow (1807–1882)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `henry-longfellow-a-psalm-of-life` | A Psalm of Life | 1838 |
| `henry-longfellow-paul-reveres-ride` | Paul Revere's Ride | 1860 |
| `henry-longfellow-the-village-blacksmith` | The Village Blacksmith | 1840 |
| `henry-longfellow-the-tide-rises-the-tide-falls` | The Tide Rises, the Tide Falls | 1880 |
| `henry-longfellow-excelsior` | Excelsior | 1841 |

#### Christina Rossetti (1830–1894)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `christina-rossetti-remember` | Remember | 1862 |
| `christina-rossetti-a-birthday` | A Birthday | 1861 |
| `christina-rossetti-when-i-am-dead-my-dearest` | When I am dead, my dearest | 1848 |
| `christina-rossetti-up-hill` | Up-Hill | 1858 |

#### Elizabeth Barrett Browning (1806–1861)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `elizabeth-barrett-browning-how-do-i-love-thee` | How Do I Love Thee? (Sonnet 43) | 1850 |
| `elizabeth-barrett-browning-grief` | Grief | 1844 |

#### Robert Browning (1812–1889)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `robert-browning-my-last-duchess` | My Last Duchess | 1842 |
| `robert-browning-porphyrias-lover` | Porphyria's Lover | 1836 |
| `robert-browning-the-pied-piper-of-hamelin` | The Pied Piper of Hamelin | 1842 |

#### Gerard Manley Hopkins (1844–1889)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `gerard-manley-hopkins-the-windhover` | The Windhover | written 1877, pub. 1918 |
| `gerard-manley-hopkins-gods-grandeur` | God's Grandeur | written 1877, pub. 1918 |
| `gerard-manley-hopkins-pied-beauty` | Pied Beauty | written 1877, pub. 1918 |
| `gerard-manley-hopkins-spring-and-fall` | Spring and Fall | written 1880, pub. 1918 |

#### Rudyard Kipling (1865–1936) — pre-1928 works only
| Slug ID | Title | Approx. Date |
|---|---|---|
| `rudyard-kipling-if` | If— | 1910 |
| `rudyard-kipling-gunga-din` | Gunga Din | 1890 |
| `rudyard-kipling-the-road-to-mandalay` | The Road to Mandalay | 1890 |
| `rudyard-kipling-recessional` | Recessional | 1897 |

#### A.E. Housman (1859–1936) — A Shropshire Lad (1896)
| Slug ID | Title | Approx. Date |
|---|---|---|
| `ae-housman-when-i-was-one-and-twenty` | When I Was One-and-Twenty | 1896 |
| `ae-housman-to-an-athlete-dying-young` | To an Athlete Dying Young | 1896 |
| `ae-housman-loveliest-of-trees-the-cherry-now` | Loveliest of trees, the cherry now | 1896 |
| `ae-housman-with-rue-my-heart-is-laden` | With rue my heart is laden | 1896 |

#### W.B. Yeats (1865–1939) — pre-1928 works only
| Slug ID | Title | Approx. Date |
|---|---|---|
| `wb-yeats-the-lake-isle-of-innisfree` | The Lake Isle of Innisfree | 1890 |
| `wb-yeats-when-you-are-old` | When You Are Old | 1892 |
| `wb-yeats-the-second-coming` | The Second Coming | 1919 |
| `wb-yeats-easter-1916` | Easter, 1916 | 1916 |
| `wb-yeats-the-wild-swans-at-coole` | The Wild Swans at Coole | 1919 |

---

### 17.5 Corpus Build Notes

- **The seed list is a starting point, not a ceiling.** The ~100 poems across 19 authors in section 17.4 exist to guide initial corpus work and validate the pipeline. The actual corpus buildout should aim for significantly more — hundreds to thousands of poems. The public domain contains an enormous wealth of poetry; sources like Project Gutenberg and PoetryDB alone can yield many thousands of eligible poems. The exact target size should be decided during Milestone 0 corpus work based on available tooling and effort, but the architecture (static JSON files, index-based discovery) scales to any size without changes.
- **Frost and Kipling caution**: Only include works published before 1928. Both authors lived past 1928 and have later works that may still be under copyright.
- **Yeats caution**: Only include works published before 1928 (as listed above). *Sailing to Byzantium* (1928) and later works remain under copyright in some jurisdictions.
- **Hopkins note**: Hopkins died in 1889 but most poems were first published posthumously in 1918 — still pre-1928 and safe to include.
- **Shakespeare sonnets**: All 154 sonnets were published in 1609; all are fully public domain worldwide.
- **Long-form works**: Poems like *The Rime of the Ancient Mariner*, *Tintern Abbey*, and *The Song of Hiawatha* are long. Consider including them but be mindful of reading UX. They can be included as valid corpus entries for completeness even if less ideal for voice recording sessions.

### 16.10 Updated Acceptance Addendum (Recording-specific)
Recording feature is considered acceptable only if:
1. It works in secure context environments.
2. It gracefully handles denied/blocked/unavailable mic conditions.
3. It records and replays audio using a browser-supported MIME type discovered at runtime.
4. It persists recording blobs + metadata locally and survives reload.
5. It exports downloadable audio with a correct extension for the actual MIME type.
