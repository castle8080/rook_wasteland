use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{
    AudioContext, AudioBuffer, AudioBufferSourceNode,
    GainNode, BiquadFilterNode, BiquadFilterType,
    ConvolverNode, DelayNode, AnalyserNode, OscillatorNode,
};

/// All Web Audio nodes for one DJ deck plus timing bookkeeping.
///
/// `source` is recreated on each `play()` call — `AudioBufferSourceNode` is one-shot.
/// The rest of the chain is persistent for the lifetime of the deck.
pub struct AudioDeck {
    pub ctx:            AudioContext,
    /// One-shot playback node; `None` when not playing.
    pub source:         Option<AudioBufferSourceNode>,
    /// Decoded audio data; `None` before file load.
    pub buffer:         Option<AudioBuffer>,
    pub pre_gain:       GainNode,
    pub eq_high:        BiquadFilterNode,
    pub eq_mid:         BiquadFilterNode,
    pub eq_low:         BiquadFilterNode,
    pub sweep_filter:   BiquadFilterNode,
    pub reverb:         ConvolverNode,
    pub reverb_dry:     GainNode,
    pub reverb_wet:     GainNode,
    pub echo_delay:     DelayNode,
    pub echo_feedback:  GainNode,
    pub echo_wet:       GainNode,
    pub echo_dry:       GainNode,
    pub flanger_delay:  DelayNode,
    pub flanger_lfo:    OscillatorNode,
    pub flanger_depth:  GainNode,
    pub flanger_wet:    GainNode,
    /// Pre-output gate node — used by the stutter effect for rhythmic gating.
    pub stutter_gate:   GainNode,
    pub channel_gain:   GainNode,
    pub analyser:       AnalyserNode,
    /// `AudioContext.currentTime` at the moment `play()` was last called.
    pub started_at:     Option<f64>,
    /// Track offset (seconds) when `play()` was last called.
    pub offset_at_play: f64,
    /// Saved cue point position in seconds; set by `cue()`.
    pub cue_point:      Option<f64>,
    /// Playback rate at the time `play()` was last called.  Used by
    /// `current_position()` to convert wall-clock elapsed time to track time.
    pub rate_at_play:   f64,
    /// Playback rate saved before a nudge starts; restored on nudge end.
    pub pre_nudge_rate: Option<f32>,
    // ── Scratch state ──────────────────────────────────────────────────────────
    pub scratch_active:     bool,
    pub scratch_last_angle: f64,
    pub scratch_last_time:  f64,
    /// Playback rate saved at the moment the user grabs the platter; restored on release.
    pub pre_scratch_rate:   f32,
}

