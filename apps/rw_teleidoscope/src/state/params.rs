use leptos::prelude::*;

/// Plain-data snapshot of all [`KaleidoscopeParams`] signal values.
///
/// Created by [`KaleidoscopeParams::snapshot`].  Passed to the renderer each
/// frame to avoid borrow-checker conflicts between the Leptos signal system
/// and `glow::Context` (which is `!Send + !Sync`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamsSnapshot {
    // Symmetry
    pub segments: u32,
    pub rotation: f32,
    pub zoom: f32,
    pub center: (f32, f32),
    // Effects
    pub spiral: f32,
    pub radial_fold: f32,
    pub lens: f32,
    pub ripple: f32,
    pub mobius: bool,
    pub recursive_depth: u32,
    // Color transforms
    pub hue_shift: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub posterize: u32,
    pub invert: bool,
}

/// All parameters that drive the WebGL renderer.
/// Provided via context; access with `expect_context::<KaleidoscopeParams>()`.
#[derive(Clone, Copy)]
pub struct KaleidoscopeParams {
    // Symmetry
    /// Mirror segment count (2–10).
    pub segments: RwSignal<u32>,
    /// Pattern rotation in degrees (0.0–360.0).
    pub rotation: RwSignal<f32>,
    /// Source sampling scale (0.1–4.0).
    pub zoom: RwSignal<f32>,
    /// Centre of symmetry, normalised 0.0–1.0; updated by canvas drag.
    pub center: RwSignal<(f32, f32)>,

    // Effects (0.0 = off, 1.0 = full)
    /// Spiral twist intensity.
    pub spiral: RwSignal<f32>,
    /// Radial fold intensity.
    pub radial_fold: RwSignal<f32>,
    /// Lens distortion intensity.
    pub lens: RwSignal<f32>,
    /// Angular ripple intensity.
    pub ripple: RwSignal<f32>,
    /// Möbius segment flip on/off.
    pub mobius: RwSignal<bool>,
    /// Recursive reflection passes (0–3).
    pub recursive_depth: RwSignal<u32>,

    // Color transforms
    /// Hue rotation in degrees (0.0–360.0).
    pub hue_shift: RwSignal<f32>,
    /// Saturation multiplier (0.0–2.0; 1.0 = unchanged).
    pub saturation: RwSignal<f32>,
    /// Brightness multiplier (0.0–2.0; 1.0 = unchanged).
    pub brightness: RwSignal<f32>,
    /// Posterize levels (0 = off, 2–16 = active).
    pub posterize: RwSignal<u32>,
    /// Colour inversion on/off.
    pub invert: RwSignal<bool>,
}

impl KaleidoscopeParams {
    /// Create a new `KaleidoscopeParams` with sensible defaults.
    pub fn new() -> Self {
        Self {
            segments:        RwSignal::new(6),
            rotation:        RwSignal::new(0.0),
            zoom:            RwSignal::new(1.0),
            center:          RwSignal::new((0.5, 0.5)),
            spiral:          RwSignal::new(0.0),
            radial_fold:     RwSignal::new(0.0),
            lens:            RwSignal::new(0.0),
            ripple:          RwSignal::new(0.0),
            mobius:          RwSignal::new(false),
            recursive_depth: RwSignal::new(0),
            hue_shift:       RwSignal::new(0.0),
            saturation:      RwSignal::new(1.0),
            brightness:      RwSignal::new(1.0),
            posterize:       RwSignal::new(0),
            invert:          RwSignal::new(false),
        }
    }
}

impl Default for KaleidoscopeParams {
    fn default() -> Self {
        Self::new()
    }
}

impl KaleidoscopeParams {
    /// Read every signal and return a plain-data snapshot.
    ///
    /// Calling this inside a Leptos `Effect` registers **all** signals as
    /// reactive dependencies, so any parameter change triggers a redraw.
    pub fn snapshot(&self) -> ParamsSnapshot {
        ParamsSnapshot {
            segments:        self.segments.get(),
            rotation:        self.rotation.get(),
            zoom:            self.zoom.get(),
            center:          self.center.get(),
            spiral:          self.spiral.get(),
            radial_fold:     self.radial_fold.get(),
            lens:            self.lens.get(),
            ripple:          self.ripple.get(),
            mobius:          self.mobius.get(),
            recursive_depth: self.recursive_depth.get(),
            hue_shift:       self.hue_shift.get(),
            saturation:      self.saturation.get(),
            brightness:      self.brightness.get(),
            posterize:       self.posterize.get(),
            invert:          self.invert.get(),
        }
    }
}
