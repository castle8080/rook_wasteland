# Tasks T4.1–T4.6: BPM Detection & Sync

**Milestone:** M4 — BPM Detection & Sync
**Status:** ✅ Done

---

## Restatement

Implement automatic BPM detection on file load, a manual TAP BPM override, and a SYNC button that snaps one deck's playback rate to match the other deck's detected tempo. The core DSP (spectral flux + autocorrelation) lives in `src/audio/bpm.rs` as pure Rust so it is unit-testable on the host without a browser. BPM signals live on `MixerState` (already defined in M1) and are wired from `DeckView` down to individual `Deck` components via props. The MASTER indicator tracks which deck is the tempo reference. Out of scope: AudioWorklet-based real-time onset detection, tempo-preserving pitch shift, persistent BPM values across sessions.

---

## Design

### Data flow

**Auto-detect (T4.1–T4.3):**
```
load_audio_file (async)
  → audio_buffer.get_channel_data(0)  [web-sys, WASM only]
  → compute_spectral_flux(&samples, sr)  [pure Rust]
  → estimate_bpm(&flux, sr, HOP_SIZE)    [pure Rust]
  → bpm_signal.set(Some(bpm))            [RwSignal<Option<f64>>]
  → BpmPanel re-renders with new value
```

**TAP BPM (T4.5):**
```
on:click on TAP button
  → window.performance().now()  → push to tap_times: Rc<RefCell<Vec<f64>>>
  → compute intervals from adjacent timestamps
  → tap_bpm_from_intervals(&intervals)  [pure Rust]
  → bpm_own.set(Some(bpm))
```

**SYNC (T4.6):**
```
on:click on SYNC button
  → new_rate = current_rate × (bpm_other / bpm_own)  [clamped 0.25–4.0]
  → playback_rate.set(new_rate)    [already wired to AudioParam via Effect in deck.rs]
  → sync_master.set(Some(deck_id))
```

### Function / type signatures

```rust
// src/audio/bpm.rs
pub const WINDOW_SIZE: usize = 1024;
pub const HOP_SIZE: usize = 512;

pub fn compute_spectral_flux(samples: &[f32], _sample_rate: f32) -> Vec<f32>
pub fn estimate_bpm(flux: &[f32], sample_rate: f32, hop: usize) -> f64
pub fn tap_bpm_from_intervals(intervals_ms: &[f64]) -> Option<f64>
```

```rust
// src/audio/loader.rs  (signature change)
pub async fn load_audio_file(
    file: web_sys::File,
    deck: Rc<RefCell<AudioDeck>>,
    state: DeckState,
    ctx: AudioContext,
    bpm_signal: RwSignal<Option<f64>>,  // NEW
)
```

```rust
// src/components/deck.rs  (new component)
#[component]
pub fn BpmPanel(
    deck_id:       DeckId,
    bpm_own:       RwSignal<Option<f64>>,
    bpm_other:     RwSignal<Option<f64>>,
    playback_rate: RwSignal<f64>,
    sync_master:   RwSignal<Option<DeckId>>,
) -> impl IntoView
```

### Edge cases

- No file loaded: `bpm_own` is `None` → display shows "---"; SYNC is a no-op (guard on `is_none`).
- Single tap: only one timestamp in `tap_times` → fewer than 2 intervals → `tap_bpm_from_intervals` returns `None` → signal unchanged.
- `bpm_own == 0.0` at SYNC time: division guard prevents NaN.
- Short audio (fewer frames than `lag_min`): `estimate_bpm` returns 120.0 default.
- `hop == 0`: `estimate_bpm` returns 120.0 default.
- Computed BPM always clamped to [60, 200] by `estimate_bpm`.
- MASTER: both decks share the same `sync_master` signal → reactive update is automatic.

### Integration points

- `src/audio/loader.rs` — adds `bpm_signal` parameter; call sites in `src/components/deck.rs` updated.
- `src/state/mixer.rs` — `DeckId` gets `Copy` derive (needed in closures/props).
- `src/components/deck.rs` — `DeckView` creates `MixerState`, distributes signals to `Deck`; `Deck` gains 4 new props; `BpmPanel` is new.
- `static/style.css` — new `.bpm-panel`, `.bpm-display`, `.bpm-value`, `.btn-tap`, `.btn-sync`, `.btn-master` rules.

