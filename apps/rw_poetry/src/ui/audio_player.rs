use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Url;

// ---------------------------------------------------------------------------
// Formatting helper
// ---------------------------------------------------------------------------

/// Format seconds as `M:SS` (no hour formatting needed for poems).
pub fn format_duration(secs: f64) -> String {
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

#[cfg(test)]
mod tests {
    use super::format_duration;

    #[test]
    fn format_zero() {
        assert_eq!(format_duration(0.0), "0:00");
    }

    #[test]
    fn format_91_5_seconds() {
        assert_eq!(format_duration(91.5), "1:31");
    }

    #[test]
    fn format_over_one_hour() {
        assert_eq!(format_duration(3661.0), "61:01");
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Custom audio player backed by a hidden `<audio>` element.
/// `src` must be an object URL or asset URL pointing to the audio data.
#[component]
pub fn AudioPlayer(
    /// Object URL or audio asset URL
    src: String,
) -> impl IntoView {
    let audio_ref: NodeRef<html::Audio> = NodeRef::new();
    let playing = RwSignal::new(false);
    let current_time = RwSignal::new(0.0f64);
    let duration = RwSignal::new(0.0f64);
    let loaded = RwSignal::new(false);

    // Revoke object URL when component unmounts
    {
        let src_clone = src.clone();
        on_cleanup(move || {
            let _ = Url::revoke_object_url(&src_clone);
        });
    }

    // Wire DOM events via Effect once the <audio> node is mounted.
    // IMPORTANT: use get_untracked — we do not want this Effect to re-run if
    // audio_ref ever changes. The handlers are attached via .forget() (no removal),
    // so a re-run would accumulate duplicate handlers. get_untracked means the
    // Effect fires exactly once on mount and is never re-triggered.
    Effect::new(move |_| {
        if let Some(audio) = audio_ref.get_untracked() {
            let audio_el: &web_sys::HtmlAudioElement = audio.as_ref();

            let ct_signal = current_time;
            let ct_handler = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                if let Some(a) = audio_ref.get() {
                    ct_signal.set(a.current_time());
                }
            });
            audio_el.set_ontimeupdate(Some(ct_handler.as_ref().unchecked_ref()));
            ct_handler.forget();

            let dur_signal = duration;
            let loaded_signal = loaded;
            let dur_handler = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                if let Some(a) = audio_ref.get() {
                    dur_signal.set(a.duration());
                    loaded_signal.set(true);
                }
            });
            audio_el.set_onloadedmetadata(Some(dur_handler.as_ref().unchecked_ref()));
            dur_handler.forget();

            let play_signal = playing;
            let ended_handler = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
                play_signal.set(false);
            });
            audio_el.set_onended(Some(ended_handler.as_ref().unchecked_ref()));
            ended_handler.forget();
        }
    });

    let on_play_pause = move |_| {
        if let Some(audio) = audio_ref.get() {
            let audio_el: web_sys::HtmlMediaElement = (*audio).clone();
            if playing.get_untracked() {
                audio_el.pause().ok();
                playing.set(false);
            } else {
                spawn_local(async move {
                    let _ = JsFuture::from(audio_el.play().unwrap()).await;
                    playing.set(true);
                });
            }
        }
    };

    let on_seek = move |ev: web_sys::Event| {
        if let Some(audio) = audio_ref.get() {
            let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
            let val: f64 = input.value().parse().unwrap_or(0.0);
            audio.set_current_time(val);
            current_time.set(val);
        }
    };

    let src_for_audio = src.clone();

    view! {
        <div class="audio-player" role="group" aria-label="audio player">
            <audio
                node_ref=audio_ref
                src=src_for_audio
                preload="metadata"
            />
            <button
                class="audio-player__play-btn"
                aria-label=move || if playing.get() { "Pause" } else { "Play" }
                on:click=on_play_pause
            >
                {move || if playing.get() { "⏸" } else { "▶" }}
            </button>
            <span class="audio-player__time">
                {move || format_duration(current_time.get())}
            </span>
            <input
                class="audio-player__seek"
                type="range"
                min="0"
                step="0.1"
                max=move || duration.get().to_string()
                value=move || current_time.get().to_string()
                aria-label="seek"
                on:input=on_seek
            />
            <span class="audio-player__duration">
                {move || if loaded.get() { format_duration(duration.get()) } else { "0:00".to_string() }}
            </span>
        </div>
    }
}
