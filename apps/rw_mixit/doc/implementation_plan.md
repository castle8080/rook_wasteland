# rw_mixit — Implementation Plan

**Stack:** Rust · Leptos 0.8 (CSR) · WebAssembly · Web Audio API · Trunk  
**Deployment:** Static files at `/rw_mixit/` on a shared static host

Each task is sized to be a single commit. Milestones are collections of tasks that can be demo'd and manually tested together as a visible, audible feature.

**Status key:** `⬜ Not Started` · `🔄 In Progress` · `✅ Done` · `🚫 Blocked`

---

## Milestone 0 — Project Scaffold

Stand up the skeleton: compilable WASM app with routing, layout shell, and base CSS. Nothing functional yet but the foundation everything else builds on.

**Milestone success criteria:** `trunk serve` compiles without warnings. A 3-column layout renders at `http://localhost:8080/rw_mixit/`. Clicking `[Settings]` and `[About]` in the header swaps views via URL hash without a page reload. Deck columns are empty placeholders. Cartoon font loads.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T0.1 | Init Cargo workspace: `Cargo.toml` with all v1 crates (`leptos/csr`, `web-sys`, `wasm-bindgen`, `gloo-events`, `console_error_panic_hook`, `rustfft`), `wasm32-unknown-unknown` target, release profile with `opt-level="z"` / `lto=true` | `cargo check --target wasm32-unknown-unknown` passes | ✅ Done |
| T0.2 | `Trunk.toml` with `public_url = "/rw_mixit/"`, `index.html` with `<div id="app">` and Trunk asset links | `trunk build` produces a `dist/` directory with correct base paths | ✅ Done |
| T0.3 | `src/main.rs` entry: `console_error_panic_hook::set_once()` + `mount_to_body(App)` | Browser console shows no panic; WASM loads | ✅ Done |
| T0.4 | `src/routing.rs`: `Route` enum (`Main`, `Settings`, `About`), `from_hash()`, `to_hash()` | Unit-testable; all hash strings round-trip correctly | ✅ Done |
| T0.5 | `src/app.rs`: `App` component reads initial hash, `hashchange` listener via `gloo-events`, `provide_context(current_route)`, `<Show>` gates for each route | Navigating to `#/settings` and `#/about` renders placeholder views; browser back/forward works | ✅ Done |
| T0.6 | `src/components/header.rs`: `<Header>` with logo link (`#/`) and nav links (`[Settings]` `[About]`) that call `set_hash` | Clicking all three links changes the URL fragment and swaps views | ✅ Done |
| T0.7 | `src/components/deck.rs` shell, `src/components/mixer.rs` shell: empty `<div>` placeholders with CSS class names; `<DeckView>` lays them out in a 3-column flex/grid row | Three columns visible with placeholder labels "DECK A", "MIXER", "DECK B" | ✅ Done |
| T0.8 | `static/style.css`: CSS custom properties (colors, deck-a/deck-b accents), Bangers font face from `static/fonts/`, base cartoon border/shadow rule, `box-sizing: border-box` reset | Bangers font renders on all text; black outline style visible on bordered elements | ✅ Done |

---

## Milestone 1 — Audio Foundation & File Loading

Wire up the full Web Audio node graph for both decks and load a file into it. No playback yet — just the infrastructure.

