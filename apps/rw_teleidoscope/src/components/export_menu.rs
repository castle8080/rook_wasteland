use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};

use crate::state::{AppState, KaleidoscopeParams};

/// Returns the file extension string for a given MIME type.
///
/// Falls back to `"png"` for any unrecognised type.
pub fn mime_to_ext(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpeg",
        "image/webp" => "webp",
        _ => "png",
    }
}

/// Builds the download filename from the current mirror count and a local
/// datetime timestamp (to-the-second precision).
///
/// Format: `teleidoscope-{segments}m-{YYYYMMDD}-{HHmmss}.{ext}`  
/// Example: `teleidoscope-6m-20260306-173045.png`
///
/// Second-precision timestamps prevent filename collisions when the user
/// exports multiple times in the same session.  Local time is used so the
/// timestamp matches the clock the user sees.
///
/// Uses `js_sys::Date` rather than `std::time` because `std::time::SystemTime`
/// is not available in WASM without the `wasm-pack` time feature.
pub(crate) fn build_filename(segments: u32, mime: &str) -> String {
    let date = js_sys::Date::new_0();
    let year  = date.get_full_year();
    let month = date.get_month() + 1; // getMonth() is 0-indexed (Jan = 0)
    let day   = date.get_date();
    let hour  = date.get_hours();
    let min   = date.get_minutes();
    let sec   = date.get_seconds();
    let ext = mime_to_ext(mime);
    format!("teleidoscope-{segments}m-{year:04}{month:02}{day:02}-{hour:02}{min:02}{sec:02}.{ext}")
}

/// Wraps the callback-based `HtmlCanvasElement.toBlob(type)` in a
/// `js_sys::Promise` so it can be awaited with `JsFuture`.
///
/// `canvas.toBlob()` in the Web API takes a `BlobCallback` rather than
/// returning a Promise directly; this helper bridges the two.
fn canvas_to_blob_promise(
    canvas: &web_sys::HtmlCanvasElement,
    mime: &str,
) -> js_sys::Promise {
    let canvas = canvas.clone();
    let mime = mime.to_string();

    js_sys::Promise::new(&mut |resolve, reject| {
        // Build a one-shot JS closure that forwards the Blob to `resolve`.
        let cb = Closure::once(move |blob: wasm_bindgen::JsValue| {
            let _ = resolve.call1(&wasm_bindgen::JsValue::NULL, &blob);
        });

        match canvas.to_blob_with_type(
            cb.as_ref().unchecked_ref::<js_sys::Function>(),
            &mime,
        ) {
            // `to_blob_with_type` schedules the callback asynchronously;
            // forget the Closure so it lives until the callback fires.
            Ok(()) => cb.forget(),
            // If the call itself fails synchronously, reject the Promise.
            Err(e) => {
                let _ = reject.call1(&wasm_bindgen::JsValue::NULL, &e);
            }
        }
    })
}

/// Performs the async canvas-to-file download sequence.
///
/// 1. Locates the canvas element by its stable ID `"kaleidoscope-canvas"`.
/// 2. Wraps `canvas.toBlob(type)` in a Promise and awaits the result.
/// 3. Creates an object URL, appends a temporary `<a>` element to `<body>`,
///    triggers a click to open the save dialog, removes the anchor, then
///    revokes the URL to free memory.
///
/// Errors at any step are logged to `console.error`; this function never panics.
async fn trigger_download(mime: String, filename: String) {
    let result: Result<(), wasm_bindgen::JsValue> = async {
        let window = web_sys::window()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))?;
        let document = window
            .document()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("no document"))?;

        let canvas = document
            .get_element_by_id("kaleidoscope-canvas")
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("canvas element not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;

        let blob_val = JsFuture::from(canvas_to_blob_promise(&canvas, &mime)).await?;
        let blob = blob_val.dyn_into::<web_sys::Blob>()?;

        let url = web_sys::Url::create_object_url_with_blob(&blob)?;

        let a = document
            .create_element("a")?
            .dyn_into::<web_sys::HtmlAnchorElement>()?;
        a.set_href(&url);
        a.set_download(&filename);

        // Appending to <body> before clicking is required in Firefox for
        // programmatic anchor downloads to work reliably.
        let body = document
            .body()
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("no body"))?;
        body.append_child(&a)?;
        a.click();
        a.remove();

        // The browser has already queued the download; revoke the URL to free
        // the underlying Blob memory.
        web_sys::Url::revoke_object_url(&url)?;

        Ok(())
    }
    .await;

    if let Err(e) = result {
        web_sys::console::error_1(&e);
    }
}

/// Download icon (Bootstrap Icons, MIT licence).
const DOWNLOAD: &str =
    "M.5 9.9a.5.5 0 0 1 .5.5v2.5a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-2.5a.5.5 0 0 1 1 0v2.5a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2v-2.5a.5.5 0 0 1 .5-.5zM7.646 11.854a.5.5 0 0 0 .708 0l3-3a.5.5 0 0 0-.708-.708L8.5 10.293V1.5a.5.5 0 0 0-1 0v8.793L5.354 8.146a.5.5 0 1 0-.708.708l3 3z";

