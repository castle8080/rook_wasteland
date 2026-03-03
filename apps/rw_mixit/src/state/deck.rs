use leptos::prelude::*;

#[allow(dead_code)]
#[derive(Clone)]
pub struct DeckState {
    pub is_playing:     RwSignal<bool>,
    pub playback_rate:  RwSignal<f64>,
    pub volume:         RwSignal<f32>,
    pub track_name:     RwSignal<Option<String>>,
    pub duration_secs:  RwSignal<f64>,
    pub current_secs:   RwSignal<f64>,
    pub loop_active:    RwSignal<bool>,
    pub loop_in:        RwSignal<f64>,
    pub loop_out:       RwSignal<f64>,
    pub hot_cues:       RwSignal<[Option<f64>; 4]>,
    pub eq_high:        RwSignal<f32>,
    pub eq_mid:         RwSignal<f32>,
    pub eq_low:         RwSignal<f32>,
    pub filter_val:     RwSignal<f32>,
    pub fx_echo:        RwSignal<bool>,
    pub fx_reverb:      RwSignal<bool>,
    pub fx_flanger:     RwSignal<bool>,
    pub fx_stutter:     RwSignal<bool>,
    pub fx_scratch:     RwSignal<bool>,
    pub vu_level:       RwSignal<f32>,
    pub waveform_peaks: RwSignal<Option<Vec<f32>>>,
}

impl DeckState {
    pub fn new() -> Self {
        Self {
            is_playing:     RwSignal::new(false),
            playback_rate:  RwSignal::new(1.0),
            volume:         RwSignal::new(1.0f32),
            track_name:     RwSignal::new(None),
            duration_secs:  RwSignal::new(0.0),
            current_secs:   RwSignal::new(0.0),
            loop_active:    RwSignal::new(false),
            loop_in:        RwSignal::new(0.0),
            loop_out:       RwSignal::new(0.0),
            hot_cues:       RwSignal::new([None; 4]),
            eq_high:        RwSignal::new(0.0f32),
            eq_mid:         RwSignal::new(0.0f32),
            eq_low:         RwSignal::new(0.0f32),
            filter_val:     RwSignal::new(0.0f32),
            fx_echo:        RwSignal::new(false),
            fx_reverb:      RwSignal::new(false),
            fx_flanger:     RwSignal::new(false),
            fx_stutter:     RwSignal::new(false),
            fx_scratch:     RwSignal::new(false),
            vu_level:       RwSignal::new(0.0f32),
            waveform_peaks: RwSignal::new(None),
        }
    }
}