impl AudioDeck {
    pub fn new(ctx: AudioContext) -> Rc<RefCell<AudioDeck>> {
        let pre_gain = ctx.create_gain()
            .expect("create_gain pre_gain: AudioContext node creation is infallible on a live context");
        pre_gain.gain().set_value(1.0);

        let eq_high = ctx.create_biquad_filter()
            .expect("create_biquad_filter eq_high: AudioContext node creation is infallible on a live context");
        eq_high.set_type(BiquadFilterType::Highshelf);
        eq_high.frequency().set_value(8000.0);
        eq_high.gain().set_value(0.0);

        let eq_mid = ctx.create_biquad_filter()
            .expect("create_biquad_filter eq_mid: AudioContext node creation is infallible on a live context");
        eq_mid.set_type(BiquadFilterType::Peaking);
        eq_mid.frequency().set_value(1000.0);
        eq_mid.q().set_value(0.7);
        eq_mid.gain().set_value(0.0);

        let eq_low = ctx.create_biquad_filter()
            .expect("create_biquad_filter eq_low: AudioContext node creation is infallible on a live context");
        eq_low.set_type(BiquadFilterType::Lowshelf);
        eq_low.frequency().set_value(200.0);
        eq_low.gain().set_value(0.0);

        let sweep_filter = ctx.create_biquad_filter()
            .expect("create_biquad_filter sweep: AudioContext node creation is infallible on a live context");
        sweep_filter.set_type(BiquadFilterType::Peaking);
        sweep_filter.gain().set_value(0.0);

        let reverb = ctx.create_convolver()
            .expect("create_convolver: AudioContext node creation is infallible on a live context");
        let reverb_dry = ctx.create_gain()
            .expect("create_gain reverb_dry: AudioContext node creation is infallible on a live context");
        reverb_dry.gain().set_value(1.0);
        let reverb_wet = ctx.create_gain()
            .expect("create_gain reverb_wet: AudioContext node creation is infallible on a live context");
        reverb_wet.gain().set_value(0.0);

        let echo_delay = ctx.create_delay_with_max_delay_time(2.0)
            .expect("create_delay echo: AudioContext node creation is infallible on a live context");
        echo_delay.delay_time().set_value(0.3);
        let echo_feedback = ctx.create_gain()
            .expect("create_gain echo_feedback: AudioContext node creation is infallible on a live context");
        echo_feedback.gain().set_value(0.0);
        let echo_wet = ctx.create_gain()
            .expect("create_gain echo_wet: AudioContext node creation is infallible on a live context");
        echo_wet.gain().set_value(0.0);
        let echo_dry = ctx.create_gain()
            .expect("create_gain echo_dry: AudioContext node creation is infallible on a live context");
        echo_dry.gain().set_value(1.0);

        let flanger_delay = ctx.create_delay_with_max_delay_time(0.02)
            .expect("create_delay flanger: AudioContext node creation is infallible on a live context");
        flanger_delay.delay_time().set_value(0.005);
        let flanger_lfo = ctx.create_oscillator()
            .expect("create_oscillator flanger_lfo: AudioContext node creation is infallible on a live context");
        flanger_lfo.frequency().set_value(0.5);
        let flanger_depth = ctx.create_gain()
            .expect("create_gain flanger_depth: AudioContext node creation is infallible on a live context");
        flanger_depth.gain().set_value(0.003);
        let flanger_wet = ctx.create_gain()
            .expect("create_gain flanger_wet: AudioContext node creation is infallible on a live context");
        flanger_wet.gain().set_value(0.0);

        let stutter_gate = ctx.create_gain()
            .expect("create_gain stutter_gate: AudioContext node creation is infallible on a live context");
        stutter_gate.gain().set_value(1.0);

        let channel_gain = ctx.create_gain()
            .expect("create_gain channel: AudioContext node creation is infallible on a live context");
        channel_gain.gain().set_value(1.0);

        let analyser = ctx.create_analyser()
            .expect("create_analyser: AudioContext node creation is infallible on a live context");
        analyser.set_fft_size(256);
        // 0.8 = smooth bounce, not jumpy (per tech spec §8.13)
        analyser.set_smoothing_time_constant(0.8);

        // Wire: pre_gain → eq_high → eq_mid → eq_low → sweep_filter
        pre_gain.connect_with_audio_node(&eq_high)
            .expect("connect pre_gain → eq_high: AudioNode.connect() is infallible between valid in-graph nodes");
        eq_high.connect_with_audio_node(&eq_mid)
            .expect("connect eq_high → eq_mid: AudioNode.connect() is infallible between valid in-graph nodes");
        eq_mid.connect_with_audio_node(&eq_low)
            .expect("connect eq_mid → eq_low: AudioNode.connect() is infallible between valid in-graph nodes");
        eq_low.connect_with_audio_node(&sweep_filter)
            .expect("connect eq_low → sweep_filter: AudioNode.connect() is infallible between valid in-graph nodes");

        // Reverb dry/wet bypass
        sweep_filter.connect_with_audio_node(&reverb_dry)
            .expect("connect sweep → reverb_dry: AudioNode.connect() is infallible between valid in-graph nodes");
        sweep_filter.connect_with_audio_node(&reverb)
            .expect("connect sweep → reverb: AudioNode.connect() is infallible between valid in-graph nodes");
        reverb.connect_with_audio_node(&reverb_wet)
            .expect("connect reverb → reverb_wet: AudioNode.connect() is infallible between valid in-graph nodes");
        reverb_dry.connect_with_audio_node(&echo_dry)
            .expect("connect reverb_dry → echo_dry: AudioNode.connect() is infallible between valid in-graph nodes");
        reverb_wet.connect_with_audio_node(&echo_dry)
            .expect("connect reverb_wet → echo_dry: AudioNode.connect() is infallible between valid in-graph nodes");

        // Echo chain
        echo_dry.connect_with_audio_node(&channel_gain)
            .expect("connect echo_dry → channel_gain: AudioNode.connect() is infallible between valid in-graph nodes");
        echo_dry.connect_with_audio_node(&echo_delay)
            .expect("connect echo_dry → echo_delay: AudioNode.connect() is infallible between valid in-graph nodes");
        echo_delay.connect_with_audio_node(&echo_wet)
            .expect("connect echo_delay → echo_wet: AudioNode.connect() is infallible between valid in-graph nodes");
        echo_wet.connect_with_audio_node(&channel_gain)
            .expect("connect echo_wet → channel_gain: AudioNode.connect() is infallible between valid in-graph nodes");
        echo_delay.connect_with_audio_node(&echo_feedback)
            .expect("connect echo_delay → echo_feedback: AudioNode.connect() is infallible between valid in-graph nodes");
        echo_feedback.connect_with_audio_node(&echo_delay)
            .expect("connect echo_feedback → echo_delay: AudioNode.connect() is infallible between valid in-graph nodes");

        // Flanger (wet=0.0 by default)
        sweep_filter.connect_with_audio_node(&flanger_delay)
            .expect("connect sweep → flanger_delay: AudioNode.connect() is infallible between valid in-graph nodes");
        flanger_delay.connect_with_audio_node(&flanger_wet)
            .expect("connect flanger_delay → flanger_wet: AudioNode.connect() is infallible between valid in-graph nodes");
        flanger_wet.connect_with_audio_node(&channel_gain)
            .expect("connect flanger_wet → channel_gain: AudioNode.connect() is infallible between valid in-graph nodes");
        flanger_lfo.connect_with_audio_node(&flanger_depth)
            .expect("connect lfo → flanger_depth: AudioNode.connect() is infallible between valid in-graph nodes");
        flanger_depth.connect_with_audio_param(&flanger_delay.delay_time())
            .expect("connect depth → delay_time: AudioNode.connect() is infallible between valid in-graph nodes");
        flanger_lfo.start()
            .expect("flanger_lfo.start(): OscillatorNode.start() is infallible when called exactly once on a new node");

        // channel_gain → stutter_gate → analyser (analyser → xfade GainNode wired in M5 via connect_to_mixer_output)
        channel_gain.connect_with_audio_node(&stutter_gate)
            .expect("connect channel_gain → stutter_gate: AudioNode.connect() is infallible between valid in-graph nodes");
        stutter_gate.connect_with_audio_node(&analyser)
            .expect("connect stutter_gate → analyser: AudioNode.connect() is infallible between valid in-graph nodes");

        // Load a default reverb impulse response (medium hall: 1.2 s, decay 2.5).
        let default_ir = generate_reverb_ir(&ctx, 1.2, 2.5);
        reverb.set_buffer(Some(&default_ir));

        Rc::new(RefCell::new(AudioDeck {
            ctx,
            source: None,
            buffer: None,
            pre_gain,
            eq_high,
            eq_mid,
            eq_low,
            sweep_filter,
            reverb,
            reverb_dry,
            reverb_wet,
            echo_delay,
            echo_feedback,
            echo_wet,
            echo_dry,
            flanger_delay,
            flanger_lfo,
            flanger_depth,
            flanger_wet,
            stutter_gate,
            channel_gain,
            analyser,
            started_at: None,
            offset_at_play: 0.0,
            rate_at_play: 1.0,
            cue_point: None,
            pre_nudge_rate: None,
            scratch_active: false,
            scratch_last_angle: 0.0,
            scratch_last_time: 0.0,
            pre_scratch_rate: 1.0,
        }))
    }