**Milestone success criteria:** Click "Load Track" on each deck, pick an audio file, and see the track name and duration appear on the deck. No browser console errors. The `AudioContext` is created only after the first user interaction.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T1.1 | `src/state/deck.rs`: `DeckState` struct with all `RwSignal<T>` fields from the tech spec (is_playing, playback_rate, volume, track_name, duration_secs, current_secs, loop_active, loop_in, loop_out, hot_cues, eq_high/mid/low, filter_val, fx_echo/fx_reverb/fx_flanger/fx_stutter/fx_scratch booleans, vu_level, waveform_peaks) + `DeckState::new()` constructor | All signals have correct default values | ✅ Done |
| T1.2 | `src/state/mixer.rs`: `MixerState` struct (`crossfader`, `master_volume`, `bpm_a`, `bpm_b`, `sync_master`) + `MixerState::new()` | Signals initialize at sensible defaults (crossfader=0.5, master_volume=1.0) | ✅ Done |
| T1.3 | `src/audio/context.rs`: `ensure_audio_context()` — lazy `AudioContext` creation on first call, stored as `Rc<RefCell<Option<AudioContext>>>` provided via Leptos context from `App` | `AudioContext` is `None` before first interaction; created on first call; subsequent calls return same instance | ✅ Done |
| T1.4 | `src/audio/deck_audio.rs`: `AudioDeck` struct holding all node handles (pre_gain, eq_high/mid/low, sweep_filter, reverb+dry/wet, echo_delay+feedback+wet/dry, flanger_delay+lfo+depth+wet, channel_gain, analyser); `AudioDeck::new(ctx)` constructs and connects the full node chain: `source(opt)` → `pre_gain` → `eq_high` → `eq_mid` → `eq_low` → `sweep_filter` → `reverb_dry/wet` → `ConvolverNode` → `echo_delay` → `echo_feedback` → `channel_gain` → `analyser` (output end of chain; crossfader GainNodes added in M5) | Nodes connect without web-sys errors; `AudioDeck` stored in `Rc<RefCell<AudioDeck>>` | ✅ Done |
| T1.5 | `src/audio/loader.rs`: `load_audio_file(file, deck, state, ctx)` async fn — `FileReader` → `ArrayBuffer` Promise → `decodeAudioData` Promise → store `AudioBuffer` in `AudioDeck`; update `state.track_name` and `state.duration_secs` | After awaiting, `DeckState.track_name` contains the filename; `duration_secs` matches actual audio length | ✅ Done |
| T1.6 | `<Deck>` component: "Load Track" button triggers `<input type="file" accept=".mp3,.wav,.ogg,.flac,.aac">` click; `on_change` event calls `load_audio_file` via `spawn_local` | File picker opens on button click; selecting a file updates the deck UI with name and duration | ✅ Done |
| T1.7 | Track label component: displays `state.track_name` (filename, truncated to fit) and formatted `state.duration_secs` (M:SS) | Track name and duration visible on deck after load | ✅ Done |

---

## Milestone 2 — Playback & Waveform

Make audio play and make it visible. Core DJ loop: load → play → see.

