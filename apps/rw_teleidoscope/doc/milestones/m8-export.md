# M8 — Export / Download

**Status:** ⬜ Pending  
**Depends on:** [M4 — Mirror Symmetry Core](m4-mirror-symmetry.md)  
**Unlocks:** [M10 — Steampunk Polish](m10-steampunk-polish.md) (after M9)

---

## Goal

A user can download the current canvas view as an image file. Format (PNG, JPEG,
or WebP) is selectable from a dropdown. The filename includes the mirror count
and a timestamp. The download must work reliably across Firefox and Chrome.

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Create `src/components/export_menu.rs` — a "Download ▾" button that toggles a small dropdown showing PNG / JPEG / WebP options | ⬜ |
| 2 | Implement download trigger for PNG — call `canvas.to_blob_with_type("image/png")`, bridge Promise with `JsFuture`, create object URL with `Url::create_object_url_with_blob`, create an `<a>` element, set `href` and `download` attribute, `.click()`, revoke URL | ⬜ |
| 3 | Implement download for JPEG — same flow with `"image/jpeg"` | ⬜ |
| 4 | Implement download for WebP — same flow with `"image/webp"` | ⬜ |
| 5 | Build filename from current params: `teleidoscope-{segments}m-{YYYYMMDD}.{ext}` (e.g. `teleidoscope-6m-20260306.png`) | ⬜ |
| 6 | Wire `ExportMenu` into the controls panel (below the Randomize button per wireframe) | ⬜ |
| 7 | Disable the Download button when `AppState.image_loaded` is false | ⬜ |
| 8 | Verify `python make.py build` and `python make.py lint` pass | ⬜ |

---

## Manual Test Checklist

- [ ] Load an image → see kaleidoscope → click Download ▾ → dropdown shows PNG / JPEG / WebP
- [ ] Select PNG → file downloads with `.png` extension; opens correctly in image viewer
- [ ] Select JPEG → file downloads with `.jpeg` extension; opens correctly
- [ ] Select WebP → file downloads with `.webp` extension; opens correctly
- [ ] Filename contains the current mirror segment count and today's date
- [ ] Download button is greyed out / disabled before any image is loaded
- [ ] Download works in both Firefox and Chrome
- [ ] No console errors during download

---

## Notes

- `HtmlCanvasElement::to_blob_with_type` returns a `Promise<Blob>` — use
  `JsFuture` to bridge it. This must run inside `spawn_local`.
- After `Url::create_object_url_with_blob`, always call `Url::revoke_object_url`
  after the click to avoid memory leaks.
- The programmatic `<a>.click()` approach is the most cross-browser-compatible
  way to trigger a download without a server.
- WebP is not supported in Safari (as of writing); the format selector can still
  offer it but it may fall back to PNG in those browsers — no special handling required in v1.
- Date for filename: get from `js_sys::Date::new_0()` to avoid any native
  `std::time` (which is not available in WASM without a feature flag).
