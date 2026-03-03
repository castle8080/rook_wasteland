use js_sys::{ArrayBuffer, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, AudioBuffer, FileReader};
use std::rc::Rc;
use std::cell::RefCell;
use leptos::prelude::Set;
use crate::audio::deck_audio::AudioDeck;
use crate::state::DeckState;

pub async fn load_audio_file(
    file: web_sys::File,
    deck: Rc<RefCell<AudioDeck>>,
    state: DeckState,
    ctx: AudioContext,
) {
    let file_name = file.name();

    // Step 1: File → ArrayBuffer via FileReader wrapped in a Promise
    let array_buffer = read_file_as_array_buffer(file).await;

    // Step 2: ArrayBuffer → AudioBuffer via Web Audio decodeAudioData
    let promise: Promise = ctx.decode_audio_data(&array_buffer)
        .expect("decode_audio_data");
    let audio_buffer: AudioBuffer = JsFuture::from(promise)
        .await
        .expect("decodeAudioData future")
        .unchecked_into();

    // Step 3: Extract waveform peaks and duration
    let duration = audio_buffer.duration();
    let peaks = extract_peaks(&audio_buffer, 1024);

    // Step 4: Store buffer and update reactive state
    deck.borrow_mut().buffer = Some(audio_buffer);
    state.track_name.set(Some(file_name));
    state.duration_secs.set(duration);
    state.waveform_peaks.set(Some(peaks));
}

async fn read_file_as_array_buffer(file: web_sys::File) -> ArrayBuffer {
    let promise = Promise::new(&mut |resolve, _reject| {
        let reader = FileReader::new().expect("FileReader::new");
        let reader_clone = reader.clone();
        let onload = Closure::<dyn FnMut(web_sys::ProgressEvent)>::new(
            move |_: web_sys::ProgressEvent| {
                let result = reader_clone.result().expect("FileReader.result");
                resolve.call1(&JsValue::NULL, &result).expect("resolve");
            },
        );
        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        reader.read_as_array_buffer(&file).expect("read_as_array_buffer");
    });
    JsFuture::from(promise)
        .await
        .expect("FileReader promise")
        .unchecked_into::<ArrayBuffer>()
}

/// Downsamples all audio channels to `num_columns` peak values.
pub fn extract_peaks(buffer: &AudioBuffer, num_columns: usize) -> Vec<f32> {
    let num_channels = buffer.number_of_channels();
    let length = buffer.length() as usize;
    if length == 0 || num_columns == 0 {
        return vec![0.0; num_columns];
    }
    let samples_per_col = (length / num_columns).max(1);

    let channels: Vec<Vec<f32>> = (0..num_channels)
        .map(|c| buffer.get_channel_data(c).unwrap_or_default())
        .collect();

    (0..num_columns)
        .map(|i| {
            let start = i * samples_per_col;
            let end = (start + samples_per_col).min(length);
            let mut peak = 0.0f32;
            for sample_idx in start..end {
                for ch in &channels {
                    if sample_idx < ch.len() {
                        peak = peak.max(ch[sample_idx].abs());
                    }
                }
            }
            peak
        })
        .collect()
}