**Milestone success criteria:** Load a file, press Play — audio is audible. Press Pause — audio stops, position held. Press Play again — resumes from where it stopped. Press Stop — resets to start. Cue button works. A scrolling waveform with a moving playhead is visible during playback. Clicking the waveform (while paused) seeks.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T2.1 | `src/audio/deck_audio.rs`: `play(offset)` method — creates new `AudioBufferSourceNode`, sets `.buffer`, sets `.playback_rate`, connects to node chain, calls `.start(0, offset)`, stores `started_at = ctx.current_time()`; `pause()` method — records current offset, calls `.stop()`, drops source node | Audio plays from correct offset; pause holds position | ✅ Done |
| T2.2 | `src/audio/deck_audio.rs`: `stop()` method — stops source, resets `offset_at_play` to 0.0; `current_position()` — computes `ctx.current_time() - started_at + offset_at_play` | Stop resets to beginning; position calculation is accurate | ✅ Done |
| T2.3 | `src/audio/deck_audio.rs`: `seek(position)` method — stop current source (if playing), update `offset_at_play`, restart if was playing; `cue()` method — sets `cue_point` to current position (or jumps to it if already set) | Seek moves playhead; cue jump is instant | ✅ Done |
| T2.4 | `src/canvas/raf_loop.rs`: `start_raf_loop()` with recursive `Closure` pattern; per-frame: call `update_current_time` for each deck (reads `AudioDeck.current_position()` → writes `DeckState.current_secs`), then calls draw fns, then `request_animation_frame` for next frame | rAF loop runs continuously; `current_secs` signal updates visibly in a debug `<p>` | ✅ Done |
| T2.5 | Transport controls component (`src/components/controls.rs`): Play/Pause toggle button (icon changes with `is_playing` signal), Stop button, Cue button, Nudge −/+ buttons; each button calls the corresponding `AudioDeck` method | All five buttons work correctly; Play/Pause icon toggles | ✅ Done |
| T2.6 | `src/audio/loader.rs`: `extract_peaks(buffer, num_columns) -> Vec<f32>` — mix L+R channels, compute max-abs per pixel column; call in `load_audio_file` and write result to `state.waveform_peaks` | After load, `waveform_peaks` contains `num_columns` values all in [0.0, 1.0] | ✅ Done |
| T2.7 | `src/canvas/waveform_draw.rs`: static pass — render `waveform_peaks` onto an offscreen `<canvas>` (bar-style, deck accent color) when peaks change; cache the offscreen canvas | Waveform shape visible and stable between playback frames | ✅ Done |
| T2.8 | `src/canvas/waveform_draw.rs`: dynamic overlay pass per rAF frame — composite offscreen onto main canvas, draw vertical playhead line at correct x position based on `current_secs / duration_secs`; scroll waveform so playhead stays centered | Playhead moves smoothly during playback; waveform scrolls | ✅ Done |
| T2.9 | Waveform seek on click: `mousedown` on waveform canvas computes click position → time → calls `seek()`; only active when paused or in cue mode | Clicking waveform while paused moves the playhead | ✅ Done |
| T2.10 | Nudge implementation: `on_mousedown` on Nudge button temporarily sets `playback_rate` to ±1.05× the current rate; `on_mouseup` ramps back to original rate via `AudioParam.linearRampToValueAtTime` over 100 ms | Holding Nudge audibly speeds/slows playback; releasing returns to normal | ✅ Done |
| T2.11 | Waveform zoom controls: `[−]` / `[+]` buttons below waveform canvas; each click multiplies/divides a `zoom_level` signal (range 1×–8×, snapping to powers of 2); `extract_peaks` is re-sampled at the current zoom level or the draw pass uses a wider/narrower time window | Pressing `[+]` zooms in to show a smaller time range with more detail; `[−]` zooms back out | ✅ Done |

---

## Milestone 3 — Platter Animation & Speed Control

The visual heartbeat of the app. A spinning vinyl record that responds to the pitch fader in real time.

**Milestone success criteria:** A cartoon vinyl record with grooves and a center label spins at ~33 RPM while playing. The pitch fader changes playback speed and the platter rotation speed follows instantly. A tonearm sweeps across the record over the track's duration.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T3.1 | `src/canvas/platter_draw.rs`: `draw_platter(canvas_ref, state)` — draws: dark circle background, concentric groove rings (thin arcs), center label circle with track name (abbreviated, Bangers font), white center spindle dot | Platter draws correctly when `current_secs = 0` and `is_playing = false` | ⬜ Not Started |
| T3.2 | Platter rotation in rAF: compute `angle = current_secs * 0.55 * playback_rate * TAU`; apply rotation transform to vinyl record portion of draw (not the label center dot) | Record rotates at correct 33 RPM–equivalent rate during playback | ⬜ Not Started |
| T3.3 | Tonearm drawing: line segment from fixed top-right pivot point, rotated by `(current_secs / duration_secs) * max_sweep_angle`; draw as thick rounded-cap line in contrasting color | Tonearm starts at outer groove, ends near center label over full track duration | ⬜ Not Started |
| T3.4 | `src/components/pitch_fader.rs`: horizontal `<input type="range">` from −1.0 to +1.0, step 0.01; styled as a cartoon lever; updates `DeckState.playback_rate` via `pitch_to_rate(fader)` | Dragging fader updates signal in real time; fader position reads back correctly | ⬜ Not Started |
| T3.5 | `pitch_to_rate()` → `AudioParam.set_value()` via Leptos `Effect`: `Effect::new(move |_| { source.playback_rate().set_value(pitch_to_rate(state.playback_rate.get())); })` | Changing fader audibly changes pitch/speed; the AudioParam is updated not just the signal | ⬜ Not Started |
| T3.6 | Platter `<canvas>` NodeRef wired into `start_raf_loop` for both decks; platter canvas sized correctly and positioned within deck column | Both platters draw and animate independently at correct size | ⬜ Not Started |