---

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Autocorrelation finds double-period lag for 128 BPM (integer-alignment artefact: lag 81 slightly outscores lag 40) | Sub-lag check: if `best_lag/2 >= lag_min` and its correlation ≥ 50% of `best_corr`, prefer the shorter lag. This correctly doubles the BPM without disturbing correctly-detected tempos like 90 BPM. |
| Simplicity | Could pass full `MixerState` to `Deck` instead of individual signals | Individual signal props are more explicit about what each component actually uses; avoids entangling Deck with the full mixer. |
| Coupling | `load_audio_file` signature change breaks existing call site | Only one call site in `deck.rs`; updated in the same commit. |
| Performance | BPM detection runs synchronously in the `load_audio_file` async task | Acceptable: happens once per file load, not in the rAF loop. For a 5-minute track at SR=44100/hop=512 ≈ 25,800 frames, the O(n) autocorrelation over ~60 lags is ~1.5M multiplications — fast enough on WASM. |
| Testability | `compute_spectral_flux` and `estimate_bpm` are pure Rust | Both are fully unit-testable on the host; no browser or WASM required. |

---

## Implementation Notes

- `DeckId::Copy` was a necessary pre-requisite for using `deck_id` inside `move` closures inside `BpmPanel`.
- The sub-lag octave correction works because: for lag `2T` vs lag `T`, the shorter lag `T` has ~twice as many autocorrelation pairs but each pair may drift. When `half_lag < lag_min`, the check is skipped (protects correctly-detected fast tempos). When `half_lag >= lag_min` and correlation ≥ 50%, the doubled BPM is preferred.
- `tap_times` is `Rc<RefCell<Vec<f64>>>` (not a signal) because: (a) taps don't need reactive updates — only the computed BPM does; (b) using a signal for mutable-interior state inside a click handler requires `update()` which is also valid but adds unnecessary re-render cycles.

---

## Test Results

**Automated (50 tests, 0 failures):**
```
test audio::bpm::tests::estimate_bpm_empty_flux_returns_120 ... ok
test audio::bpm::tests::estimate_bpm_flux_too_short_for_lag_range_returns_120 ... ok
test audio::bpm::tests::estimate_bpm_hop_zero_returns_120 ... ok
test audio::bpm::tests::estimate_bpm_on_128_beat ... ok
test audio::bpm::tests::estimate_bpm_on_170_beat ... ok
test audio::bpm::tests::estimate_bpm_on_90_beat ... ok
test audio::bpm::tests::estimate_bpm_output_always_in_spec_range ... ok
test audio::bpm::tests::estimate_bpm_result_in_range ... ok
test audio::bpm::tests::spectral_flux_exactly_window_size_gives_one_frame ... ok
test audio::bpm::tests::spectral_flux_near_zero_on_silence ... ok
test audio::bpm::tests::spectral_flux_non_empty_for_beat_signal ... ok
test audio::bpm::tests::spectral_flux_returns_empty_for_short_input ... ok
test audio::bpm::tests::tap_bpm_120_for_500ms_intervals ... ok
test audio::bpm::tests::tap_bpm_90_for_667ms_intervals ... ok
test audio::bpm::tests::tap_bpm_none_for_empty_slice ... ok
test audio::bpm::tests::tap_bpm_none_for_single_interval ... ok
test result: ok. 50 passed; 0 failed
```

**Clippy (`--target wasm32-unknown-unknown -- -D warnings`):** ✅ 0 warnings  
**`trunk build`:** ✅ success

**Manual steps to perform in browser:**
- [ ] Load a drum loop with known BPM — verify display shows correct value within ~1 second
- [ ] Load silence / ambient audio — BPM shows a value (auto-detect may be inaccurate; TAP override works)
- [ ] TAP BPM: tap 8+ times in tempo — value stabilises to correct BPM
- [ ] Load two tracks at different speeds; press SYNC on one — platter + audio speed changes to match the other
- [ ] MASTER button lights up on click; transfers between decks

---

## Review Notes

- All public functions in `bpm.rs` have `///` doc comments. ✓
- `WINDOW_SIZE` and `HOP_SIZE` are named constants. ✓
- `tap_bpm_from_intervals` returns `Option<f64>` (not a panic or magic default) for the under-2-intervals case. ✓
- `let _ = best_corr;` suppressor is intentional — the variable is only used for the comparison guard in the sub-lag check and Rust otherwise warns it is assigned but never read. Added inline comment explaining this.
- No `.unwrap()` without `.expect()` in M4 code. ✓

---

## Callouts / Gotchas

- **Sub-lag check threshold (0.50):** This value was chosen empirically for the 128 BPM test case. It may mis-correct for tracks where the true tempo autocorrelation at the full lag is much stronger than at the half lag for a reason unrelated to octave doubling. If future integration testing with real tracks reveals false doublings, increase the threshold toward 0.70.
- **BPM detection accuracy on real tracks:** The spectral flux + autocorrelation approach works well for electronic/hip-hop with clear transients. It is less reliable on acoustic music with soft attacks. TAP BPM is the intended fallback.
- **Performance on long tracks:** A 10-minute track at 44100 Hz produces ~51,000 flux frames. The O(n × lags) autocorrelation (n=51k, lags=60) = ~3M multiplications — still fast, but if future profiling shows load latency on long files, consider sub-sampling the flux vector.
