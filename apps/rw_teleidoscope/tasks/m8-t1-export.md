# Task M8-T1: Export / Download

**Milestone:** M8 — Export / Download  
**Status:** 🔄 In Progress

## Restatement

Implement a canvas export feature that lets the user download the current
kaleidoscope view as PNG, JPEG, or WebP.  The feature lives entirely in
`src/components/export_menu.rs` and is wired into `src/components/controls_panel.rs`
at the bottom of the side panel.  `ExportMenu` reads `AppState.image_loaded` to
disable the trigger button before any image is loaded, reads
`KaleidoscopeParams.segments` to build the filename, and uses
`HtmlCanvasElement::to_blob_with_type` → `JsFuture` → programmatic `<a>.click()`
to trigger the browser save dialog.  Continuous export or server-side storage is
explicitly out of scope.

## Design

### Data flow

1. User clicks **↓ EXPORT** button → `dropdown_open.update(|o| *o = !*o)`.
   Button is `disabled` when `AppState.image_loaded == false`.
2. Dropdown renders with three radio options (PNG / JPEG / WebP) and a
   **↓ DOWNLOAD** button.  `selected_format: RwSignal<&'static str>` holds the
   chosen MIME type (default `"image/png"`).
3. User clicks **↓ DOWNLOAD** → reads `selected_format` and `params.segments`
   untracked, builds a filename, closes the dropdown, then calls
   `spawn_local(trigger_download(mime, filename))`.
4. `trigger_download` (async):
   - Gets canvas by `document.get_element_by_id("kaleidoscope-canvas")`.
   - Calls `canvas.to_blob_with_type(mime)` → awaits `JsFuture`.
   - Creates object URL → creates an `<a>` element, sets `href` + `download`,
     appends to `<body>`, calls `.click()`, removes with `.remove()`, revokes URL.
5. Errors from any step are logged to `console.error` — no UI error state needed
   in v1 (covered by the milestone notes).

### Function / type signatures

```rust
/// Returns the file extension string for a MIME type.
/// Falls back to `"png"` for any unrecognised type.
pub fn mime_to_ext(mime: &str) -> &'static str;

/// Builds the download filename from current segments and today's JS Date.
/// Format: `teleidoscope-{segments}m-{YYYYMMDD}.{ext}`
pub(crate) fn build_filename(segments: u32, mime: &str) -> String;

/// Performs the async download sequence (blob → URL → anchor click → revoke).
/// Logs to console.error on failure; never panics.
async fn trigger_download(mime: String, filename: String);

/// Export format picker and canvas download trigger (Leptos component).
#[component]
pub fn ExportMenu() -> impl IntoView;
```

### Edge cases

- **Image not loaded:** button is `disabled`; click handler cannot fire.
- **Unknown MIME fallback:** `mime_to_ext` returns `"png"` for anything unexpected.
- **Canvas not in DOM:** `trigger_download` logs an error and returns early.
- **`to_blob_with_type` fails (e.g. security error):** `JsFuture` returns `Err`;
  caught and logged.
- **Rapid double-click on DOWNLOAD:** dropdown closes on first click; second click
  has no dropdown to click.

### Integration points

- `src/components/export_menu.rs` — new implementation (was a stub)
- `src/components/controls_panel.rs` — add `<ExportMenu/>` at bottom
- `Cargo.toml` — add `"HtmlAnchorElement"`, `"Node"`, `"HtmlBodyElement"` to
  `web-sys` features
- `tests/m8_export.rs` — new browser integration test file

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `to_blob_with_type` might time out if canvas is very large or GPU is busy | Acceptable for v1; no timeout needed per milestone notes |
| Simplicity | Appending `<a>` to body adds DOM mutation complexity | Required for Firefox cross-browser compat; wrapped in a single helper fn |
| Coupling | ExportMenu reads canvas by string ID — brittle if ID changes | ID `"kaleidoscope-canvas"` is stable (set in CanvasView); acceptable |
| Performance | Each download creates a new Blob + URL; no caching | Downloads are infrequent; no caching needed |
| Testability | `trigger_download` is hard to unit-test (requires real canvas with GPU) | Test disabled state and dropdown wiring via DOM tests; download flow covered by manual checklist |

## Implementation Notes

- `js_sys::Date::get_month()` is 0-indexed; add 1 before formatting.
- `js_sys::Date::get_full_year()` returns `u32`.
- After `.click()`, call `a.remove()` (from `Element`) rather than `parent.remove_child(&a)`.
- `web_sys::Url::revoke_object_url` should be called after the click even though
  the download is async — the browser has already queued the download by then.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `mime_to_ext` returns correct ext for png/jpeg/webp | 2 | ✅ | wasm_bindgen_test in module |
| `mime_to_ext` fallback for unknown mime | 2 | ✅ | wasm_bindgen_test in module |
| Download button disabled when `image_loaded = false` | 3 | ✅ | tests/m8_export.rs |
| Download button enabled when `image_loaded = true` | 3 | ✅ | tests/m8_export.rs |
| Clicking EXPORT toggles dropdown | 3 | ✅ | tests/m8_export.rs |
| Format radio selection updates signal | 3 | ✅ | tests/m8_export.rs |
| `trigger_download` async blob flow | 2 | ❌ waived | requires real GPU canvas; covered by manual checklist MT-1 to MT-4 |
| Filename format `teleidoscope-Nm-YYYYMMDD.ext` | 2 | ✅ | wasm_bindgen_test in module |

## Test Results

_Filled after Phase 6._

## Review Notes

_Filled after Phase 7._

## Callouts / Gotchas

_Filled after Phase 10._