---

## Milestone 4 — BPM Detection & Sync

Tempo awareness: detect BPM on load, allow manual TAP override, and snap two decks into sync.

**Milestone success criteria:** Load a track with a clear beat (e.g., a drum loop) — a BPM value appears on the deck within a second. TAP BPM button refines or overrides the value. With two tracks loaded at different speeds, pressing SYNC on one snaps its playback rate to match the other deck's BPM.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T4.1 | `src/audio/bpm.rs`: `compute_spectral_flux(samples: &[f32], sample_rate: f32) -> Vec<f32>` — frame signal with 1024-sample Hanning window, 512-sample hop; FFT each frame via `rustfft`; compute half-wave-rectified spectral flux per frame; smooth with 5-frame moving average | Flux vector has non-zero values on a track with beats; near-zero on silence | ⬜ Not Started |
| T4.2 | `src/audio/bpm.rs`: `estimate_bpm(flux: &[f32], sample_rate: f32, hop: usize) -> f64` — autocorrelation of flux over lags for BPM 60–200; find `argmax`; octave correction; return BPM in [60.0, 200.0] | Returns ~128 BPM for a 128 BPM test signal; ~90 for a 90 BPM signal | ⬜ Not Started |
| T4.3 | Wire BPM detection into `load_audio_file`: after `decodeAudioData`, extract channel 0 float data, run `compute_spectral_flux` + `estimate_bpm`, write result to `MixerState.bpm_a` or `bpm_b` | After loading a file, the BPM signal updates within ~200 ms | ⬜ Not Started |
| T4.4 | BPM display component: shows `bpm_a` / `bpm_b` value on each deck formatted to one decimal place; shows "---" if None | BPM value appears after load; shows "---" before any file is loaded | ⬜ Not Started |
| T4.5 | TAP BPM button: on each tap, record `performance.now()`; maintain a rolling window of last 8 tap intervals; compute average interval → BPM; write to `MixerState.bpm_a` / `bpm_b` | Tapping in tempo for ~4 taps produces a stable BPM reading | ⬜ Not Started |
| T4.6 | SYNC button + MASTER indicator: pressing SYNC on Deck B computes `rate_adjustment = bpm_a / bpm_b` and multiplies Deck B's `playback_rate` by it; MASTER indicator shows which deck is the tempo reference; clicking MASTER transfers it | SYNC snaps the deck to matching BPM; MASTER marker is visible | ⬜ Not Started |

---

## Milestone 5 — Mixer Panel

Blend two decks. Crossfader, channel faders, and master volume all working with correct audio routing.

