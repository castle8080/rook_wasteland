use leptos::prelude::*;

// Leptos RwSignal fields and enum variants are accessed via reactive closures
// and pattern matches inside view! and Effect macros.  rustc's dead-code pass
// does not trace through the macro boundary, producing false-positive warnings.
// These allow attributes are correct — all items ARE used in the app.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DeckId { A, B }

// Same false-positive reason as DeckId above.
#[allow(dead_code)]
#[derive(Clone)]
pub struct MixerState {
    pub crossfader:       RwSignal<f32>,
    pub master_volume:    RwSignal<f32>,
    pub bpm_a:            RwSignal<Option<f64>>,
    pub bpm_b:            RwSignal<Option<f64>>,
    pub sync_master:      RwSignal<Option<DeckId>>,
    /// When `true`, crossfader uses linear gains; when `false` (default), uses
    /// equal-power (cos/sin) curve for smoother perceived loudness.
    pub crossfader_curve_linear: RwSignal<bool>,
}

// Same false-positive reason as DeckId and MixerState above.
#[allow(dead_code)]
impl MixerState {
    pub fn new() -> Self {
        Self {
            crossfader:    RwSignal::new(0.5f32),
            master_volume: RwSignal::new(1.0f32),
            bpm_a:         RwSignal::new(None),
            bpm_b:         RwSignal::new(None),
            sync_master:   RwSignal::new(None),
            crossfader_curve_linear: RwSignal::new(false),
        }
    }
}

impl Default for MixerState {
    fn default() -> Self {
        Self::new()
    }
}
