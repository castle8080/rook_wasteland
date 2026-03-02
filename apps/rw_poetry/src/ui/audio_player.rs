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
/// Returns `"0:00"` for non-finite values (NaN, Infinity) which the browser
/// may briefly report via `HTMLMediaElement.duration` before metadata loads.
pub fn format_duration(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00".to_string();
    }
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

    #[test]
    fn format_infinity_returns_zero() {
        assert_eq!(format_duration(f64::INFINITY), "0:00");
    }

    #[test]
    fn format_nan_returns_zero() {
        assert_eq!(format_duration(f64::NAN), "0:00");
    }

    #[test]
    fn format_negative_returns_zero() {
        assert_eq!(format_duration(-1.0), "0:00");
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Custom audio player backed by a hidden `<audio>` element.
///
/// `src` must be an object URL or asset URL pointing to the audio data.
///
/// `duration_hint_secs` — optional clock-measured duration from recording metadata.
/// Browser-recorded WebM/Opus files (from MediaRecorder) always report
/// `duration = Infinity`. Passing the hint allows the player to display the
/// correct seek range immediately, before the seek-trick refinement completes.
#[component]
pub fn AudioPlayer(
    /// Object URL or audio asset URL
    src: String,
    /// Optional duration hint from recording metadata (seconds).
    #[prop(default = None)]
    duration_hint_secs: Option<f64>,
) -> impl IntoView {
    let audio_ref: NodeRef<html::Audio> = NodeRef::new();
    let playing = RwSignal::new(false);
    let current_time = RwSignal::new(0.0f64);
    let duration = RwSignal::new(0.0f64);
    let loaded = RwSignal::new(false);

    // Flag: true while the seek-to-end trick is in progress.
    // Shared (via Copy) between the onloadedmetadata and ontimeupdate handlers.
    // get_untracked/set only — never used as a reactive dependency.
    let fixing_duration = RwSignal::new(false);

    // Revoke object URL when component unmounts
    {
        let src_clone = src.clone();
        on_cleanup(move || {
            let _ = Url::revoke_object_url(&src_clone);
        });
    }

    // Wire DOM events via Effect once the <audio> node is mounted.
    // get_untracked throughout: we do not want this Effect to re-run.
    // Handlers are attached via .forget() (no removal), so a re-run would
    // accumulate duplicate handlers.
    Effect::new(move |_| {
        let Some(audio) = audio_ref.get_untracked() else { return; };
        let audio_el: &web_sys::HtmlAudioElement = audio.as_ref();

        // --- ontimeupdate ---
        // Dual role, selected by the fixing_duration flag:
        //
        // (A) Seek-trick mode (fixing_duration = true):
        //     The browser has just seeked toward 1e9 s and fired timeupdate,
        //     meaning it has moved to its best approximation of that position
        //     (i.e., near EOF). currentTime ≈ real duration. Record it, clear
        //     the flag, and reset the playhead to 0.
        //
        // (B) Normal mode: update the current position signal.
        let ct_handler = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            let Some(a) = audio_ref.get_untracked() else { return; };
            if fixing_duration.get_untracked() {
                let ct = a.current_time();
                if ct <= 0.0 {
                    return; // seek not complete yet; wait for next timeupdate
                }
                // Use audio.duration() if it has been resolved, otherwise fall
                // back to currentTime (which is near EOF = ≈ real duration).
                let dur = if a.duration().is_finite() && a.duration() > 0.0 {
                    a.duration()
                } else {
                    ct
                };
                fixing_duration.set(false);
                duration.set(dur);
                loaded.set(true);
                a.set_current_time(0.0);
            } else {
                // get_untracked: DOM event handler, not a reactive context.
                current_time.set(a.current_time());
            }
        });
        audio_el.set_ontimeupdate(Some(ct_handler.as_ref().unchecked_ref()));
        ct_handler.forget();

        // --- onloadedmetadata ---
        // For regular audio: duration is finite and correct immediately.
        // For WebM from MediaRecorder: duration = Infinity.
        //   - If a hint was passed, apply it right away so the UI is usable.
        //   - Start the seek-to-end trick to refine to the precise value.
        //     The result is picked up by ontimeupdate above.
        let lm_handler = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            let Some(a) = audio_ref.get_untracked() else { return; };
            let d = a.duration();
            if d.is_finite() && d > 0.0 {
                duration.set(d);
                loaded.set(true);
            } else {
                // WebM Infinity duration: use hint for immediate display.
                if let Some(hint) = duration_hint_secs {
                    duration.set(hint);
                    loaded.set(true);
                }
                // Seek to near EOF so ontimeupdate can extract the real duration.
                fixing_duration.set(true);
                a.set_current_time(1e9);
            }
        });
        audio_el.set_onloadedmetadata(Some(lm_handler.as_ref().unchecked_ref()));
        lm_handler.forget();

        // --- onended ---
        let ended_handler = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            playing.set(false);
        });
        audio_el.set_onended(Some(ended_handler.as_ref().unchecked_ref()));
        ended_handler.forget();

        // Belt-and-suspenders: read duration eagerly in case loadedmetadata
        // already fired before this Effect ran (blob URL in memory).
        let eager = audio_el.duration();
        if eager.is_finite() && eager > 0.0 {
            duration.set(eager);
            loaded.set(true);
        }
    });

    let on_play_pause = move |_| {
        // get_untracked: on:click handlers are DOM event callbacks, not reactive contexts.
        if let Some(audio) = audio_ref.get_untracked() {
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
        // get_untracked: on:input handlers are DOM event callbacks, not reactive contexts.
        if let Some(audio) = audio_ref.get_untracked() {
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
                prop:value=move || current_time.get().to_string()
                aria-label="seek"
                on:input=on_seek
            />
            <span class="audio-player__duration">
                {move || if loaded.get() { format_duration(duration.get()) } else { "0:00".to_string() }}
            </span>
        </div>
    }
}