**Milestone success criteria:** Two tracks playing. Sliding the crossfader fully left silences Deck B; fully right silences Deck A; center is equal loudness both (equal-power curve). Channel faders independently control each deck's level. Master volume knob controls overall output.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T5.1 | Add crossfader `GainNode` pair to audio graph: `xfade_gain_a` and `xfade_gain_b`; connect `AudioDeck.channel_gain` output → respective xfade GainNode → `master_gain` → `ctx.destination` for each deck | Audio flows from both decks to destination; both audible at default crossfader 0.5 | ⬜ Not Started |
| T5.2 | `src/components/mixer.rs`: Crossfader `<input type="range">` 0.0→1.0; `Effect` updates `xfade_gain_a.gain = cos(val * π/2)` and `xfade_gain_b.gain = sin(val * π/2)` | Moving crossfader to 0.0 silences Deck B; to 1.0 silences Deck A; constant perceived loudness at center | ⬜ Not Started |
| T5.3 | Channel fader components in Mixer: two vertical `<input type="range">` sliders 0.0→1.0; each `Effect` drives `AudioDeck.channel_gain.gain` | Dragging channel fader A down silences Deck A without affecting Deck B | ⬜ Not Started |
| T5.4 | Master volume knob component: rotary-style `<input type="range">` 0.0→1.0; `Effect` drives `master_gain.gain` | Master volume knob scales overall output level | ⬜ Not Started |
| T5.5 | `<Mixer>` component layout: channel faders side by side, crossfader below them, master volume, BPM display panel | Mixer column visually complete with all controls in correct positions | ⬜ Not Started |

---

## Milestone 6 — Loop Controls

Set loop points and hear seamless loops. Quick bar-length buttons make looping effortless.

**Milestone success criteria:** Press Loop In at a beat, Loop Out a few beats later, toggle Loop ON — audio loops seamlessly between those points. A translucent overlay highlights the loop region on the waveform. Quick-length buttons (½, 1, 2, 4 bars) auto-set the loop out point based on detected BPM.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T6.1 | Loop In button: sets `state.loop_in = state.current_secs.get()` | `loop_in` signal updates to current position | ⬜ Not Started |
| T6.2 | Loop Out button: sets `state.loop_out = state.current_secs.get()`; if `loop_out <= loop_in`, clamp or swap; also sets `state.loop_active = true` (loop activates immediately on setting the out point, per spec §6.4) | `loop_out` signal updates; always greater than `loop_in`; loop is immediately active | ⬜ Not Started |
| T6.3 | Loop toggle button: flips `state.loop_active`; button shows active/inactive state | `loop_active` signal toggles; UI reflects state | ⬜ Not Started |
| T6.4 | Loop boundary check in rAF: when `loop_active && current_secs >= loop_out`: call `seek(loop_in)` then `play(loop_in)` | Audio loops at correct points; no audible gap | ⬜ Not Started |
| T6.5 | Loop region waveform overlay: in dynamic draw pass, compute x-pixel positions of `loop_in` and `loop_out`; fill translucent rect between them | Loop region visible as colored overlay on waveform | ⬜ Not Started |
| T6.6 | Quick loop bar buttons (½, 1, 2, 4, 8): `loop_in` = current position; `loop_out = loop_in + (bars × beat_duration × 4)`; auto-enables `loop_active`; requires BPM to be set | Each button immediately creates and activates a loop of the correct musical length | ⬜ Not Started |

---

## Milestone 7 — Hot Cues

Instant jump points. The backbone of live performance on the app.

**Milestone success criteria:** Hold a hot cue button → the current playhead position is saved and the button lights up. Tap the lit button → playback instantly jumps to that position. Double-tap → cue is cleared and button dims. Saved cue positions show as colored vertical markers on the waveform.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T7.1 | `src/components/hot_cues.rs`: 4 buttons in deck colors (red, blue, green, yellow); each button knows its index (0–3) and reads `state.hot_cues.get()[index]` to show set/unset state | Buttons render; unset buttons appear dim; set buttons appear lit | ⬜ Not Started |
| T7.2 | Hold-to-set: `pointerdown` starts a timer; if still held after 300 ms, set `hot_cues[index] = Some(current_secs)` and give haptic/visual feedback | Holding 300 ms sets cue; quick tap does not set | ⬜ Not Started |
| T7.3 | Tap-to-jump: `pointerdown` + `pointerup` within 300 ms on a cue that is `Some(_)` → `seek(cue_pos)` | Tapping a set cue button seeks to its saved position | ⬜ Not Started |
| T7.4 | Double-tap-to-clear / right-click-to-clear: detect two tap events within 400 ms OR a `contextmenu` event → set `hot_cues[index] = None`; call `event.prevent_default()` on right-click to suppress browser menu | Double-tapping or right-clicking a set cue clears it; button dims | ⬜ Not Started |
| T7.5 | Hot cue markers on waveform: in dynamic draw pass, iterate `hot_cues`; for each `Some(pos)`, draw a colored vertical line at the corresponding x position | Each set hot cue shows a colored line on the waveform | ⬜ Not Started |

