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
/// Thin web-sys wrapper around [`extract_peaks_from_samples`].
pub fn extract_peaks(buffer: &AudioBuffer, num_columns: usize) -> Vec<f32> {
    let channels: Vec<Vec<f32>> = (0..buffer.number_of_channels())
        .map(|c| buffer.get_channel_data(c).unwrap_or_default())
        .collect();
    extract_peaks_from_samples(&channels, buffer.length() as usize, num_columns)
}

/// Pure peak extraction — no web-sys types, fully unit-testable on the host.
///
/// `channels` is a slice of per-channel sample vectors (any length).
/// `total_length` is the canonical sample count (may differ from `channels[i].len()`
/// if the browser returned a shorter slice for some reason).
/// Returns `num_columns` values in `[0.0, 1.0]` representing max absolute amplitude
/// per column bucket.
pub fn extract_peaks_from_samples(
    channels: &[Vec<f32>],
    total_length: usize,
    num_columns: usize,
) -> Vec<f32> {
    if total_length == 0 || num_columns == 0 || channels.is_empty() {
        return vec![0.0; num_columns];
    }
    let samples_per_col = (total_length / num_columns).max(1);

    (0..num_columns)
        .map(|i| {
            let start = i * samples_per_col;
            // Last column extends to total_length so no samples are dropped
            // when total_length is not evenly divisible by num_columns.
            let end = if i == num_columns - 1 {
                total_length
            } else {
                (start + samples_per_col).min(total_length)
            };
            let mut peak = 0.0f32;
            for sample_idx in start..end {
                for ch in channels {
                    if sample_idx < ch.len() {
                        peak = peak.max(ch[sample_idx].abs());
                    }
                }
            }
            peak
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peaks_returns_correct_column_count() {
        let ch = vec![0.5f32; 2048];
        let result = extract_peaks_from_samples(&[ch], 2048, 100);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn peaks_all_same_amplitude() {
        let ch = vec![0.75f32; 1024];
        let result = extract_peaks_from_samples(&[ch], 1024, 64);
        assert!(result.iter().all(|&v| (v - 0.75).abs() < 1e-6),
            "all columns should equal 0.75, got {result:?}");
    }

    #[test]
    fn peaks_negative_samples_abs() {
        // Negative samples should be treated as absolute value.
        let ch = vec![-0.9f32; 512];
        let result = extract_peaks_from_samples(&[ch], 512, 32);
        assert!(result.iter().all(|&v| (v - 0.9).abs() < 1e-6));
    }

    #[test]
    fn peaks_mixed_channels_take_max() {
        // Channel 0: all 0.3, Channel 1: all 0.8 — peak should be 0.8.
        let ch0 = vec![0.3f32; 512];
        let ch1 = vec![0.8f32; 512];
        let result = extract_peaks_from_samples(&[ch0, ch1], 512, 16);
        assert!(result.iter().all(|&v| (v - 0.8).abs() < 1e-6));
    }

    #[test]
    fn peaks_silence_is_zero() {
        let ch = vec![0.0f32; 1024];
        let result = extract_peaks_from_samples(&[ch], 1024, 50);
        assert!(result.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn peaks_empty_channels_returns_zeros() {
        let result = extract_peaks_from_samples(&[], 0, 10);
        assert_eq!(result, vec![0.0f32; 10]);
    }

    #[test]
    fn peaks_zero_columns_returns_empty() {
        let ch = vec![1.0f32; 512];
        let result = extract_peaks_from_samples(&[ch], 512, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn peaks_last_column_covers_remainder() {
        // 100 samples into 3 columns: cols cover [0..33], [33..66], [66..99] approx.
        // Last column should still return a valid peak, not 0.
        let mut ch = vec![0.0f32; 100];
        ch[99] = 1.0; // spike at the very end
        let result = extract_peaks_from_samples(&[ch], 100, 3);
        assert_eq!(result.len(), 3);
        // At least one column must contain the spike
        assert!(result.iter().any(|&v| v > 0.5));
    }

    // --- WASM tests: exercise the web-sys wrapper with a real AudioBuffer ---

    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;
        use crate::audio::context::ensure_audio_context;

        fn make_constant_buffer(num_channels: u32, num_samples: u32, value: f32) -> web_sys::AudioBuffer {
            let holder = std::rc::Rc::new(std::cell::RefCell::new(None));
            let ctx = ensure_audio_context(&holder);
            let buf = ctx.create_buffer(num_channels, num_samples, 44100.0)
                .expect("create_buffer");
            let data: Vec<f32> = vec![value; num_samples as usize];
            for ch in 0..num_channels {
                buf.copy_to_channel(&data, ch as i32).expect("copy_to_channel");
            }
            buf
        }

        #[wasm_bindgen_test]
        fn wrapper_returns_correct_column_count() {
            let buf = make_constant_buffer(1, 44100, 0.5);
            assert_eq!(extract_peaks(&buf, 100).len(), 100);
        }

        #[wasm_bindgen_test]
        fn wrapper_constant_buffer_all_same_value() {
            let buf = make_constant_buffer(2, 44100, 0.6);
            for (i, &v) in extract_peaks(&buf, 64).iter().enumerate() {
                assert!((v - 0.6).abs() < 1e-4, "col {i}: expected ~0.6, got {v}");
            }
        }

        #[wasm_bindgen_test]
        fn wrapper_silence_is_all_zero() {
            let buf = make_constant_buffer(2, 44100, 0.0);
            assert!(extract_peaks(&buf, 50).iter().all(|&v| v == 0.0));
        }

        #[wasm_bindgen_test]
        fn wrapper_stereo_takes_max_channel() {
            // ch0 = 0.3, ch1 = 0.9  →  peaks should be ≈ 0.9
            let holder = std::rc::Rc::new(std::cell::RefCell::new(None));
            let ctx = ensure_audio_context(&holder);
            let buf = ctx.create_buffer(2, 4096, 44100.0).expect("create_buffer");
            buf.copy_to_channel(&vec![0.3f32; 4096], 0).expect("copy ch0");
            buf.copy_to_channel(&vec![0.9f32; 4096], 1).expect("copy ch1");
            for (i, &v) in extract_peaks(&buf, 32).iter().enumerate() {
                assert!((v - 0.9).abs() < 1e-4, "col {i}: expected ~0.9, got {v}");
            }
        }
    }
}
