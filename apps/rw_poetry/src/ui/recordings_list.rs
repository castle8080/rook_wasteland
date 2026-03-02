use js_sys::Uint8Array;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, Url};

use crate::recording_store::{RecordingMetadata, get_audio_blob, list_recordings};
use crate::ui::audio_player::{AudioPlayer, format_duration};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert raw bytes + mime type into a `web_sys::Blob`.
fn bytes_to_blob(data: Vec<u8>, mime: &str) -> Blob {
    let uint8 = Uint8Array::from(data.as_slice());
    let parts = js_sys::Array::new();
    parts.push(&uint8.buffer());
    let props = BlobPropertyBag::new();
    props.set_type(mime);
    Blob::new_with_buffer_source_sequence_and_options(&parts, &props).unwrap()
}

/// Format an ISO 8601 date-time string as "Mar 1, 2026".
fn format_date(iso: &str) -> String {
    let date = js_sys::Date::new(&js_sys::JsString::from(iso));
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month_idx = date.get_month() as usize;
    let day = date.get_date();
    let year = date.get_full_year();
    format!(
        "{} {}, {}",
        months.get(month_idx).copied().unwrap_or("?"),
        day,
        year
    )
}

/// Build a download filename per spec section 6.7.
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

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Recordings library view at /readings.
#[component]
pub fn RecordingsListView() -> impl IntoView {
    // (recording_id, object_url) of the currently-playing row
    let active_src: RwSignal<Option<(String, String)>> = RwSignal::new(None);
    // recording_id of any row that failed to load its audio blob
    let play_error_id: RwSignal<Option<String>> = RwSignal::new(None);

    let recordings_resource: LocalResource<Result<Vec<RecordingMetadata>, String>> =
        LocalResource::new(move || async move {
            let mut list = list_recordings().await.map_err(|e| e.to_string())?;
            list.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));
            Ok(list)
        });

    let on_play = move |recording_id: String, blob_key: String, mime_type: String| {
        spawn_local(async move {
            // Toggle off if same row
            if let Some((cur_id, cur_url)) = active_src.get_untracked() {
                let _ = Url::revoke_object_url(&cur_url);
                if cur_id == recording_id {
                    active_src.set(None);
                    return;
                }
            }
            play_error_id.set(None);

            match get_audio_blob(&blob_key).await {
                Err(_) => play_error_id.set(Some(recording_id)),
                Ok(data) => {
                    let blob = bytes_to_blob(data, &mime_type);
                    match Url::create_object_url_with_blob(&blob) {
                        Err(_) => play_error_id.set(Some(recording_id)),
                        Ok(url) => active_src.set(Some((recording_id, url))),
                    }
                }
            }
        });
    };

    let on_download = move |metadata: RecordingMetadata| {
        let blob_key = metadata.audio_blob_key.clone();
        let mime_type = metadata.mime_type.clone();
        let filename = download_filename(&metadata);
        spawn_local(async move {
            if let Ok(data) = get_audio_blob(&blob_key).await {
                let blob = bytes_to_blob(data, &mime_type);
                if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                    let doc = web_sys::window()
                        .and_then(|w| w.document())
                        .expect("document");
                    let a: web_sys::HtmlAnchorElement =
                        doc.create_element("a").unwrap().unchecked_into();
                    a.set_href(&url);
                    a.set_download(&filename);
                    a.click();
                    let _ = Url::revoke_object_url(&url);
                }
            }
        });
    };

    view! {
        <main class="content-column">
            <h1 class="recordings-list__heading">"Readings"</h1>
            {move || {
                match recordings_resource.get() {
                    None => view! {
                        <p class="state-message">"Loading readings…"</p>
                    }.into_any(),

                    Some(Err(e)) => view! {
                        <p class="state-message">{format!("Unable to load recordings. ({e})")}</p>
                    }.into_any(),

                    Some(Ok(list)) if list.is_empty() => view! {
                        <p class="state-message text-secondary">
                            "No readings yet — record your first poem on the reader page."
                        </p>
                    }.into_any(),

                    Some(Ok(list)) => {
                        let active_id = move || active_src.get().map(|(id, _)| id);
                        view! {
                            <ul class="recordings-list" role="list">
                                {list.into_iter().map(|m| {
                                    let rid = m.recording_id.clone();
                                    let bkey = m.audio_blob_key.clone();
                                    let mime = m.mime_type.clone();
                                    let rid2 = rid.clone();
                                    let rid2b = rid.clone();
                                    let rid_err = rid.clone();
                                    let is_active = move || active_id() == Some(rid2.clone());
                                    let is_active2 = move || active_id() == Some(rid2b.clone());
                                    let has_play_error = move || play_error_id.get() == Some(rid_err.clone());
                                    let active_url = {
                                        let rid3 = rid.clone();
                                        move || active_src.get()
                                            .filter(|(id, _)| id == &rid3)
                                            .map(|(_, url)| url)
                                    };
                                    let duration_secs = m.duration_ms
                                        .map(|ms| ms as f64 / 1000.0)
                                        .unwrap_or(0.0);
                                    let date_str = format_date(&m.recorded_at);
                                    let dur_str = format_duration(duration_secs);
                                    let rid_link = m.recording_id.clone();
                                    let rid_play = m.recording_id.clone();
                                    let m_dl = m.clone();

                                    view! {
                                        <li class="recordings-list__item">
                                            <div class="recordings-list__meta">
                                                <a
                                                    class="recordings-list__title"
                                                    href=format!("#/readings/{}", rid_link)
                                                >
                                                    {m.poem_title.clone()}
                                                </a>
                                                <span class="recordings-list__secondary text-secondary">
                                                    {m.poem_author.clone()}
                                                    " · " {date_str} " · " {dur_str}
                                                </span>
                                            </div>
                                            <div class="recordings-list__actions">
                                                <button
                                                    class="btn btn-icon"
                                                    aria-label=move || if is_active() { "Stop" } else { "Play" }
                                                    on:click={
                                                        let rp = rid_play.clone();
                                                        let bk = bkey.clone();
                                                        let mt = mime.clone();
                                                        move |_| on_play(rp.clone(), bk.clone(), mt.clone())
                                                    }
                                                >
                                                    {move || if is_active2() { "⏹" } else { "▶" }}
                                                </button>
                                                <button
                                                    class="btn btn-icon"
                                                    aria-label="Download"
                                                    on:click={
                                                        let md = m_dl.clone();
                                                        move |_| on_download(md.clone())
                                                    }
                                                >
                                                    "↓"
                                                </button>
                                            </div>
                                            {move || active_url().map(|url| view! {
                                                <div class="recordings-list__player">
                                                    <AudioPlayer src=url />
                                                </div>
                                            })}
                                            {move || has_play_error().then(|| view! {
                                                <p class="text-secondary" style="font-size:0.85rem;">
                                                    "Recording data unavailable."
                                                </p>
                                            })}
                                        </li>
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }
                }
            }}
        </main>
    }
}