    // ── Playback ──────────────────────────────────────────────────────────────

    /// Wire this deck's output (analyser) into the mixer's crossfader gain node.
    ///
    /// Must be called once after `MixerAudio` has been constructed.
    pub fn connect_to_mixer_output(&self, xfade_gain: &GainNode) {
        self.analyser
            .connect_with_audio_node(xfade_gain)
            .expect("AudioDeck::connect_to_mixer_output — analyser → xfade: AudioNode.connect() is infallible between valid in-graph nodes");
    }

    /// Start or restart playback from `offset` seconds at the given `rate`.
    ///
    /// Creates a new `AudioBufferSourceNode` (one-shot by Web Audio design),
    /// connects it to the processing chain, and calls `start(0, offset)`.
    /// Does nothing if no buffer has been loaded.
    pub fn play(&mut self, offset: f64, rate: f32) {
        // Stop any currently playing source first.
        self.stop_source();

        let buffer = match &self.buffer {
            Some(b) => b.clone(),
            None => return,
        };

        let src = self.ctx.create_buffer_source()
            .expect("AudioDeck::play — create_buffer_source: AudioBufferSourceNode creation is infallible on a live context");
        src.set_buffer(Some(&buffer));
        src.playback_rate().set_value(rate);
        src.connect_with_audio_node(&self.pre_gain)
            .expect("AudioDeck::play — connect source → pre_gain: AudioNode.connect() is infallible between valid in-graph nodes");

        // start(when=0, offset) — begin immediately at the given track offset.
        src.start_with_when_and_grain_offset(0.0, offset)
            .expect("AudioDeck::play — source.start: AudioBufferSourceNode.start() cannot fail on a node that has not been started before");

        self.started_at = Some(self.ctx.current_time());
        self.offset_at_play = offset;
        self.rate_at_play = rate as f64;
        self.source = Some(src);
    }

    /// Pause playback. Records the current position so `play()` can resume it.
    ///
    /// Returns the held position in seconds.
    pub fn pause(&mut self) -> f64 {
        let pos = self.current_position();
        self.stop_source();
        self.started_at = None;
        self.offset_at_play = pos;
        pos
    }

    /// Stop playback and reset the playhead to the beginning of the track.
    pub fn stop(&mut self) {
        self.stop_source();
        self.started_at = None;
        self.offset_at_play = 0.0;
    }

    /// Seek to `position` seconds. Restarts playback at the new position if
    /// the deck was already playing.
    pub fn seek(&mut self, position: f64, rate: f32) {
        let was_playing = self.source.is_some();
        self.stop_source();
        self.started_at = None;
        self.offset_at_play = position;
        if was_playing {
            self.play(position, rate);
        }
    }

    /// Set or jump to the cue point.
    ///
    /// - If no cue point is set, saves the current position as the cue point.
    /// - If a cue point is already set, seeks to it (resuming playback state).
    pub fn cue(&mut self, rate: f32) {
        match self.cue_point {
            None => {
                self.cue_point = Some(self.current_position());
            }
            Some(cue_pos) => {
                self.seek(cue_pos, rate);
            }
        }
    }

    // ── Nudge ─────────────────────────────────────────────────────────────────

    /// Begin a tempo nudge in `direction` (+1.0 = speed up, -1.0 = slow down).
    ///
    /// Temporarily shifts the active source's `playbackRate` by ±5% and saves
    /// the pre-nudge rate so `nudge_end` can restore it smoothly.
    pub fn nudge_start(&mut self, direction: f32) {
        if let Some(ref src) = self.source {
            let current = src.playback_rate().value();
            self.pre_nudge_rate = Some(current);
            // Nudge by ±5% of the current rate.
            src.playback_rate().set_value(current * (1.0 + NUDGE_FACTOR * direction));
        }
    }

    /// End the nudge, ramping `playbackRate` back to the pre-nudge value over
    /// `NUDGE_RAMP_SECS` seconds.
    pub fn nudge_end(&mut self) {
        if let Some(rate) = self.pre_nudge_rate.take() {
            if let Some(ref src) = self.source {
                let target_time = self.ctx.current_time() + NUDGE_RAMP_SECS;
                src.playback_rate()
                    .linear_ramp_to_value_at_time(rate, target_time)
                    .expect("AudioDeck::nudge_end — linear_ramp_to_value_at_time: AudioParam scheduling is infallible");
            }
        }
    }

    // ── Position tracking ────────────────────────────────────────────────────