/// Export format picker and canvas download trigger.
///
/// Renders a **↓ EXPORT** toggle button at the bottom of the controls panel.
/// When clicked (and an image is loaded), a small dropdown reveals three format
/// radio buttons (PNG / JPEG / WebP) and a **↓ DOWNLOAD** button.
///
/// The EXPORT button is `disabled` when `AppState.image_loaded` is `false`.
/// The currently selected format is kept in a local `RwSignal<&'static str>` and
/// defaults to `"image/png"`.
#[component]
pub fn ExportMenu() -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let params = expect_context::<KaleidoscopeParams>();

    let dropdown_open: RwSignal<bool> = RwSignal::new(false);
    let selected_format: RwSignal<&'static str> = RwSignal::new("image/png");

    let on_toggle = move |_| {
        dropdown_open.update(|o| *o = !*o);
    };

    let on_download = move |_| {
        let mime = selected_format.get_untracked();
        let segments = params.segments.get_untracked();
        let filename = build_filename(segments, mime);
        dropdown_open.set(false);
        spawn_local(trigger_download(mime.to_string(), filename));
    };

    view! {
        <div class="export-menu">
            <button
                class="export-button"
                disabled=move || !app_state.image_loaded.get()
                on:click=on_toggle
            >
                <svg viewBox="0 0 16 16" fill="currentColor" width="14" height="14" class="btn-icon">
                    <path d=DOWNLOAD/>
                </svg>
                " EXPORT"
            </button>
            <Show when=move || dropdown_open.get()>
                <div class="export-dropdown">
                    <label class="export-option">
                        <input
                            type="radio"
                            name="export-format"
                            value="image/png"
                            prop:checked=move || selected_format.get() == "image/png"
                            on:change=move |_| selected_format.set("image/png")
                        />
                        " PNG"
                    </label>
                    <label class="export-option">
                        <input
                            type="radio"
                            name="export-format"
                            value="image/jpeg"
                            prop:checked=move || selected_format.get() == "image/jpeg"
                            on:change=move |_| selected_format.set("image/jpeg")
                        />
                        " JPEG"
                    </label>
                    <label class="export-option">
                        <input
                            type="radio"
                            name="export-format"
                            value="image/webp"
                            prop:checked=move || selected_format.get() == "image/webp"
                            on:change=move |_| selected_format.set("image/webp")
                        />
                        " WebP"
                    </label>
                    <hr class="export-divider"/>
                    <button class="download-button" on:click=on_download>
                        <svg viewBox="0 0 16 16" fill="currentColor" width="14" height="14" class="btn-icon">
                            <path d=DOWNLOAD/>
                        </svg>
                        " DOWNLOAD"
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn mime_to_ext_png() {
        assert_eq!(mime_to_ext("image/png"), "png");
    }

    #[wasm_bindgen_test]
    fn mime_to_ext_jpeg() {
        assert_eq!(mime_to_ext("image/jpeg"), "jpeg");
    }

    #[wasm_bindgen_test]
    fn mime_to_ext_webp() {
        assert_eq!(mime_to_ext("image/webp"), "webp");
    }

    #[wasm_bindgen_test]
    fn mime_to_ext_fallback() {
        assert_eq!(mime_to_ext("unknown/format"), "png");
    }

    #[wasm_bindgen_test]
    fn build_filename_format_png() {
        let name = build_filename(6, "image/png");
        assert!(
            name.starts_with("teleidoscope-6m-"),
            "filename should start with teleidoscope-6m-; got {name}"
        );
        assert!(name.ends_with(".png"), "filename should end with .png; got {name}");
        // "teleidoscope-6m-YYYYMMDD-HHmmss.png" = 36 chars
        assert_eq!(
            name.len(),
            "teleidoscope-6m-YYYYMMDD-HHmmss.png".len(),
            "filename length mismatch: {name}"
        );
    }

    #[wasm_bindgen_test]
    fn build_filename_format_jpeg() {
        let name = build_filename(12, "image/jpeg");
        assert!(name.starts_with("teleidoscope-12m-"), "got {name}");
        assert!(name.ends_with(".jpeg"), "got {name}");
        // "teleidoscope-12m-YYYYMMDD-HHmmss.jpeg" = 38 chars
        assert_eq!(
            name.len(),
            "teleidoscope-12m-YYYYMMDD-HHmmss.jpeg".len(),
            "filename length mismatch: {name}"
        );
    }

    #[wasm_bindgen_test]
    fn build_filename_format_webp() {
        let name = build_filename(3, "image/webp");
        assert!(name.starts_with("teleidoscope-3m-"), "got {name}");
        assert!(name.ends_with(".webp"), "got {name}");
    }

    #[wasm_bindgen_test]
    fn build_filename_contains_timestamp_segment() {
        // The HHmmss part should be a 6-digit block between the date and extension.
        // Format: teleidoscope-{n}m-YYYYMMDD-HHmmss.ext
        let name = build_filename(6, "image/png");
        // Split on '-' to extract components: ["teleidoscope", "6m", "YYYYMMDD", "HHmmss.png"]
        let parts: Vec<&str> = name.splitn(4, '-').collect();
        assert_eq!(parts.len(), 4, "expected 4 dash-separated parts; got {name}");
        let date_part = parts[2];
        let time_ext  = parts[3]; // "HHmmss.png"
        assert_eq!(date_part.len(), 8, "date part should be 8 chars (YYYYMMDD); got {date_part}");
        let time_part = time_ext.split('.').next().unwrap_or("");
        assert_eq!(time_part.len(), 6, "time part should be 6 chars (HHmmss); got {time_part}");
        assert!(date_part.chars().all(|c| c.is_ascii_digit()), "date part not all digits: {date_part}");
        assert!(time_part.chars().all(|c| c.is_ascii_digit()), "time part not all digits: {time_part}");
    }
}
