use std::cell::RefCell;

use leptos::prelude::*;
use leptos::task::spawn_local;
use uuid::Uuid;

use crate::audio_capture::{pick_mime_type, request_mic, AudioRecorder};
use crate::recording_store::{save_recording, RecordingMetadata};

// ---------------------------------------------------------------------------
// Thread-local recorder handle (WASM is single-threaded)
// ---------------------------------------------------------------------------

thread_local! {
    static RECORDER: RefCell<Option<AudioRecorder>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum RecordingState {
    Idle,
    Recording { elapsed_secs: u32 },
    Saved,
    Error(String),
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Recording controls: Record / Stop / Saved states wired to audio_capture + recording_store.
#[component]
pub fn RecordingControls(
    /// The current poem's ID
    poem_id: String,
    /// Snapshotted poem title for metadata
    poem_title: String,
    /// Snapshotted poem author for metadata
    poem_author: String,
) -> impl IntoView {
    let state: RwSignal<RecordingState> = RwSignal::new(RecordingState::Idle);

    let poem_id = StoredValue::new_local(poem_id);
    let poem_title = StoredValue::new_local(poem_title);
    let poem_author = StoredValue::new_local(poem_author);

    let on_record = move |_| {
        spawn_local(async move {
            match request_mic().await {
                Err(e) => {
                    state.set(RecordingState::Error(e.to_string()));
                }
                Ok(stream) => {
                    let mime = pick_mime_type();
                    match AudioRecorder::start(&stream, mime) {
                        Err(e) => state.set(RecordingState::Error(e.to_string())),
                        Ok(recorder) => {
                            RECORDER.with(|r| *r.borrow_mut() = Some(recorder));
                            state.set(RecordingState::Recording { elapsed_secs: 0 });

                            // Tick elapsed timer every second
                            spawn_local(async move {
                                loop {
                                    gloo_timers::future::TimeoutFuture::new(1000).await;
                                    match state.get_untracked() {
                                        RecordingState::Recording { elapsed_secs } => {
                                            state.set(RecordingState::Recording {
                                                elapsed_secs: elapsed_secs + 1,
                                            });
                                        }
                                        _ => break,
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });
    };

    let on_stop = move |_| {
        let recorder = RECORDER.with(|r| r.borrow_mut().take());
        if let Some(recorder) = recorder {
            let pid = poem_id.get_value();
            let ptitle = poem_title.get_value();
            let pauthor = poem_author.get_value();
            spawn_local(async move {
                match recorder.stop().await {
                    Err(e) => state.set(RecordingState::Error(e.to_string())),
                    Ok(blob) => {
                        let recording_id = Uuid::new_v4().to_string();
                        let blob_key = Uuid::new_v4().to_string();
                        let metadata = RecordingMetadata {
                            recording_id,
                            poem_id: pid,
                            poem_title: ptitle,
                            poem_author: pauthor,
                            recorded_at: js_sys::Date::new_0()
                                .to_iso_string()
                                .as_string()
                                .unwrap_or_default(),
                            duration_ms: blob.duration_ms,
                            mime_type: blob.mime_type,
                            audio_blob_key: blob_key,
                        };
                        match save_recording(metadata, blob.data).await {
                            Ok(()) => {
                                state.set(RecordingState::Saved);
                                spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(2000).await;
                                    state.set(RecordingState::Idle);
                                });
                            }
                            Err(e) => {
                                state.set(RecordingState::Error(e.to_string()));
                            }
                        }
                    }
                }
            });
        }
    };

    view! {
        <div class="recording-controls">
            {move || {
                match state.get() {
                    RecordingState::Idle => view! {
                        <button class="btn btn-record" on:click=on_record>"⏺ Record"</button>
                    }.into_any(),

                    RecordingState::Recording { elapsed_secs } => view! {
                        <button class="btn btn-stop" on:click=on_stop>"⏹ Stop"</button>
                        <span class="recording-controls__timer" aria-live="polite">
                            {format!("{}:{:02}", elapsed_secs / 60, elapsed_secs % 60)}
                        </span>
                        <span class="recording-active-indicator" aria-hidden="true"></span>
                    }.into_any(),

                    RecordingState::Saved => view! {
                        <span class="recording-controls__saved">"✓ Saved"</span>
                    }.into_any(),

                    RecordingState::Error(msg) => view! {
                        <div>
                            <p class="text-secondary">{msg.clone()}</p>
                            <button class="btn btn-secondary"
                                on:click=move |_| state.set(RecordingState::Idle)>
                                "Dismiss"
                            </button>
                        </div>
                    }.into_any(),
                }
            }}
        </div>
    }
}
