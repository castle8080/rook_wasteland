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

        let channel_gain = ctx.create_gain()
            .expect("create_gain channel: AudioContext node creation is infallible on a live context");
        channel_gain.gain().set_value(1.0);

        let analyser = ctx.create_analyser()
            .expect("create_analyser: AudioContext node creation is infallible on a live context");
        analyser.set_fft_size(256);

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

        // channel_gain → analyser (analyser → xfade GainNode wired in M5 via connect_to_mixer_output)
        channel_gain.connect_with_audio_node(&analyser)
            .expect("connect channel_gain → analyser: AudioNode.connect() is infallible between valid in-graph nodes");

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
            channel_gain,
            analyser,
            started_at: None,
            offset_at_play: 0.0,
            rate_at_play: 1.0,
            cue_point: None,
            pre_nudge_rate: None,
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
}
