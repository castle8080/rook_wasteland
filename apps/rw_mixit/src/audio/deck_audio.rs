use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{
    AudioContext, AudioBuffer, AudioBufferSourceNode,
    GainNode, BiquadFilterNode, BiquadFilterType,
    ConvolverNode, DelayNode, AnalyserNode, OscillatorNode,
};

#[allow(dead_code)]
pub struct AudioDeck {
    pub ctx:            AudioContext,
    pub source:         Option<AudioBufferSourceNode>,
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
    pub started_at:     Option<f64>,
    pub offset_at_play: f64,
}

impl AudioDeck {
    pub fn new(ctx: AudioContext) -> Rc<RefCell<AudioDeck>> {
        let pre_gain = ctx.create_gain().expect("create_gain pre_gain");
        pre_gain.gain().set_value(1.0);

        let eq_high = ctx.create_biquad_filter().expect("create_biquad_filter eq_high");
        eq_high.set_type(BiquadFilterType::Highshelf);
        eq_high.frequency().set_value(8000.0);
        eq_high.gain().set_value(0.0);

        let eq_mid = ctx.create_biquad_filter().expect("create_biquad_filter eq_mid");
        eq_mid.set_type(BiquadFilterType::Peaking);
        eq_mid.frequency().set_value(1000.0);
        eq_mid.q().set_value(0.7);
        eq_mid.gain().set_value(0.0);

        let eq_low = ctx.create_biquad_filter().expect("create_biquad_filter eq_low");
        eq_low.set_type(BiquadFilterType::Lowshelf);
        eq_low.frequency().set_value(200.0);
        eq_low.gain().set_value(0.0);

        let sweep_filter = ctx.create_biquad_filter().expect("create_biquad_filter sweep");
        sweep_filter.set_type(BiquadFilterType::Peaking);
        sweep_filter.gain().set_value(0.0);

        let reverb = ctx.create_convolver().expect("create_convolver");
        let reverb_dry = ctx.create_gain().expect("create_gain reverb_dry");
        reverb_dry.gain().set_value(1.0);
        let reverb_wet = ctx.create_gain().expect("create_gain reverb_wet");
        reverb_wet.gain().set_value(0.0);

        let echo_delay = ctx.create_delay_with_max_delay_time(2.0).expect("create_delay echo");
        echo_delay.delay_time().set_value(0.3);
        let echo_feedback = ctx.create_gain().expect("create_gain echo_feedback");
        echo_feedback.gain().set_value(0.0);
        let echo_wet = ctx.create_gain().expect("create_gain echo_wet");
        echo_wet.gain().set_value(0.0);
        let echo_dry = ctx.create_gain().expect("create_gain echo_dry");
        echo_dry.gain().set_value(1.0);

        let flanger_delay = ctx.create_delay_with_max_delay_time(0.02).expect("create_delay flanger");
        flanger_delay.delay_time().set_value(0.005);
        let flanger_lfo = ctx.create_oscillator().expect("create_oscillator");
        flanger_lfo.frequency().set_value(0.5);
        let flanger_depth = ctx.create_gain().expect("create_gain flanger_depth");
        flanger_depth.gain().set_value(0.003);
        let flanger_wet = ctx.create_gain().expect("create_gain flanger_wet");
        flanger_wet.gain().set_value(0.0);

        let channel_gain = ctx.create_gain().expect("create_gain channel");
        channel_gain.gain().set_value(1.0);

        let analyser = ctx.create_analyser().expect("create_analyser");
        analyser.set_fft_size(256);

        // Wire: pre_gain → eq_high → eq_mid → eq_low → sweep_filter
        pre_gain.connect_with_audio_node(&eq_high).expect("connect pre_gain → eq_high");
        eq_high.connect_with_audio_node(&eq_mid).expect("connect eq_high → eq_mid");
        eq_mid.connect_with_audio_node(&eq_low).expect("connect eq_mid → eq_low");
        eq_low.connect_with_audio_node(&sweep_filter).expect("connect eq_low → sweep_filter");

        // Reverb dry/wet bypass
        sweep_filter.connect_with_audio_node(&reverb_dry).expect("connect sweep → reverb_dry");
        sweep_filter.connect_with_audio_node(&reverb).expect("connect sweep → reverb");
        reverb.connect_with_audio_node(&reverb_wet).expect("connect reverb → reverb_wet");
        reverb_dry.connect_with_audio_node(&echo_dry).expect("connect reverb_dry → echo_dry");
        reverb_wet.connect_with_audio_node(&echo_dry).expect("connect reverb_wet → echo_dry");

        // Echo chain
        echo_dry.connect_with_audio_node(&channel_gain).expect("connect echo_dry → channel_gain");
        echo_dry.connect_with_audio_node(&echo_delay).expect("connect echo_dry → echo_delay");
        echo_delay.connect_with_audio_node(&echo_wet).expect("connect echo_delay → echo_wet");
        echo_wet.connect_with_audio_node(&channel_gain).expect("connect echo_wet → channel_gain");
        echo_delay.connect_with_audio_node(&echo_feedback).expect("connect echo_delay → echo_feedback");
        echo_feedback.connect_with_audio_node(&echo_delay).expect("connect echo_feedback → echo_delay");

        // Flanger (wet=0.0 by default)
        sweep_filter.connect_with_audio_node(&flanger_delay).expect("connect sweep → flanger_delay");
        flanger_delay.connect_with_audio_node(&flanger_wet).expect("connect flanger_delay → flanger_wet");
        flanger_wet.connect_with_audio_node(&channel_gain).expect("connect flanger_wet → channel_gain");
        flanger_lfo.connect_with_audio_node(&flanger_depth).expect("connect lfo → flanger_depth");
        flanger_depth.connect_with_audio_param(&flanger_delay.delay_time()).expect("connect depth → delay_time");
        flanger_lfo.start().expect("flanger_lfo.start");

        // channel_gain → analyser (analyser → destination wired in M5)
        channel_gain.connect_with_audio_node(&analyser).expect("connect channel_gain → analyser");

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
        }))
    }
}
