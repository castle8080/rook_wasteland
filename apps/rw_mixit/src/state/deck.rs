use leptos::prelude::*;

/// Valid waveform zoom levels — powers of two in [1, 8].
///
/// Enforces at the type level that only 1×, 2×, 4×, and 8× are representable,
/// eliminating the need for runtime range checks on the zoom value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZoomLevel { #[default] X1 = 1, X2 = 2, X4 = 4, X8 = 8 }

impl ZoomLevel {
    /// Step one level in (towards 8×). Returns current level if already at max.
    pub fn zoom_in(self) -> Self {
        match self {
            ZoomLevel::X1 => ZoomLevel::X2,
            ZoomLevel::X2 => ZoomLevel::X4,
            ZoomLevel::X4 => ZoomLevel::X8,
            ZoomLevel::X8 => ZoomLevel::X8,
        }
    }

    /// Step one level out (towards 1×). Returns current level if already at min.
    pub fn zoom_out(self) -> Self {
        match self {
            ZoomLevel::X1 => ZoomLevel::X1,
            ZoomLevel::X2 => ZoomLevel::X1,
            ZoomLevel::X4 => ZoomLevel::X2,
            ZoomLevel::X8 => ZoomLevel::X4,
        }
    }

    /// The numeric zoom factor (1, 2, 4, or 8).
    pub fn factor(self) -> u8 { self as u8 }
}

impl std::fmt::Display for ZoomLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.factor())
    }
}

// Leptos RwSignal fields are accessed via signal getters inside view! closures
// and Effects.  rustc's dead-code pass does not trace through the reactive macro
// boundary, so it incorrectly flags every field as unused.  This allow is a
// false-positive suppression — all fields ARE used in components and the rAF loop.
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
    pub fx_echo:              RwSignal<bool>,
    pub fx_echo_time:         RwSignal<f32>,
    pub fx_echo_feedback_val: RwSignal<f32>,
    pub fx_reverb:            RwSignal<bool>,
    pub fx_reverb_duration:   RwSignal<f32>,
    pub fx_reverb_decay:      RwSignal<f32>,
    pub fx_flanger:           RwSignal<bool>,
    pub fx_flanger_rate:      RwSignal<f32>,
    pub fx_flanger_depth:     RwSignal<f32>,
    pub fx_stutter:           RwSignal<bool>,
    /// Stutter subdivision denominator: 4.0=quarter, 8.0=eighth, 16.0=sixteenth, 32.0=thirty-second.
    pub fx_stutter_subdiv:    RwSignal<f32>,
    pub fx_scratch:           RwSignal<bool>,
    pub vu_level:       RwSignal<f32>,
    pub waveform_peaks: RwSignal<Option<Vec<f32>>>,
    /// Waveform zoom level — only valid powers of two (1, 2, 4, 8). Controls the
    /// visible time window around the playhead.
    pub zoom_level:     RwSignal<ZoomLevel>,
    /// Set when a file load or decode operation fails; cleared on each new load
    /// attempt.  Displayed as an inline error message on the deck.
    pub load_error:     RwSignal<Option<String>>,
}

impl Default for DeckState {
    fn default() -> Self {
        Self::new()
    }
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
            fx_echo:              RwSignal::new(false),
            fx_echo_time:         RwSignal::new(0.3f32),
            fx_echo_feedback_val: RwSignal::new(0.4f32),
            fx_reverb:            RwSignal::new(false),
            fx_reverb_duration:   RwSignal::new(1.2f32),
            fx_reverb_decay:      RwSignal::new(2.5f32),
            fx_flanger:           RwSignal::new(false),
            fx_flanger_rate:      RwSignal::new(0.5f32),
            fx_flanger_depth:     RwSignal::new(0.003f32),
            fx_stutter:           RwSignal::new(false),
            fx_stutter_subdiv:    RwSignal::new(8.0f32),
            fx_scratch:           RwSignal::new(false),
            vu_level:       RwSignal::new(0.0f32),
            waveform_peaks: RwSignal::new(None),
            zoom_level:     RwSignal::new(ZoomLevel::X1),
            load_error:     RwSignal::new(None),
        }
    }
}