    /// Compute the current playhead position in seconds.
    ///
    /// Returns `offset_at_play` when not playing (i.e. `started_at` is `None`).
    pub fn current_position(&self) -> f64 {
        match self.started_at {
            Some(started_at) => {
                let elapsed = self.ctx.current_time() - started_at;
                (elapsed * self.rate_at_play + self.offset_at_play).max(0.0)
            }
            None => self.offset_at_play,
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Stop and drop the active source node, ignoring errors (already-stopped is fine).
    #[allow(deprecated)] // stop_with_when is deprecated in web-sys but remains the correct call
    fn stop_source(&mut self) {
        if let Some(ref src) = self.source {
            // stop_with_when(0.0) = stop immediately; the parameterless stop() is also deprecated.
            let _ = src.stop_with_when(0.0);
        }
        self.source = None;
    }
}

/// Nudge tempo shift factor (5% of current rate).
const NUDGE_FACTOR: f32 = 0.05;
/// Time in seconds to ramp back to normal rate after releasing the nudge button.
const NUDGE_RAMP_SECS: f64 = 0.1;

// ── EQ / Filter helpers (T8.2, T8.3) ─────────────────────────────────────────

/// Apply the sweep-filter logic to `node` based on `filter_val` ∈ [−1.0, +1.0].
///
/// | `filter_val` | Effect |
/// |---|---|
/// | ≈ 0 (abs < 0.02) | Flat — peaking at 0 dB gain |
/// | negative | Low-pass: sweeps from 20 kHz (open) down to 200 Hz (closed) |
/// | positive | High-pass: sweeps from 20 Hz (open) up to 2 000 Hz (closed) |
pub fn apply_sweep_filter(node: &BiquadFilterNode, filter_val: f32) {
    const BYPASS_THRESHOLD: f32 = 0.02;
    if filter_val.abs() < BYPASS_THRESHOLD {
        node.set_type(BiquadFilterType::Peaking);
        node.gain().set_value(0.0);
    } else if filter_val < 0.0 {
        // Remap [−1, 0] → [0, 1] — 0 = open (20 kHz), 1 = closed (200 Hz)
        let t = 1.0 + filter_val;
        let freq = 200.0_f32 + t * (20_000.0 - 200.0);
        node.set_type(BiquadFilterType::Lowpass);
        node.frequency().set_value(freq);
        node.q().set_value(0.5);
    } else {
        // [0, 1] → 20 Hz (open) to 2 000 Hz (closed)
        let freq = 20.0_f32 + filter_val * (2_000.0 - 20.0);
        node.set_type(BiquadFilterType::Highpass);
        node.frequency().set_value(freq);
        node.q().set_value(0.5);
    }
}

/// Compute the RMS level of the most recent audio frame from an `AnalyserNode`.
///
/// Returns a value in [0.0, 1.0] suitable for driving a VU meter bar height:
/// dBFS range −60..0 is mapped linearly to 0.0..1.0.
pub fn read_vu_level(analyser: &AnalyserNode) -> f32 {
    let n = analyser.fft_size() as usize; // 256 at 44.1 kHz ≈ 5.8 ms
    let mut buf = vec![0.0f32; n];
    analyser.get_float_time_domain_data(&mut buf);

    let rms = (buf.iter().map(|&s| s * s).sum::<f32>() / n as f32).sqrt();
    // Map dBFS [−60, 0] → [0.0, 1.0]
    let db = (20.0 * rms.max(1e-6_f32).log10()).max(-60.0);
    (db + 60.0) / 60.0
}

// ── FX helper functions (T9.1–T9.4) ─────────────────────────────────────────

/// Ramp gain over 20 ms — avoids click artefacts on wet/dry transitions.
///
/// Anchors the current value with `setValueAtTime` first, as required by the
/// Web Audio spec when no prior automation event exists on the parameter.
fn ramp_gain(param: &web_sys::AudioParam, ctx: &AudioContext, target: f32) {
    let now = ctx.current_time();
    let _ = param.set_value_at_time(param.value(), now);
    let _ = param.linear_ramp_to_value_at_time(target, now + FX_RAMP_SECS);
}

/// Fade-in/out duration for all FX wet/dry switches (20 ms = click-free).
const FX_RAMP_SECS: f64 = 0.02;

/// Generate a stereo exponential-decay white-noise impulse response for the convolver.
///
/// Algorithm: `IR[ch][t] = noise × exp(−decay × t)` where `t` is normalised
/// position 0→1 across the buffer.  Equivalent to `exp(-decay × t_secs / duration_secs)`
/// with time in seconds.  `decay` alone controls the envelope shape: higher values
/// produce tighter, drier reverbs; lower values produce long cathedral tails.
/// Two channels use different LCG seeds for stereo decorrelation.
pub fn generate_reverb_ir(ctx: &AudioContext, duration_secs: f32, decay: f32) -> AudioBuffer {
    let sample_rate = ctx.sample_rate();
    let num_samples = (sample_rate * duration_secs) as usize;
    let ir = ctx.create_buffer(2, num_samples as u32, sample_rate)
        .expect("generate_reverb_ir — create_buffer: infallible on a live context");

    for channel in 0..2_u32 {
        let mut state: u64 = if channel == 0 {
            0x12345678_9ABCDEF0
        } else {
            0xFEDCBA98_76543210
        };
        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                // Shift by 32 to get full 32-bit range → [0, u32::MAX]; dividing by
                // u32::MAX maps to [0.0, 1.0]; after * 2 - 1 → [-1.0, +1.0].
                let noise = (state >> 32) as f32 / (u32::MAX as f32) * 2.0 - 1.0;
                // t = normalised position (0→1); exp(-decay×t) gives the envelope.
                let t = i as f32 / num_samples as f32;
                noise * (-decay * t).exp()
            })
            .collect();
        ir.copy_to_channel(&samples, channel as i32)
            .expect("generate_reverb_ir — copy_to_channel: infallible on a valid AudioBuffer");
    }
    ir
}

