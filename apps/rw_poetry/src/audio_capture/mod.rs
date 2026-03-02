// Items in this module are consumed starting in T08.
#![allow(dead_code)]

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, MediaRecorder, MediaRecorderOptions, MediaStream};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum MicError {
    PermissionDenied,
    NoDevice,
    HardwareError,
    NotSupported,
    Unexpected(String),
}

impl fmt::Display for MicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MicError::PermissionDenied => write!(
                f,
                "Microphone access was denied. Open browser settings to allow access."
            ),
            MicError::NoDevice => write!(f, "No microphone found on this device."),
            MicError::HardwareError => write!(
                f,
                "Could not access the microphone. Another app may be using it."
            ),
            MicError::NotSupported => {
                write!(f, "This browser doesn't support audio recording.")
            }
            MicError::Unexpected(msg) => write!(f, "Microphone error: {msg}"),
        }
    }
}

impl MicError {
    fn from_js(err: &JsValue) -> Self {
        let name = js_sys::Reflect::get(err, &JsValue::from_str("name"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        match name.as_str() {
            "NotAllowedError" | "PermissionDeniedError" => MicError::PermissionDenied,
            "NotFoundError" | "DevicesNotFoundError" => MicError::NoDevice,
            "NotReadableError" | "TrackStartError" => MicError::HardwareError,
            "NotSupportedError" => MicError::NotSupported,
            other => MicError::Unexpected(other.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// AudioBlob
// ---------------------------------------------------------------------------

pub struct AudioBlob {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub duration_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// MIME type negotiation
// ---------------------------------------------------------------------------

/// Returns the best supported MIME type from the preferred fallback order.
pub fn pick_mime_type() -> &'static str {
    let candidates = [
        "audio/webm;codecs=opus",
        "audio/webm",
        "audio/ogg;codecs=opus",
        "audio/ogg",
        "audio/mp4",
    ];
    for mime in candidates {
        if MediaRecorder::is_type_supported(mime) {
            return mime;
        }
    }
    "audio/webm"
}

// ---------------------------------------------------------------------------
// AudioRecorder
// ---------------------------------------------------------------------------

pub struct AudioRecorder {
    recorder: MediaRecorder,
    stream: MediaStream,
    /// Blob chunks collected by ondataavailable
    chunks: Rc<RefCell<Vec<Blob>>>,
    start_time_ms: f64,
}

impl AudioRecorder {
    /// Start recording from the given stream.
    pub fn start(stream: &MediaStream, mime_type: &str) -> Result<Self, MicError> {
        let options = MediaRecorderOptions::new();
        options.set_mime_type(mime_type);

        let recorder =
            MediaRecorder::new_with_media_stream_and_media_recorder_options(stream, &options)
                .map_err(|e| MicError::from_js(&e))?;

        let chunks: Rc<RefCell<Vec<Blob>>> = Rc::new(RefCell::new(Vec::new()));
        let chunks_clone = chunks.clone();

        let on_data: Closure<dyn Fn(web_sys::BlobEvent)> =
            Closure::new(move |event: web_sys::BlobEvent| {
                if let Some(blob) = event.data().filter(|b| b.size() > 0.0) {
                    chunks_clone.borrow_mut().push(blob);
                }
            });

        recorder.set_ondataavailable(Some(on_data.as_ref().unchecked_ref()));
        on_data.forget(); // kept alive for the duration of recording

        recorder
            .start_with_time_slice(100)
            .map_err(|e| MicError::from_js(&e))?;

        Ok(AudioRecorder {
            recorder,
            stream: stream.clone(),
            chunks,
            start_time_ms: js_sys::Date::now(),
        })
    }

    /// Stop recording and collect all audio data into a single `AudioBlob`.
    pub async fn stop(self) -> Result<AudioBlob, MicError> {
        let duration_ms = (js_sys::Date::now() - self.start_time_ms) as u64;
        let chunks = self.chunks.clone();
        let mime_type = self.recorder.mime_type();

        // Set up a promise that resolves when onstop fires (after final data arrives).
        let (tx, rx) = futures_channel_oneshot();

        let on_stop: Closure<dyn FnMut()> = Closure::new(move || {
            let _ = tx.borrow_mut().take().map(|s: js_sys::Function| {
                let _ = s.call0(&JsValue::NULL);
            });
        });
        self.recorder
            .set_onstop(Some(on_stop.as_ref().unchecked_ref()));

        self.recorder
            .stop()
            .map_err(|e| MicError::from_js(&e))?;

        rx.await;

        on_stop.forget();

        // Release microphone tracks
        let tracks = self.stream.get_audio_tracks();
        for i in 0..tracks.length() {
            if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                track.stop();
            }
        }

        // Combine all chunks into one Blob
        let parts_array = js_sys::Array::new();
        for blob in chunks.borrow().iter() {
            parts_array.push(blob.as_ref());
        }

        let props = BlobPropertyBag::new();
        props.set_type(&mime_type);

        let combined_blob =
            Blob::new_with_blob_sequence_and_options(&parts_array, &props)
                .map_err(|e| MicError::Unexpected(format!("{e:?}")))?;

        // Read combined blob as ArrayBuffer → Vec<u8>
        let ab = JsFuture::from(combined_blob.array_buffer())
            .await
            .map_err(|e| MicError::Unexpected(format!("{e:?}")))?;

        let data = js_sys::Uint8Array::new(&ab).to_vec();

        Ok(AudioBlob {
            data,
            mime_type,
            duration_ms: Some(duration_ms),
        })
    }
}

/// Simple one-shot channel using a shared Rc<RefCell<Option<js_sys::Function>>>.
/// Returns (sender_holder, future_that_resolves_when_sender_called).
fn futures_channel_oneshot() -> (
    Rc<RefCell<Option<js_sys::Function>>>,
    impl std::future::Future<Output = ()>,
) {
    let resolve_cell: Rc<RefCell<Option<js_sys::Function>>> = Rc::new(RefCell::new(None));
    let resolve_cell_clone = resolve_cell.clone();

    let promise = js_sys::Promise::new(&mut |resolve: js_sys::Function, _| {
        *resolve_cell_clone.borrow_mut() = Some(resolve);
    });

    let future = async move {
        JsFuture::from(promise).await.ok();
    };

    (resolve_cell, future)
}

// ---------------------------------------------------------------------------
// Mic permission
// ---------------------------------------------------------------------------

/// Request microphone access via getUserMedia({ audio: true }).
pub async fn request_mic() -> Result<MediaStream, MicError> {
    let window = web_sys::window().ok_or(MicError::Unexpected("no window".to_string()))?;
    let media_devices = window
        .navigator()
        .media_devices()
        .map_err(|_| MicError::NotSupported)?;

    let constraints = web_sys::MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::TRUE);

    let promise = media_devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| MicError::from_js(&e))?;

    let result = JsFuture::from(promise)
        .await
        .map_err(|e| MicError::from_js(&e))?;

    result.dyn_into::<MediaStream>().map_err(|_| {
        MicError::Unexpected("getUserMedia did not return a MediaStream".to_string())
    })
}