---

## Milestone 8 — EQ, Filter & VU Meter

Shape the sound per-deck and see it metered.

**Milestone success criteria:** Turning the Low EQ knob all the way down audibly removes bass from that deck. The filter knob sweeps a low-pass then high-pass filter — leftmost is muddy bass, rightmost is thin treble, center is flat. The VU meter bar bounces in real time with the audio level.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T8.1 | `src/components/eq.rs`: rotary knob component — `<input type="range">` styled as a circle; range −12 to +12 dB; `on_input` updates the corresponding signal | Knob updates signal value on drag | ⬜ Not Started |
| T8.2 | EQ Effects: `Effect::new` for each band that calls `node.gain().set_value(db)` on the corresponding `BiquadFilterNode`; configure node types and frequencies on construction (highshelf/8kHz, peaking/1kHz/Q=0.7, lowshelf/200Hz) | Turning Low EQ to −12 dB audibly removes bass; +12 boosts it | ⬜ Not Started |
| T8.3 | Filter knob component and `apply_sweep_filter()` Effect: knob range −1.0 to +1.0; Effect calls `apply_sweep_filter` from tech spec §8.7 on `sweep_filter` node | Center position is flat; left sweeps low-pass closed; right sweeps high-pass closed | ⬜ Not Started |
| T8.4 | `src/canvas/raf_loop.rs`: call `read_vu_level(analyser)` each frame → `state.vu_level.set(level)` for each deck | `vu_level` signal updates 60 times/second during playback | ⬜ Not Started |
| T8.5 | VU meter UI component: `<div>` whose CSS `height` is set reactively from `vu_level` signal (as a percentage); styled as animated green/yellow/red bars | Meter bar height tracks audio level visually in real time | ⬜ Not Started |

---

## Milestone 9 — Effects Panel

Echo, reverb, flanger, stutter, and scratch simulation.

**Milestone success criteria:** Each of the five effects can be toggled on/off, is clearly audible when on, and has at least one working parameter knob. Dragging the mouse on the platter with Scratch enabled changes the playback rate in response to the drag direction.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T9.1 | Echo/Delay: `EchoNodes` construction (`DelayNode` max 2s + feedback `GainNode` + wet/dry `GainNode`s); insert into audio graph; FX panel toggle → ramp wet to 0.6 / 0.0 over 20 ms; time and feedback knobs | Echo is audible when toggled on; feedback knob creates more/fewer repeats | ⬜ Not Started |
| T9.2 | Reverb: `generate_reverb_ir()` fn (from tech spec §8.8); `ConvolverNode` loaded with IR on deck construction; wet/dry bypass GainNodes; FX panel toggle; duration and decay knobs route to `generate_reverb_ir` and reload the IR | Reverb tail is audible when toggled on; duration knob changes tail length | ⬜ Not Started |
| T9.3 | Flanger: `FlangerNodes` construction (from tech spec §8.10); LFO starts on deck init; FX panel toggle → ramp wet in/out; rate and depth knobs | Sweeping comb-filter flange is audible when toggled on; rate knob changes sweep speed | ⬜ Not Started |
| T9.4 | Stutter: `GainNode` inserted as pre-FX gate node; FX panel toggle → call `schedule_stutter()` with rolling lookahead (from tech spec §8.11); on disable, `cancelScheduledValues` + ramp to 1.0; subdivision selector (1/4, 1/8, 1/16, 1/32) | Stutter gates audio rhythmically when on; subdivision selector changes chop rate | ⬜ Not Started |
| T9.5 | Scratch simulation: FX panel toggle enables scratch mode on platter canvas; `pointerdown` / `pointermove` / `pointerup` listeners on platter canvas; angular velocity → playbackRate via `on_platter_mouse_move` (from tech spec §8.12); `on_platter_mouse_up` ramps back | Dragging on the platter changes pitch/speed proportionally to drag speed; release returns to normal | ⬜ Not Started |
| T9.6 | FX panel layout (`src/components/fx_panel.rs`): 5 toggle buttons in a row; active buttons visually highlighted; param knobs appear below each active effect | FX panel looks correct; active state visual feedback works | ⬜ Not Started |