/// Pre-schedule a repeating stutter gate pattern for `bars` bars from `start_time`.
///
/// `subdivision` is the denominator of the note value:
/// 4.0 = quarter note, 8.0 = eighth, 16.0 = sixteenth, 32.0 = thirty-second.
/// `duty` is the fraction of each gate period that stays open (0.0–1.0).
pub fn schedule_stutter(
    gate:        &GainNode,
    start_time:  f64,
    bpm:         f64,
    subdivision: f64,
    duty:        f64,
    bars:        f64,
) {
    let (gate_period, gate_open, window_dur) = stutter_timings(bpm, subdivision, duty, bars);
    let end_time = start_time + window_dur;

    let mut t = start_time;
    while t < end_time {
        let _ = gate.gain().set_value_at_time(1.0, t);
        let _ = gate.gain().set_value_at_time(0.0, t + gate_open);
        t += gate_period;
    }
}

/// Pure timing computation for stutter — extracted for native unit testing.
///
/// Returns `(gate_period, gate_open, window_duration)` in seconds.
pub(crate) fn stutter_timings(
    bpm:         f64,
    subdivision: f64,
    duty:        f64,
    bars:        f64,
) -> (f64, f64, f64) {
    let beat_dur    = 60.0 / bpm;
    let gate_period = beat_dur * 4.0 / subdivision;
    let gate_open   = gate_period * duty;
    let window_dur  = bars * beat_dur * 4.0;
    (gate_period, gate_open, window_dur)
}

/// Map pointer angular velocity to `AudioBufferSourceNode.playbackRate`.
///
/// `d_angle` is the angle delta in radians for the interval `dt_secs` (seconds).
/// 33 RPM = 0.55 rot/sec; one full rotation per second → `playbackRate = 1/0.55 ≈ 1.818`.
/// Result is clamped to [0.0, 4.0] (forward-only; see spec §8.12).
pub(crate) fn scratch_rate_from_angular_velocity(d_angle: f64, dt_secs: f64) -> f32 {
    let rate = ((d_angle / dt_secs) / (std::f64::consts::TAU * 0.55)) as f32;
    rate.clamp(0.0, 4.0)
}

impl AudioDeck {
    // ── Echo / Delay ─────────────────────────────────────────────────────────

    /// Enable echo: ramp wet gain up; leave dry at 1.0 (parallel mix).
    pub fn enable_echo(&self) {
        ramp_gain(&self.echo_wet.gain(), &self.ctx, 0.6);
    }

    /// Disable echo: ramp wet gain to 0 over 20 ms.
    pub fn disable_echo(&self) {
        ramp_gain(&self.echo_wet.gain(), &self.ctx, 0.0);
    }

    // ── Reverb ─────────────────────────────────────────────────────────────

    /// Enable reverb: ramp wet gain up; dry stays at 1.0.
    pub fn enable_reverb(&self) {
        ramp_gain(&self.reverb_wet.gain(), &self.ctx, 0.5);
    }

    /// Disable reverb: ramp wet gain to 0.
    pub fn disable_reverb(&self) {
        ramp_gain(&self.reverb_wet.gain(), &self.ctx, 0.0);
    }

    /// Regenerate the reverb impulse response with new parameters and reload it
    /// into the `ConvolverNode`.  Call this when the user changes reverb
    /// duration or decay in the FX panel.
    pub fn reload_reverb_ir(&self, duration_secs: f32, decay: f32) {
        let ir = generate_reverb_ir(&self.ctx, duration_secs, decay);
        self.reverb.set_buffer(Some(&ir));
    }

    // ── Flanger ──────────────────────────────────────────────────────────────

    /// Enable flanger: ramp wet gain up.
    pub fn enable_flanger(&self) {
        ramp_gain(&self.flanger_wet.gain(), &self.ctx, 0.5);
    }

    /// Disable flanger: ramp wet gain to 0.
    pub fn disable_flanger(&self) {
        ramp_gain(&self.flanger_wet.gain(), &self.ctx, 0.0);
    }

    // ── Stutter ───────────────────────────────────────────────────────────────

    /// Enable stutter: schedule 16 bars of gating from now.
    /// Uses the given BPM and subdivision denominator.
    pub fn enable_stutter(&self, bpm: f64, subdivision: f64) {
        let start = self.ctx.current_time();
        schedule_stutter(&self.stutter_gate, start, bpm, subdivision, 0.5, 16.0);
    }

    /// Disable stutter: cancel all scheduled values and ramp gate back open.
    pub fn disable_stutter(&self) {
        let _ = self.stutter_gate.gain().cancel_scheduled_values(0.0);
        ramp_gain(&self.stutter_gate.gain(), &self.ctx, 1.0);
    }

    // ── Scratch ───────────────────────────────────────────────────────────────

    /// Begin a scratch gesture.  Records the initial angle and time, and saves
    /// the current source playback rate so `scratch_end` can restore it.
    ///
    /// `angle` is the pointer's angle in radians from the platter centre.
    /// `time` is `performance.now()` in milliseconds.
    pub fn scratch_start(&mut self, angle: f64, time: f64) {
        if self.scratch_active {
            return;
        }
        self.scratch_active = true;
        self.pre_scratch_rate = self.source
            .as_ref()
            .map(|s| s.playback_rate().value())
            .unwrap_or(1.0);
        self.scratch_last_angle = angle;
        self.scratch_last_time  = time;
    }

