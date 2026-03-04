use leptos::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DeckId { A, B }

#[allow(dead_code)]
#[derive(Clone)]
pub struct MixerState {
    pub crossfader:    RwSignal<f32>,
    pub master_volume: RwSignal<f32>,
    pub bpm_a:         RwSignal<Option<f64>>,
    pub bpm_b:         RwSignal<Option<f64>>,
    pub sync_master:   RwSignal<Option<DeckId>>,
}

#[allow(dead_code)]
impl MixerState {
    pub fn new() -> Self {
        Self {
            crossfader:    RwSignal::new(0.5f32),
            master_volume: RwSignal::new(1.0f32),
            bpm_a:         RwSignal::new(None),
            bpm_b:         RwSignal::new(None),
            sync_master:   RwSignal::new(None),
        }
    }
}

impl Default for MixerState {
    fn default() -> Self {
        Self::new()
    }
}
