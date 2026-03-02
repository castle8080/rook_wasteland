use js_sys::Uint8Array;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, Url};

use crate::poem_repository::{fetch_index, fetch_poem};
use crate::recording_store::{get_audio_blob, get_recording, RecordingMetadata};
use crate::ui::audio_player::AudioPlayer;

// ---------------------------------------------------------------------------
// Helpers (same pattern as recordings_list)
// ---------------------------------------------------------------------------

fn bytes_to_blob(data: Vec<u8>, mime: &str) -> Blob {
    let uint8 = Uint8Array::from(data.as_slice());
    let parts = js_sys::Array::new();
    parts.push(&uint8.buffer());
    let props = BlobPropertyBag::new();
    props.set_type(mime);
    Blob::new_with_buffer_source_sequence_and_options(&parts, &props).unwrap()
}

fn download_filename(metadata: &RecordingMetadata) -> String {
    let title_slug: String = metadata
        .poem_title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    let date_part = metadata
        .recorded_at
        .get(..19)
        .unwrap_or("1970-01-01T00-00-00")
        .replace('T', "_")
        .replace(':', "-");

    let ext = if metadata.mime_type.contains("ogg") {
        "ogg"
    } else if metadata.mime_type.contains("mp4") || metadata.mime_type.contains("aac") {
        "m4a"
    } else {
        "webm"
    };

    format!("{title_slug}_{date_part}.{ext}")
}

fn format_date(iso: &str) -> String {
    let date = js_sys::Date::new(&js_sys::JsString::from(iso));
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let idx = date.get_month() as usize;
    format!(
        "{} {}, {}",
        months.get(idx).copied().unwrap_or("?"),
        date.get_date(),
        date.get_full_year()
    )
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Recording detail view at /readings/:recording_id.
#[component]
pub fn RecordingDetailView() -> impl IntoView {
    let params = use_params_map();
    let recording_id =
        move || params.read().get("recording_id").unwrap_or_default().to_string();

    type DetailResult = Result<(RecordingMetadata, Option<String>, String), String>;

    // Fetch metadata + poem content in one resource
    let detail_resource: LocalResource<DetailResult> = LocalResource::new(move || {
        let rid = recording_id();
        async move {
            let metadata = get_recording(&rid)
                .await
                .map_err(|e| e.to_string())?;

            // Try fetching poem content via index lookup
            let poem_content = 'poem: {
                let index = match fetch_index().await {
                    Err(_) => break 'poem None,
                    Ok(idx) => idx,
                };
                let entry = match index.poems.iter().find(|e| e.id == metadata.poem_id) {
                    None => break 'poem None,
                    Some(e) => e.clone(),
                };
                fetch_poem(&entry.path).await.ok().map(|p| p.content)
            };

            let audio_url = build_audio_url(&metadata).await?;
            Ok((metadata, poem_content, audio_url))
        }
    });

    view! {
        <main class="content-column">
            {move || match detail_resource.get() {
                None => view! {
                    <p class="state-message">"Loading…"</p>
                }.into_any(),

                Some(Err(e)) => view! {
                    <div class="state-message">
                        <p>{e.to_string()}</p>
                        <a href="/readings">"← All readings"</a>
                    </div>
                }.into_any(),

                Some(Ok((metadata, poem_content, audio_url))) => {
                    let poem_id_for_link = metadata.poem_id.clone();
                    let m_dl = metadata.clone();

                    view! {
                        <nav class="recording-detail__nav">
                            <a href="/readings">"← All readings"</a>
                            <a href=format!("/?poem_id={}", poem_id_for_link)>
                                "Read this poem →"
                            </a>
                        </nav>

                        <article>
                            <h1 class="poem-title">{metadata.poem_title.clone()}</h1>
                            <p class="poem-meta">{metadata.poem_author.clone()}</p>
                            {match poem_content {
                                Some(text) => view! {
                                    <pre class="poem-body">{text}</pre>
                                }.into_any(),
                                None => view! {
                                    <p class="text-secondary">
                                        "(Full poem text is not available.)"
                                    </p>
                                }.into_any(),
                            }}
                        </article>

                        <section class="recording-detail__player">
                            <AudioPlayer src=audio_url />
                        </section>

                        <section class="recording-detail__meta">
                            <p class="text-secondary">
                                "Recorded " {format_date(&metadata.recorded_at)}
                                {metadata.duration_ms.map(|ms| {
                                    let secs = ms as f64 / 1000.0;
                                    let m = (secs as u64) / 60;
                                    let s = (secs as u64) % 60;
                                    format!(" · {}:{:02}", m, s)
                                })}
                            </p>
                        </section>

                        <button
                            class="btn btn-secondary"
                            on:click=move |_| {
                                let m = m_dl.clone();
                                let bk = m.audio_blob_key.clone();
                                let mt = m.mime_type.clone();
                                let fn_ = download_filename(&m);
                                spawn_local(async move {
                                    if let Ok(data) = get_audio_blob(&bk).await {
                                        let blob = bytes_to_blob(data, &mt);
                                        if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                            let doc = web_sys::window()
                                                .and_then(|w| w.document())
                                                .expect("document");
                                            let a: web_sys::HtmlAnchorElement = doc
                                                .create_element("a")
                                                .unwrap()
                                                .unchecked_into();
                                            a.set_href(&url);
                                            a.set_download(&fn_);
                                            a.click();
                                            let _ = Url::revoke_object_url(&url);
                                        }
                                    }
                                });
                            }
                        >
                            "↓ Download"
                        </button>
                    }.into_any()
                }
            }}
        </main>
    }
}

async fn build_audio_url(metadata: &RecordingMetadata) -> Result<String, String> {
    let data = get_audio_blob(&metadata.audio_blob_key)
        .await
        .map_err(|e| e.to_string())?;
    let blob = bytes_to_blob(data, &metadata.mime_type);
    Url::create_object_url_with_blob(&blob).map_err(|_| "Failed to create object URL".to_string())
}
