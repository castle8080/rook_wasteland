//! BPM detection via spectral flux + autocorrelation.
//!
//! All public functions are pure Rust — no web-sys types — and are fully
//! unit-testable on the host with `cargo test`.

use rustfft::{FftPlanner, num_complex::Complex};

/// FFT window size used by [`compute_spectral_flux`].
pub const WINDOW_SIZE: usize = 1024;
/// Hop (step) size between successive frames.
pub const HOP_SIZE: usize = 512;

/// Compute per-frame half-wave-rectified spectral flux with 5-frame smoothing.
///
/// Steps:
/// 1. Frame the signal with a 1024-sample Hanning window, 512-sample hop.
/// 2. FFT each frame; compute magnitude spectrum.
/// 3. Half-wave rectify the per-bin difference vs. the previous frame and sum.
/// 4. Smooth the resulting curve with a 5-frame centred moving average.
///
/// Returns one flux value per frame; empty if `samples` is shorter than `WINDOW_SIZE`.
pub fn compute_spectral_flux(samples: &[f32], _sample_rate: f32) -> Vec<f32> {
    if samples.len() < WINDOW_SIZE {
        return Vec::new();
    }

    // Pre-compute Hanning window coefficients.
    let hann: Vec<f32> = (0..WINDOW_SIZE)
        .map(|i| {
            let phase = 2.0 * std::f32::consts::PI * i as f32 / (WINDOW_SIZE - 1) as f32;
            0.5 * (1.0 - phase.cos())
        })
        .collect();

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(WINDOW_SIZE);

    // Only positive frequencies are needed (DC…Nyquist).
    let half = WINDOW_SIZE / 2 + 1;
    let num_frames = (samples.len() - WINDOW_SIZE) / HOP_SIZE + 1;

    let mut prev_mag = vec![0.0f32; half];
    let mut flux: Vec<f32> = Vec::with_capacity(num_frames);

    for frame_idx in 0..num_frames {
        let start = frame_idx * HOP_SIZE;
        let end = start + WINDOW_SIZE;
        if end > samples.len() {
            break;
        }

        let mut buf: Vec<Complex<f32>> = samples[start..end]
            .iter()
            .zip(hann.iter())
            .map(|(&s, &w)| Complex::new(s * w, 0.0))
            .collect();

        fft.process(&mut buf);

        let mag: Vec<f32> = buf[..half].iter().map(|c| c.norm()).collect();

        // Half-wave-rectified spectral flux: sum max(0, |X_k(t)| - |X_k(t-1)|).
        let frame_flux: f32 = mag
            .iter()
            .zip(prev_mag.iter())
            .map(|(&m, &p)| (m - p).max(0.0))
            .sum();

        flux.push(frame_flux);
        prev_mag = mag;
    }

    // 5-frame centred moving average (half-width = 2).
    let hw = 2usize;
    (0..flux.len())
        .map(|i| {
            let lo = i.saturating_sub(hw);
            let hi = (i + hw + 1).min(flux.len());
            let sum: f32 = flux[lo..hi].iter().sum();
            sum / (hi - lo) as f32
        })
        .collect()
}