---

## Milestone 10 — Keyboard Shortcuts

Play the app without touching the mouse.

**Milestone success criteria:** Every keyboard shortcut from the product spec §9 triggers the correct action. Pressing Space plays/pauses Deck A. Pressing Enter plays/pauses Deck B. Shortcuts do not fire when focus is inside a text input.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T10.1 | `src/utils/keyboard.rs`: `is_input_focused() -> bool` — checks `document.activeElement` tag name is not INPUT or TEXTAREA | Returns true when a text input is focused; false otherwise | ⬜ Not Started |
| T10.2 | `register_keyboard_shortcuts(deck_a, deck_b)` — global `keydown` listener on `window` via `gloo-events`; returns `EventListener` held at app root to prevent drop | Listener registered; no double-firing | ⬜ Not Started |
| T10.3 | Deck A shortcut handlers: `Space` → play/pause; `KeyQ` → cue; `KeyZ` → loop in; `KeyX` → loop out; `ArrowLeft` / `ArrowRight` → nudge; `Digit1`–`Digit4` → hot cues | Each key triggers the documented action on Deck A | ⬜ Not Started |
| T10.4 | Deck B shortcut handlers: `Enter` → play/pause; `KeyP` → cue; `KeyN` → loop in; `KeyM` → loop out; `BracketLeft` / `BracketRight` → nudge; `Digit7`–`Digit0` → hot cues | Each key triggers the documented action on Deck B | ⬜ Not Started |

---

## Milestone 11 — Settings, About & Visual Polish

Complete the UI: secondary views, cartoon aesthetic fully applied, animations alive.

**Milestone success criteria:** The app looks like a Saturday-morning cartoon. Settings view lets you tune reverb and crossfader curve. About view shows version and stack. Buttons bounce on press, hot cue triggers emit a star-burst animation, active FX buttons wiggle. The layout doesn't break when the browser window is narrowed.

