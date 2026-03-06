use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::state::AppState;
use crate::{renderer, utils};

/// Feed a `File` through the full decode → resize → texture-upload pipeline.
///
/// The pipeline is asynchronous (two nested callbacks):
/// 1. `FileReader.readAsDataURL(file)` → `ProgressEvent "load"` →
/// 2. `HtmlImageElement.src = dataURL` → `Event "load"` →
/// 3. `utils::resize_to_800(img)` → `renderer::upload_image` →
/// 4. `AppState.image_loaded.set(true)`
///
/// Files with an unsupported MIME type emit a `console.warn` and are ignored.
///
/// This function is `pub` so that `canvas_view.rs` can reuse it for the
/// drag-and-drop path without duplicating the pipeline.
pub fn load_file(file: web_sys::File, app_state: AppState) {
    let mime = file.type_();
    if !utils::is_accepted_image_type(&mime) {
        web_sys::console::warn_1(
            &format!("Unsupported file type: \"{mime}\". Use PNG, JPEG, or WebP.").into(),
        );
        return;
    }

    let reader = web_sys::FileReader::new().expect("FileReader::new");
    let img = web_sys::HtmlImageElement::new().expect("HtmlImageElement::new");

    // Step 2 — image onload: resize and upload to GPU.
    let img_for_load = img.clone();
    let onload_img = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| {
        match utils::resize_to_800(&img_for_load) {
            Ok(image_data) => {
                renderer::with_renderer_mut(|r| r.upload_image(&image_data));
                app_state.image_loaded.set(true);
            }
            Err(e) => {
                web_sys::console::error_1(&e.into());
            }
        }
    });
    img.set_onload(Some(onload_img.as_ref().unchecked_ref()));
    onload_img.forget();

    // Step 1 — reader onload: set image.src to the data URL.
    let reader_for_onload = reader.clone();
    let img_for_reader = img.clone();
    let onload_reader =
        Closure::<dyn FnMut(web_sys::ProgressEvent)>::new(move |_: web_sys::ProgressEvent| {
            let result = reader_for_onload
                .result()
                .expect("FileReader result")
                .as_string()
                .expect("result as string");
            img_for_reader.set_src(&result);
        });
    reader.set_onload(Some(onload_reader.as_ref().unchecked_ref()));
    onload_reader.forget();

    reader.read_as_data_url(&file).expect("read_as_data_url");
}

/// App title bar with the "Load Image" and "Use Camera" buttons.
///
/// Clicking "Load Image" opens a hidden `<input type="file">` restricted to
/// PNG, JPEG, and WebP.  "Use Camera" is a stub until M7.
#[component]
pub fn Header() -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let file_input_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_file_change = move |ev: web_sys::Event| {
        // In Leptos 0.8, on:change fires with a web_sys::Event (not InputEvent).
        let input: web_sys::HtmlInputElement =
            ev.target().expect("event target").unchecked_into();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                load_file(file, app_state);
            }
        }
        // Reset value so the same file can be re-selected in the same session.
        input.set_value("");
    };

    view! {
        <header id="app-header">
            <span class="app-title">"⚙ TELEIDOSCOPE"</span>
            <div class="header-actions">
                <button
                    class="header-btn"
                    on:click=move |_| {
                        if let Some(input) = file_input_ref.get() {
                            input.click();
                        }
                    }
                >
                    "⚙ LOAD IMAGE"
                </button>
                <button
                    class="header-btn"
                    on:click=move |_| app_state.camera_open.set(true)
                >
                    "📷 USE CAMERA"
                </button>
            </div>
            // Hidden file input; triggered programmatically by the button above.
            <input
                node_ref=file_input_ref
                type="file"
                accept="image/png,image/jpeg,image/webp"
                style="display:none"
                on:change=on_file_change
            />
        </header>
    }
}