/// Estimate BPM from a spectral-flux vector via autocorrelation.
///
/// Searches lags that correspond to tempos in [60, 200] BPM, finds the
/// lag with maximum autocorrelation energy, converts to BPM, applies one
/// round of octave correction, and clamps to [60.0, 200.0].
///
/// `hop` should match the hop size used in [`compute_spectral_flux`] (512).
pub fn estimate_bpm(flux: &[f32], sample_rate: f32, hop: usize) -> f64 {
    if flux.is_empty() || hop == 0 {
        return 120.0;
    }

    let fps = sample_rate as f64 / hop as f64; // frames per second

    // Lag (in frames) corresponding to the BPM range [60, 200].
    let lag_min = (fps * 60.0 / 200.0).ceil() as usize;
    let lag_max = ((fps * 60.0 / 60.0).floor() as usize).min(flux.len() / 2);

    if lag_min >= lag_max || lag_min == 0 {
        return 120.0;
    }

    let n = flux.len();
    let (mut best_lag, mut best_corr) = (lag_min, f64::NEG_INFINITY);

    for lag in lag_min..=lag_max {
        let corr: f64 = flux[..n - lag]
            .iter()
            .zip(&flux[lag..])
            .map(|(&a, &b)| a as f64 * b as f64)
            .sum();
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    // Sub-lag check: prefer the half-lag (double the BPM) when it has
    // similar autocorrelation energy to the best lag. This corrects the
    // common case where the algorithm finds a double-period (half-tempo)
    // lag due to integer-alignment artefacts.
    let half = best_lag / 2;
    if half >= lag_min {
        let half_corr: f64 = flux[..n - half]
            .iter()
            .zip(&flux[half..])
            .map(|(&a, &b)| a as f64 * b as f64)
            .sum();
        if half_corr >= 0.5 * best_corr {
            best_lag = half;
            best_corr = half_corr;
        }
    }
    let _ = best_corr; // used only for the sub-lag comparison

    let bpm = fps * 60.0 / best_lag as f64;

    // Octave correction: one step up or down to land in a natural range.
    let bpm = if bpm < 60.0 { bpm * 2.0 } else { bpm };
    let bpm = if bpm < 60.0 { bpm * 2.0 } else { bpm };
    let bpm = if bpm > 200.0 { bpm / 2.0 } else { bpm };
    let bpm = if bpm > 200.0 { bpm / 2.0 } else { bpm };

    bpm.clamp(60.0, 200.0)
}

/// Compute BPM from a rolling window of consecutive tap intervals (milliseconds).
///
/// Pure function — no web-sys dependencies. Used by the TAP BPM button handler.
/// Returns `None` when fewer than 2 intervals are available.
pub fn tap_bpm_from_intervals(intervals_ms: &[f64]) -> Option<f64> {
    if intervals_ms.len() < 2 {
        return None;
    }
    let avg_ms = intervals_ms.iter().sum::<f64>() / intervals_ms.len() as f64;
    if avg_ms <= 0.0 {
        return None;
    }
    Some((60_000.0 / avg_ms).clamp(40.0, 250.0))
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 44100.0;

    /// Generate a simple kick-drum impulse train at `bpm`.
    fn impulse_train(bpm: f64, duration_secs: f32, sample_rate: f32) -> Vec<f32> {
        let n = (duration_secs * sample_rate) as usize;
        let period = (60.0 / bpm * sample_rate as f64) as usize;
        let mut samples = vec![0.0f32; n];
        let mut i = 0;
        while i < n {
            // Gaussian-shaped impulse width ≈ 64 samples.
            for k in 0..64usize {
                let idx = i + k;
                if idx < n {
                    let x = k as f32 / 16.0;
                    samples[idx] += (-x * x).exp();
                }
            }
            i += period;
        }
        samples
    }

    #[test]
    fn spectral_flux_non_empty_for_beat_signal() {
        let samples = impulse_train(128.0, 10.0, SR);
        let flux = compute_spectral_flux(&samples, SR);
        assert!(!flux.is_empty(), "flux should have frames");
        assert!(
            flux.iter().any(|&v| v > 0.0),
            "flux should be non-zero on a beat signal"
        );
    }

    #[test]
    fn spectral_flux_near_zero_on_silence() {
        let samples = vec![0.0f32; SR as usize * 5];
        let flux = compute_spectral_flux(&samples, SR);
        let max_flux = flux.iter().cloned().fold(0.0f32, f32::max);
        assert!(
            max_flux < 1e-3,
            "silence should produce near-zero flux, got {max_flux}"
        );
    }

    #[test]
    fn spectral_flux_returns_empty_for_short_input() {
        let short = vec![0.1f32; 512]; // less than WINDOW_SIZE
        assert!(compute_spectral_flux(&short, SR).is_empty());
    }

    #[test]
    fn estimate_bpm_on_128_beat() {
        let samples = impulse_train(128.0, 20.0, SR);
        let flux = compute_spectral_flux(&samples, SR);
        let bpm = estimate_bpm(&flux, SR, HOP_SIZE);
        assert!(
            (bpm - 128.0).abs() < 5.0,
            "expected ~128 BPM, got {bpm:.1}"
        );
    }

    #[test]
    fn estimate_bpm_on_90_beat() {
        let samples = impulse_train(90.0, 30.0, SR);
        let flux = compute_spectral_flux(&samples, SR);
        let bpm = estimate_bpm(&flux, SR, HOP_SIZE);
        assert!(
            (bpm - 90.0).abs() < 5.0,
            "expected ~90 BPM, got {bpm:.1}"
        );
    }

    #[test]
    fn estimate_bpm_result_in_range() {
        let samples = impulse_train(120.0, 15.0, SR);
        let flux = compute_spectral_flux(&samples, SR);
        let bpm = estimate_bpm(&flux, SR, HOP_SIZE);
        assert!(bpm >= 60.0 && bpm <= 200.0, "BPM {bpm:.1} out of range");
    }

    #[test]
    fn estimate_bpm_empty_flux_returns_120() {
        assert_eq!(estimate_bpm(&[], SR, HOP_SIZE), 120.0);
    }

    #[test]
    fn tap_bpm_none_for_single_interval() {
        assert_eq!(tap_bpm_from_intervals(&[500.0]), None);
    }

    #[test]
    fn tap_bpm_none_for_empty_slice() {
        assert_eq!(tap_bpm_from_intervals(&[]), None);
    }

    #[test]
    fn tap_bpm_120_for_500ms_intervals() {
        let intervals = vec![500.0f64; 7];
        let bpm = tap_bpm_from_intervals(&intervals).unwrap();
        assert!((bpm - 120.0).abs() < 0.1, "expected 120 BPM, got {bpm}");
    }

    #[test]
    fn tap_bpm_90_for_667ms_intervals() {
        let intervals = vec![666.67f64; 6];
        let bpm = tap_bpm_from_intervals(&intervals).unwrap();
        assert!((bpm - 90.0).abs() < 1.0, "expected ~90 BPM, got {bpm}");
    }

    #[test]
    fn spectral_flux_exactly_window_size_gives_one_frame() {
        // Exactly WINDOW_SIZE samples → exactly 1 frame, so 1 flux value.
        let samples = vec![0.5f32; WINDOW_SIZE];
        let flux = compute_spectral_flux(&samples, SR);
        assert_eq!(flux.len(), 1, "expected 1 frame for WINDOW_SIZE input");
    }

    #[test]
    fn estimate_bpm_hop_zero_returns_120() {
        let flux = vec![1.0f32; 200];
        assert_eq!(estimate_bpm(&flux, SR, 0), 120.0);
    }

    #[test]
    fn estimate_bpm_flux_too_short_for_lag_range_returns_120() {
        // With SR=44100 and hop=512: lag_min ≈ 26, lag_max = flux.len()/2.
        // If flux.len() = 40, lag_max = 20 < lag_min → early return.
        let flux = vec![1.0f32; 40];
        assert_eq!(estimate_bpm(&flux, SR, HOP_SIZE), 120.0);
    }

    #[test]
    fn estimate_bpm_on_170_beat() {
        let samples = impulse_train(170.0, 20.0, SR);
        let flux = compute_spectral_flux(&samples, SR);
        let bpm = estimate_bpm(&flux, SR, HOP_SIZE);
        assert!(
            (bpm - 170.0).abs() < 8.0,
            "expected ~170 BPM, got {bpm:.1}"
        );
    }

    #[test]
    fn estimate_bpm_output_always_in_spec_range() {
        // For any realistic input, result must be in [60, 200].
        for target in [70u32, 90, 110, 128, 140, 160, 175] {
            let samples = impulse_train(target as f64, 15.0, SR);
            let flux = compute_spectral_flux(&samples, SR);
            let bpm = estimate_bpm(&flux, SR, HOP_SIZE);
            assert!(
                bpm >= 60.0 && bpm <= 200.0,
                "BPM {bpm:.1} out of [60,200] for target {target}"
            );
        }
    }
}