| ID | Task | Success Criteria | Status |
|---|---|---|---|
| T11.1 | `src/components/settings.rs`: Settings view with reverb duration slider (0.5–3.5 s) and decay slider (0.5–4.0) → call `generate_reverb_ir()` and reload the `ConvolverNode`; crossfader curve toggle (equal-power / linear) | Changing reverb duration audibly changes tail length after confirming | ⬜ Not Started |
| T11.2 | `src/components/about.rs`: About view with version string, stack bullet list | About page renders correctly at `#/about` | ⬜ Not Started |
| T11.3 | Cartoon CSS pass — panels: thick black borders (`3px solid #111`), `box-shadow: 4px 4px 0 #111`, `border-radius: 12px` on panels, `50%` on knobs/buttons; deck A accent (`--color-deck-a: #3b82f6`), deck B accent (`--color-deck-b: #f97316`) applied throughout | App visually resembles the wireframe aesthetic | ⬜ Not Started |
| T11.4 | Button press animation: add `.active` class on `pointerdown`, remove on `pointerup`; CSS `@keyframes pop { 0%{scale:1} 50%{scale:0.92} 100%{scale:1} }` with `animation: pop 80ms ease-out` on `.active` | Every button has a satisfying physical "press" feel | ⬜ Not Started |
| T11.5 | Hot cue burst animation: on hot cue trigger, briefly render a cartoon star-burst `<div>` over the button with `@keyframes burst { 0%{opacity:1;scale:0.5} 100%{opacity:0;scale:1.8} }` | A star-burst appears for ~200 ms on hot cue tap | ⬜ Not Started |
| T11.6 | FX active wiggle animation: when `fx_* = true`, add a CSS class with `@keyframes wiggle { 0%,100%{rotate:0} 25%{rotate:-3deg} 75%{rotate:3deg} }` `animation: wiggle 0.4s infinite` | Active FX buttons visibly wiggle | ⬜ Not Started |
| T11.7 | Loop region pulse animation: when `loop_active = true`, the waveform loop overlay pulses opacity via `@keyframes pulse { 0%,100%{opacity:0.3} 50%{opacity:0.6} }` (CSS class toggled by Leptos signal) | Loop overlay gently pulses when loop is active | ⬜ Not Started |
| T11.8 | Platter label: render track name (truncated to ~16 chars) inside the label circle using `CanvasRenderingContext2d.fill_text()`; Bangers font via canvas `font` property | Track name appears on the spinning label circle | ⬜ Not Started |
| T11.9 | Responsive layout: replace fixed pixel widths with `flex: 1` or `min-width` constraints; verify at 1024 px, 1280 px, 1440 px viewport widths — no overflow, no clipping | Layout stays usable at 1024 px wide without horizontal scrolling | ⬜ Not Started |
| T11.10 | Load button animations: CSS `:hover` state adds a `translateY(-2px)` bounce via `@keyframes loadhover`; on successful file load, add a `.slot-in` class to the platter for 400 ms that plays a brief downward scale-in animation simulating the record "slotting in" | Load button visibly bounces on hover; platter animates on successful load | ⬜ Not Started |
| T11.11 | Crossfader glow animation: add CSS class `.xfader-moving` while the crossfader is being dragged (set on `pointerdown`, remove on `pointerup`); class applies `box-shadow: 0 0 12px var(--color-glow)` and a slow `@keyframes pulse-glow` | Crossfader glows while being dragged; glow fades when released | ⬜ Not Started |

---

## Summary

| Milestone | Tasks | Status |
|---|---|---|
| M0 — Project Scaffold | T0.1 – T0.8 (8 tasks) | ✅ Done |
| M1 — Audio Foundation & File Loading | T1.1 – T1.7 (7 tasks) | ✅ Done |
| M2 — Playback & Waveform | T2.1 – T2.11 (11 tasks) | ✅ Done |
| M3 — Platter Animation & Speed Control | T3.1 – T3.6 (6 tasks) | ⬜ Not Started |
| M4 — BPM Detection & Sync | T4.1 – T4.6 (6 tasks) | ⬜ Not Started |
| M5 — Mixer Panel | T5.1 – T5.5 (5 tasks) | ⬜ Not Started |
| M6 — Loop Controls | T6.1 – T6.6 (6 tasks) | ⬜ Not Started |
| M7 — Hot Cues | T7.1 – T7.5 (5 tasks) | ⬜ Not Started |
| M8 — EQ, Filter & VU Meter | T8.1 – T8.5 (5 tasks) | ⬜ Not Started |
| M9 — Effects Panel | T9.1 – T9.6 (6 tasks) | ⬜ Not Started |
| M10 — Keyboard Shortcuts | T10.1 – T10.4 (4 tasks) | ⬜ Not Started |
| M11 — Settings, About & Visual Polish | T11.1 – T11.11 (11 tasks) | ⬜ Not Started |
| **Total** | **81 tasks** | |

---

*Task order within milestones reflects implementation dependency. Milestones M0–M2 are strictly sequential. M3–M5 can begin in parallel once M2 is done. M6–M11 depend on M5 being complete.*