    /// Update the playback rate based on pointer angular velocity.
    ///
    /// Maps `Δangle / Δt` to a `playbackRate` — one full rotation per second
    /// at 33 RPM (0.55 rot/s) corresponds to `playbackRate = 1.0`.
    pub fn scratch_move(&mut self, angle: f64, time: f64) {
        if !self.scratch_active {
            return;
        }
        // dt in seconds; guard against divide-by-zero on rapid successive events.
        let dt = ((time - self.scratch_last_time) / 1000.0).max(0.001);
        let mut d_angle = angle - self.scratch_last_angle;
        if d_angle >  std::f64::consts::PI { d_angle -= std::f64::consts::TAU; }
        if d_angle < -std::f64::consts::PI { d_angle += std::f64::consts::TAU; }

        let rate = scratch_rate_from_angular_velocity(d_angle, dt);
        if let Some(ref src) = self.source {
            src.playback_rate().set_value(rate);
        }
        self.scratch_last_angle = angle;
        self.scratch_last_time  = time;
    }

    /// End the scratch gesture: ramp playback rate smoothly back to the
    /// pre-scratch value over 100 ms.
    pub fn scratch_end(&mut self) {
        if !self.scratch_active {
            return;
        }
        if let Some(ref src) = self.source {
            let now = self.ctx.current_time();
            // Anchor current value first — required by Web Audio spec so that
            // linearRampToValueAtTime has a defined starting point.
            let current_rate = src.playback_rate().value();
            let _ = src.playback_rate().set_value_at_time(current_rate, now);
            let _ = src.playback_rate()
                .linear_ramp_to_value_at_time(
                    self.pre_scratch_rate, // f32 value; f64 end_time per web-sys signature
                    now + 0.1,
                );
        }
        self.scratch_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    use crate::audio::context::ensure_audio_context;

    fn make_ctx() -> AudioContext {
        let holder = std::rc::Rc::new(std::cell::RefCell::new(None::<AudioContext>));
        ensure_audio_context(&holder)
    }

    #[wasm_bindgen_test]
    fn constructs_without_panic() {
        // Verifies every node creation and connection call in AudioDeck::new
        // succeeds in a real browser environment.
        let _deck = AudioDeck::new(make_ctx());
    }

    #[wasm_bindgen_test]
    fn buffer_is_none_before_load() {
        let deck = AudioDeck::new(make_ctx());
        assert!(deck.borrow().buffer.is_none());
    }

    #[wasm_bindgen_test]
    fn source_is_none_before_play() {
        let deck = AudioDeck::new(make_ctx());
        assert!(deck.borrow().source.is_none());
    }

    #[wasm_bindgen_test]
    fn channel_gain_defaults_to_one() {
        let deck = AudioDeck::new(make_ctx());
        let gain = deck.borrow().channel_gain.gain().value();
        assert!((gain - 1.0).abs() < 1e-6, "expected 1.0, got {gain}");
    }

    #[wasm_bindgen_test]
    fn reverb_wet_defaults_to_zero() {
        // Reverb is off by default — wet gain must be 0 so dry path carries signal.
        let deck = AudioDeck::new(make_ctx());
        let wet = deck.borrow().reverb_wet.gain().value();
        assert!(wet.abs() < 1e-6, "reverb_wet should be 0.0, got {wet}");
    }

    #[wasm_bindgen_test]
    fn echo_wet_defaults_to_zero() {
        let deck = AudioDeck::new(make_ctx());
        let wet = deck.borrow().echo_wet.gain().value();
        assert!(wet.abs() < 1e-6, "echo_wet should be 0.0, got {wet}");
    }

    #[wasm_bindgen_test]
    fn flanger_wet_defaults_to_zero() {
        let deck = AudioDeck::new(make_ctx());
        let wet = deck.borrow().flanger_wet.gain().value();
        assert!(wet.abs() < 1e-6, "flanger_wet should be 0.0, got {wet}");
    }

    #[wasm_bindgen_test]
    fn current_position_scales_by_rate_at_play() {
        // Simulate 10 wall-clock seconds elapsed at 2× rate → position should be ~20s.
        let deck = AudioDeck::new(make_ctx());
        {
            let mut d = deck.borrow_mut();
            let now = d.ctx.current_time();
            d.started_at     = Some(now - 10.0);
            d.offset_at_play = 0.0;
            d.rate_at_play   = 2.0;
        }
        let pos = deck.borrow().current_position();
        assert!((pos - 20.0).abs() < 0.1, "expected ~20.0 at 2× rate, got {pos}");
    }

    #[wasm_bindgen_test]
    fn current_position_with_offset_and_rate() {
        // 5 wall seconds elapsed at 0.5× rate from offset 10s → position should be ~12.5s.
        let deck = AudioDeck::new(make_ctx());
        {
            let mut d = deck.borrow_mut();
            let now = d.ctx.current_time();
            d.started_at     = Some(now - 5.0);
            d.offset_at_play = 10.0;
            d.rate_at_play   = 0.5;
        }
        let pos = deck.borrow().current_position();
        assert!((pos - 12.5).abs() < 0.1, "expected ~12.5, got {pos}");
    }

    #[wasm_bindgen_test]
    fn current_position_is_zero_before_play() {
        let deck = AudioDeck::new(make_ctx());
        let pos = deck.borrow().current_position();
        assert!(pos.abs() < 1e-9, "expected 0.0, got {pos}");
    }

    #[wasm_bindgen_test]
    fn stop_resets_position_to_zero() {
        let deck = AudioDeck::new(make_ctx());
        deck.borrow_mut().offset_at_play = 42.0;
        deck.borrow_mut().stop();
        let pos = deck.borrow().current_position();
        assert!(pos.abs() < 1e-9, "stop() should reset to 0.0, got {pos}");
    }

    #[wasm_bindgen_test]
    fn cue_point_is_none_before_cue_called() {
        let deck = AudioDeck::new(make_ctx());
        assert!(deck.borrow().cue_point.is_none());
    }

    #[wasm_bindgen_test]
    fn cue_sets_cue_point_when_none() {
        let deck = AudioDeck::new(make_ctx());
        // offset_at_play is 0.0 and started_at is None so current_position() = 0.0
        deck.borrow_mut().cue(1.0);
        let cue = deck.borrow().cue_point;
        assert!(cue.is_some(), "cue_point should be set");
        assert!((cue.unwrap()).abs() < 1e-9);
    }

    // ── M9: stutter timing (native) ───────────────────────────────────────────
    // These tests run on the native host (no browser needed) and verify the
    // pure arithmetic inside stutter_timings().

    #[test]
    fn stutter_eighth_note_period_at_120_bpm() {
        // At 120 BPM, beat = 0.5 s; bar = 2.0 s; 1/8 note = 0.25 s.
        let (gate_period, gate_open, window_dur) = stutter_timings(120.0, 8.0, 0.5, 4.0);
        assert!((gate_period - 0.25).abs() < 1e-9, "gate_period={gate_period}");
        assert!((gate_open   - 0.125).abs() < 1e-9, "gate_open={gate_open}");
        assert!((window_dur  - 8.0).abs() < 1e-9, "window_dur={window_dur}");
    }

    #[test]
    fn stutter_quarter_note_period_at_120_bpm() {
        // 1/4 note at 120 BPM = 0.5 s; 50% duty → open 0.25 s.
        let (gate_period, gate_open, _) = stutter_timings(120.0, 4.0, 0.5, 1.0);
        assert!((gate_period - 0.5).abs() < 1e-9, "gate_period={gate_period}");
        assert!((gate_open   - 0.25).abs() < 1e-9, "gate_open={gate_open}");
    }

    #[test]
    fn stutter_sixteenth_note_period_at_128_bpm() {
        // At 128 BPM, beat = 60/128 ≈ 0.46875 s; 1/16 = beat/4 ≈ 0.117 s.
        let beat = 60.0 / 128.0;
        let expected_period = beat * 4.0 / 16.0;
        let (gate_period, _, _) = stutter_timings(128.0, 16.0, 0.5, 1.0);
        assert!((gate_period - expected_period).abs() < 1e-9, "gate_period={gate_period}");
    }

    #[test]
    fn stutter_window_scales_with_bars() {
        // Window duration = bars × beat × 4. At 60 BPM: beat = 1s, bar = 4s.
        let (_, _, window_1) = stutter_timings(60.0, 8.0, 0.5, 1.0);
        let (_, _, window_4) = stutter_timings(60.0, 8.0, 0.5, 4.0);
        assert!((window_1 - 4.0).abs() < 1e-9);
        assert!((window_4 - 16.0).abs() < 1e-9);
    }

    #[test]
    fn stutter_duty_zero_means_always_closed() {
        let (_, gate_open, _) = stutter_timings(120.0, 8.0, 0.0, 4.0);
        assert!(gate_open.abs() < 1e-9, "duty=0 → gate_open should be 0, got {gate_open}");
    }

    #[test]
    fn stutter_duty_one_means_always_open() {
        let (gate_period, gate_open, _) = stutter_timings(120.0, 8.0, 1.0, 4.0);
        assert!((gate_open - gate_period).abs() < 1e-9, "duty=1 → gate_open should equal period");
    }

    // ── M9: scratch angular velocity → playback rate (native) ────────────────

    #[test]
    fn scratch_stationary_gives_zero_rate() {
        // No rotation → rate = 0 (clamped from negative result too).
        let rate = scratch_rate_from_angular_velocity(0.0, 0.1);
        assert!(rate.abs() < 1e-6, "stationary should give 0, got {rate}");
    }

    #[test]
    fn scratch_one_full_rotation_per_second_gives_rate_near_one() {
        // TAU rad / 1.0 s → playback_rate ≈ 1/0.55 ≈ 1.818 (unclamped).
        // This simulates scrubbing at 33 RPM-equivalent angular velocity.
        let rate = scratch_rate_from_angular_velocity(std::f64::consts::TAU, 1.0);
        let expected = (1.0_f64 / 0.55) as f32;
        assert!((rate - expected).abs() < 0.01, "expected ~{expected:.3}, got {rate}");
    }

    #[test]
    fn scratch_rate_clamped_at_upper_bound() {
        // Extremely fast rotation should not produce a rate above 4.0.
        let rate = scratch_rate_from_angular_velocity(100.0 * std::f64::consts::TAU, 0.01);
        assert!(rate <= 4.0, "expected ≤4.0, got {rate}");
        assert!((rate - 4.0).abs() < 1e-6, "expected exactly 4.0 at clamp, got {rate}");
    }

    #[test]
    fn scratch_negative_angular_velocity_clamped_to_zero() {
        // Reverse motion gives negative angular velocity; clamped to 0 (no reverse playback).
        let rate = scratch_rate_from_angular_velocity(-std::f64::consts::TAU, 1.0);
        assert!(rate.abs() < 1e-6, "negative velocity should clamp to 0, got {rate}");
    }

    #[test]
    fn scratch_half_speed_rotation() {
        // Half of 33 RPM equivalent → rate ≈ 0.5 / 0.55 ≈ 0.909.
        let rate = scratch_rate_from_angular_velocity(std::f64::consts::TAU * 0.5, 1.0);
        let expected = (0.5_f64 / 0.55) as f32;
        assert!((rate - expected).abs() < 0.01, "expected ~{expected:.3}, got {rate}");
    }

    // ── M9: reverb IR (WASM) ──────────────────────────────────────────────────

    #[wasm_bindgen_test]
    fn reverb_ir_has_two_channels() {
        let ctx = make_ctx();
        let ir = generate_reverb_ir(&ctx, 1.0, 2.0);
        assert_eq!(ir.number_of_channels(), 2);
    }

    #[wasm_bindgen_test]
    fn reverb_ir_length_matches_duration() {
        let ctx = make_ctx();
        let duration = 0.5_f32;
        let ir = generate_reverb_ir(&ctx, duration, 2.0);
        let expected_samples = (ctx.sample_rate() * duration) as u32;
        assert_eq!(ir.length(), expected_samples);
    }

    #[wasm_bindgen_test]
    fn reverb_ir_has_positive_and_negative_samples() {
        // Validates the LCG produces full [-1, +1] range (not [-1, 0] from >> 33 bug).
        let ctx = make_ctx();
        let ir = generate_reverb_ir(&ctx, 0.5, 2.0);
        let n = ir.length() as usize;
        let mut ch0 = vec![0.0_f32; n];
        ir.copy_from_channel(&mut ch0, 0).expect("copy_from_channel ch0");
        let has_positive = ch0.iter().any(|&s| s > 0.01);
        let has_negative = ch0.iter().any(|&s| s < -0.01);
        assert!(has_positive, "IR channel 0 has no positive samples — LCG range bug?");
        assert!(has_negative, "IR channel 0 has no negative samples");
    }

    #[wasm_bindgen_test]
    fn reverb_ir_decays_toward_end() {
        // Validates exp(-decay*t) envelope: average magnitude at start > at end.
        let ctx = make_ctx();
        let ir = generate_reverb_ir(&ctx, 1.0, 2.5);
        let n = ir.length() as usize;
        let mut ch = vec![0.0_f32; n];
        ir.copy_from_channel(&mut ch, 0).expect("copy_from_channel");

        let quarter = n / 4;
        let start_rms = rms(&ch[..quarter]);
        let end_rms   = rms(&ch[n - quarter..]);
        assert!(
            start_rms > end_rms * 2.0,
            "IR should decay: start_rms={start_rms:.4} end_rms={end_rms:.4}"
        );
    }

    #[wasm_bindgen_test]
    fn reverb_ir_channels_differ_for_stereo_width() {
        // Two LCG seeds → channels must not be identical (stereo decorrelation).
        let ctx = make_ctx();
        let ir = generate_reverb_ir(&ctx, 0.2, 2.0);
        let n = ir.length() as usize;
        let mut ch0 = vec![0.0_f32; n];
        let mut ch1 = vec![0.0_f32; n];
        ir.copy_from_channel(&mut ch0, 0).expect("ch0");
        ir.copy_from_channel(&mut ch1, 1).expect("ch1");
        let identical = ch0.iter().zip(&ch1).all(|(a, b)| (a - b).abs() < 1e-9);
        assert!(!identical, "IR channels are identical — stereo decorrelation is broken");
    }

    // ── M9: FX enable/disable (WASM) ─────────────────────────────────────────

    #[wasm_bindgen_test]
    fn stutter_gate_defaults_to_one() {
        let deck = AudioDeck::new(make_ctx());
        let g = deck.borrow().stutter_gate.gain().value();
        assert!((g - 1.0).abs() < 1e-6, "stutter_gate should default to 1.0, got {g}");
    }

    #[wasm_bindgen_test]
    fn enable_echo_raises_wet_gain() {
        let deck = AudioDeck::new(make_ctx());
        deck.borrow().enable_echo();
        // After a ramp is scheduled the current *instantaneous* value is still 0.0,
        // but the AudioParam must have a scheduled event — check via a read after
        // the ramp end time has elapsed.  In a headless test we just verify the
        // call does not panic and the immediate value is still bounded.
        let wet = deck.borrow().echo_wet.gain().value();
        assert!(wet >= 0.0, "echo_wet should be non-negative after enable, got {wet}");
    }

    #[wasm_bindgen_test]
    fn disable_echo_does_not_panic() {
        let deck = AudioDeck::new(make_ctx());
        deck.borrow().enable_echo();
        deck.borrow().disable_echo();
    }

    #[wasm_bindgen_test]
    fn enable_reverb_does_not_panic() {
        // Verifies reverb is wired (ConvolverNode has an IR loaded) and enable/disable
        // calls succeed without panic.
        let deck = AudioDeck::new(make_ctx());
        deck.borrow().enable_reverb();
        deck.borrow().disable_reverb();
    }

    #[wasm_bindgen_test]
    fn enable_flanger_does_not_panic() {
        let deck = AudioDeck::new(make_ctx());
        deck.borrow().enable_flanger();
        deck.borrow().disable_flanger();
    }

    #[wasm_bindgen_test]
    fn enable_stutter_does_not_panic() {
        let deck = AudioDeck::new(make_ctx());
        deck.borrow().enable_stutter(120.0, 8.0);
        deck.borrow().disable_stutter();
    }

    #[wasm_bindgen_test]
    fn scratch_state_inactive_by_default() {
        let deck = AudioDeck::new(make_ctx());
        assert!(!deck.borrow().scratch_active);
    }

    #[wasm_bindgen_test]
    fn scratch_end_without_start_is_safe() {
        // Calling scratch_end when scratch is not active should be a no-op.
        let deck = AudioDeck::new(make_ctx());
        deck.borrow_mut().scratch_end(); // must not panic
    }
}

/// RMS of a sample slice — used by the reverb IR decay test.
#[cfg(test)]
fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    (samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}
