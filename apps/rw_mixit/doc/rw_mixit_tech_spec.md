# rw_mixit — Technical Implementation Spec

**Project:** rw_mixit  
**Status:** Draft  
**Version:** 0.1.0  
**Companion doc:** `rw_mixit_spec.md` (product requirements)

---

## 1. Executive Summary

`rw_mixit` is a **pure client-side WebAssembly** application. There is no backend. All audio processing runs in the browser via the Web Audio API; all UI reactivity is handled by Leptos in CSR (client-side rendering) mode. The build artifact is a single directory of static files deployable to any static web server (nginx, Apache, GitHub Pages, CDN). Trunk is used to build and bundle.

---

## 2. Technology Stack — Pinned Versions

| Tool / Crate | Version | Role |
|---|---|---|
| Rust toolchain | stable (≥1.78) | Language |
| `wasm32-unknown-unknown` target | — | Compile target |
| `leptos` | 0.8.x (latest: 0.8.17) | Reactive UI framework (CSR mode only) |
| `wasm-bindgen` | 0.2.x | Rust↔JS interop |
| `web-sys` | 0.3.x | Web API bindings (DOM, Web Audio, Canvas, File) |
| `js-sys` | 0.3.x | JS primitive types |
| `gloo-events` | 0.2.x | DOM event listener helpers |
| `console_error_panic_hook` | 0.1.x | Panic → browser console |
| `wee_alloc` (optional) | 0.4.x | Smaller WASM binary allocator |
| Trunk | 0.21.x (latest: 0.21.14) | WASM build + bundle + dev server |
| wasm-opt (via Trunk) | binaryen latest | WASM binary optimization |

**v2 DSP additions (not in v1 binary):**

| Crate | Version | Role |
|---|---|---|
| `dasp` | 0.11.x | DSP primitives: interpolation, ring buffers (if WSOLA ever revived) |

**No server runtime. No npm/node build pipeline. No JS framework alongside Leptos.**

---

## 3. Project Structure

```
rw_mixit/
├── Cargo.toml
├── Trunk.toml
├── index.html                  # Trunk entry point
├── src/
│   ├── main.rs                 # WASM entry (mount Leptos app)
│   ├── app.rs                  # Root component + fragment router
│   ├── routing.rs              # Fragment routing logic
│   ├── state/
│   │   ├── mod.rs
│   │   ├── deck.rs             # DeckState signals
│   │   └── mixer.rs            # MixerState signals
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── context.rs          # AudioContext initialization
│   │   ├── deck_audio.rs       # AudioDeck (Web Audio nodes per deck)
│   │   ├── effects.rs          # Echo, reverb, flanger, filter, stutter
│   │   └── loader.rs           # File → ArrayBuffer → AudioBuffer decode
│   ├── components/
│   │   ├── mod.rs
│   │   ├── deck.rs             # <Deck> component (left/right)
│   │   ├── mixer.rs            # <Mixer> center panel
│   │   ├── platter.rs          # <Platter> canvas component
│   │   ├── waveform.rs         # <Waveform> canvas component
│   │   ├── controls.rs         # Play/Cue/Stop/Nudge buttons
│   │   ├── pitch_fader.rs      # Tempo slider
│   │   ├── eq.rs               # 3-band EQ knobs
│   │   ├── hot_cues.rs         # 4 hot cue buttons
│   │   ├── loop_controls.rs    # Loop In/Out/Toggle/Bar buttons
│   │   ├── fx_panel.rs         # Effects toggles + params
│   │   └── help.rs             # <HelpView> — quick-start guide (Route::Help)
│   ├── canvas/
│   │   ├── mod.rs
│   │   ├── raf_loop.rs         # requestAnimationFrame driver
│   │   ├── platter_draw.rs     # Platter drawing logic
│   │   └── waveform_draw.rs    # Waveform + playhead drawing logic
│   └── utils/
│       ├── mod.rs
│       └── keyboard.rs         # Global keyboard shortcut handler
├── static/
│   ├── style.css               # Cartoon CSS styles + keyframe animations
│   └── fonts/                  # Custom cartoon-style web fonts
└── doc/
    ├── rw_mixit_spec.md
    └── rw_mixit_tech_spec.md
```

---

## 4. Leptos Configuration — CSR Only

`rw_mixit` uses Leptos in **client-side rendering (CSR) mode exclusively**. No SSR, no server functions, no hydration. This means:

- `leptos` feature flag: `csr` only.
- No `axum`, `actix`, or any server runtime dependency.
- No `leptos_router` — see Section 6 for the custom fragment router.

**`Cargo.toml` key excerpt:**
```toml
[package]
name = "rw_mixit"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
leptos = { version = "0.8", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
  # DOM
  "Window", "Document", "Element", "HtmlElement", "HtmlCanvasElement",
  "HtmlInputElement", "CanvasRenderingContext2d", "FileList", "File",
  "FileReader", "Blob", "Url",
  # Web Audio
  "AudioContext", "AudioBuffer", "AudioBufferSourceNode",
  "AudioDestinationNode", "AudioNode", "AudioParam", "GainNode",
  "BiquadFilterNode", "BiquadFilterType", "ConvolverNode", "DelayNode",
  "DynamicsCompressorNode", "AnalyserNode", "OfflineAudioContext",
  # Events
  "Event", "MouseEvent", "KeyboardEvent", "HashChangeEvent",
  "ProgressEvent", "EventTarget",
  # Misc
  "Performance", "Navigator", "Location",
] }
js-sys = "0.3"
gloo-events = "0.2"
console_error_panic_hook = "0.1"
# BPM auto-detection (§8.14) — spectral flux + autocorrelation; runs on file load
rustfft = { version = "6", default-features = false }

[profile.release]
opt-level = "z"   # optimize for size
lto = true
codegen-units = 1
```

**`Trunk.toml`:**
```toml
[build]
target = "index.html"
dist = "dist"
public_url = "/rw_mixit/"   # IMPORTANT: app is served from this subdirectory on shared static server

[watch]
ignore = ["dist", "doc"]
```

> **Critical — subdirectory hosting:** `public_url` must be set to `/rw_mixit/` so Trunk injects correct absolute paths for the WASM binary, JS glue, and CSS into `index.html`. Without this, the browser requests assets from `/` and receives 404s. This value must match the deployment path exactly.

**`index.html`:**
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
  <title>rw_mixit</title>
  <link data-trunk rel="css" href="static/style.css"/>
  <link data-trunk rel="copy-dir" href="static/fonts"/>
</head>
<body>
  <div id="app"></div>
</body>
</html>
```

---

## 5. Static Deployment

The app is hosted at **`/rw_mixit/`** on a shared static server alongside other apps. Trunk produces the `dist/` directory which is deployed to that path.

**Deployed files under `/rw_mixit/`:**
- `index.html`
- `rw_mixit_bg.wasm` (optimized WASM binary)
- `rw_mixit.js` (wasm-bindgen glue)
- `style.css`, `fonts/`

> **Fragment routing is subdirectory-safe.**Because routing is entirely hash-based (`/#/settings`, `/#/about`), the server only ever receives requests for `/rw_mixit/` — all navigation happens in the fragment, which is never sent to the server. No `try_files` rewriting is needed for route changes.

> **Note on SharedArrayBuffer:** We are NOT using AudioWorklets with WASM threading in v1, so `Cross-Origin-Opener-Policy` / `Cross-Origin-Embedder-Policy` headers are **not required**.

---

## 6. Fragment-Based Routing (No Library)

The app is largely a single view (the DJ decks), but has a small set of "pages" navigated by URL hash (e.g. `/#/`, `/#/about`, `/#/settings`).

**`src/routing.rs`:**

```rust
#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Main,
    Settings,
    About,
    Help,
}

impl Route {
    pub fn from_hash(hash: &str) -> Self {
        match hash {
            "#/settings" => Route::Settings,
            "#/about"    => Route::About,
            "#/help"     => Route::Help,
            _            => Route::Main,
        }
    }

    pub fn to_hash(&self) -> &'static str {
        match self {
            Route::Main     => "#/",
            Route::Settings => "#/settings",
            Route::About    => "#/about",
            Route::Help     => "#/help",
        }
    }
}
```

**`src/app.rs` — router integration:**

```rust
#[component]
pub fn App() -> impl IntoView {
    // Read initial hash
    let initial_hash = window().location().hash().unwrap_or_default();
    let current_route = RwSignal::new(Route::from_hash(&initial_hash));

    // Listen for browser back/forward (hashchange event)
    let route_signal = current_route;
    let _listener = gloo_events::EventListener::new(
        &window(),
        "hashchange",
        move |_| {
            let hash = window().location().hash().unwrap_or_default();
            route_signal.set(Route::from_hash(&hash));
        },
    );

    // Provide route context to children
    provide_context(current_route);

    view! {
        <div id="rw-mixit-root">
            <Header/>
            <Show when=move || current_route.get() == Route::Main>
                <DeckView/>
            </Show>
            <Show when=move || current_route.get() == Route::Settings>
                <SettingsView/>
            </Show>
            <Show when=move || current_route.get() == Route::About>
                <AboutView/>
            </Show>
            <Show when=move || current_route.get() == Route::Help>
                <HelpView/>
            </Show>
        </div>
    }
}
```

Navigation is performed by setting `window.location.hash` — the `hashchange` listener picks it up automatically:

```rust
fn navigate(route: &Route) {
    let _ = window().location().set_hash(route.to_hash());
}
```

---

## 7. State Architecture

### 7.1 Two Worlds: Reactive vs. Imperative

There are two distinct execution contexts in this application:

| World | Technology | Driven by | Examples |
|---|---|---|---|
| **Reactive UI** | Leptos signals + `view!` macro | User interactions (clicks, drags) | Volume fader, EQ knobs, play button state |
| **Imperative Canvas Loop** | `requestAnimationFrame` closure | rAF timer (~60 fps) | Platter rotation, waveform playhead scroll |

These two worlds share state through `Leptos` signals. The rAF loop reads signal values (using `.get_untracked()` to avoid tracking overhead) each frame. The Leptos reactive graph handles all other updates.

### 7.2 DeckState

```rust
#[derive(Clone)]
pub struct DeckState {
    // Playback
    pub is_playing:     RwSignal<bool>,
    pub playback_rate:  RwSignal<f64>,   // 0.5–2.0 (vinyl mode)
    pub volume:         RwSignal<f32>,   // 0.0–1.0

    // Track info
    pub track_name:     RwSignal<Option<String>>,
    pub duration_secs:  RwSignal<f64>,
    pub current_secs:   RwSignal<f64>,   // updated by rAF loop

    // Loop
    pub loop_active:    RwSignal<bool>,
    pub loop_in:        RwSignal<f64>,
    pub loop_out:       RwSignal<f64>,

    // Hot cues (index 0–3)
    pub hot_cues:       RwSignal<[Option<f64>; 4]>,

    // EQ
    pub eq_high:        RwSignal<f32>,   // dB gain, -12 to +12
    pub eq_mid:         RwSignal<f32>,
    pub eq_low:         RwSignal<f32>,
    pub filter_val:     RwSignal<f32>,   // -1.0 (LP) to +1.0 (HP), 0 = bypass

    // FX
    pub fx_echo:        RwSignal<bool>,
    pub fx_reverb:      RwSignal<bool>,
    pub fx_flanger:     RwSignal<bool>,
    pub fx_stutter:     RwSignal<bool>,
    pub fx_scratch:     RwSignal<bool>,

    // VU meter level (updated by rAF loop, drives meter bar height)
    pub vu_level:       RwSignal<f32>,   // 0.0–1.0

    // Waveform data (set once on load, read by canvas draw)
    pub waveform_peaks: RwSignal<Option<Vec<f32>>>,
}
```

### 7.3 AudioDeck (Web Audio nodes)

`AudioDeck` holds the actual `web-sys` audio graph objects. It is stored in `Rc<RefCell<AudioDeck>>` — it is **not** a Leptos signal because the Web Audio nodes are JS objects not suited to reactive ownership.

```rust
pub struct AudioDeck {
    pub ctx:            AudioContext,
    pub source:         Option<AudioBufferSourceNode>,  // recreated on each play
    pub buffer:         Option<AudioBuffer>,

    // Pre-FX / stutter gate
    pub pre_gain:       GainNode,

    // 3-band EQ + sweep filter
    pub eq_high:        BiquadFilterNode,
    pub eq_mid:         BiquadFilterNode,
    pub eq_low:         BiquadFilterNode,
    pub sweep_filter:   BiquadFilterNode,

    // Reverb (ConvolverNode + dry/wet bypass GainNodes)
    pub reverb:         ConvolverNode,
    pub reverb_dry:     GainNode,
    pub reverb_wet:     GainNode,

    // Echo / delay (delay + feedback loop + wet/dry)
    pub echo_delay:     DelayNode,
    pub echo_feedback:  GainNode,
    pub echo_wet:       GainNode,
    pub echo_dry:       GainNode,

    // Flanger (short delay driven by LFO)
    pub flanger_delay:  DelayNode,
    pub flanger_lfo:    OscillatorNode,
    pub flanger_depth:  GainNode,       // LFO amplitude → DelayNode.delayTime
    pub flanger_wet:    GainNode,

    // Channel output
    pub channel_gain:   GainNode,       // channel fader
    pub analyser:       AnalyserNode,   // used by VU meter rAF

    // Timing bookkeeping
    pub started_at:     Option<f64>,    // AudioContext.currentTime when play started
    pub offset_at_play: f64,            // track offset when play() was called
}
```

Leptos `Effect`s connect the reactive signals to the imperative audio nodes:

```rust
// Example: volume fader drives GainNode
Effect::new(move |_| {
    let vol = deck_state.volume.get();
    audio_deck.borrow().gain.gain().set_value(vol);
});
```

### 7.4 MixerState

```rust
#[derive(Clone)]
pub struct MixerState {
    pub crossfader:     RwSignal<f32>,   // 0.0 = full A, 1.0 = full B
    pub master_volume:  RwSignal<f32>,
    pub bpm_a:          RwSignal<Option<f64>>,
    pub bpm_b:          RwSignal<Option<f64>>,
    pub sync_master:    RwSignal<Option<DeckId>>,
}
```

---

## 8. Audio Engine

### 8.1 AudioContext Initialization

The Web Audio API requires a user gesture to create (or resume) an `AudioContext`. The context is created lazily on first user interaction (e.g., first "Load" or "Play" click). A single shared `AudioContext` is used for both decks — this is important because both decks must share the same time base for sync to work.

```rust
// Stored as a Leptos context provided from the root App component
let audio_ctx: Rc<RefCell<Option<AudioContext>>> = Rc::new(RefCell::new(None));
provide_context(audio_ctx.clone());

fn ensure_audio_context(ctx: &Rc<RefCell<Option<AudioContext>>>) -> AudioContext {
    let mut borrow = ctx.borrow_mut();
    if borrow.is_none() {
        *borrow = Some(AudioContext::new().expect("AudioContext failed"));
    }
    borrow.as_ref().unwrap().clone()
}
```

### 8.2 Audio Graph per Deck

```
[AudioBufferSourceNode]  ← .playbackRate for vinyl speed
         │
    [GainNode]           ← pre-FX gain / stutter gate
         │
  [BiquadFilter HIGH]    ← high shelf EQ
         │
  [BiquadFilter MID]     ← peaking EQ
         │
  [BiquadFilter LOW]     ← low shelf EQ
         │
  [BiquadFilter SWEEP]   ← low-pass/high-pass sweep filter
         │
  [ConvolverNode]        ← reverb IR (bypassed via dry/wet GainNodes)
         │
  [DelayNode]─────────[FeedbackGainNode]─┐  ← echo loop
         │◄─────────────────────────────┘
         │
    [GainNode]           ← channel volume fader
         │
         └──────────────► [crossfader blend] ──► [master GainNode] ──► destination
```

**Crossfader blend** uses two `GainNode`s (one per deck) controlled by a single crossfader signal:
- Deck A gain = `cos(crossfader * π/2)` (equal-power curve)
- Deck B gain = `sin(crossfader * π/2)`

### 8.3 Playback and Looping

`AudioBufferSourceNode` is a one-shot node — it cannot be restarted. The pattern is:

1. On **Play**: create a new `AudioBufferSourceNode`, set `.buffer`, set `.playbackRate.value`, connect to the graph, call `.start(0, offset)` where `offset` is the current track position.
2. On **Pause**: record the current offset (`ctx.current_time() - started_at + offset_at_play`), call `.stop()`, drop the node.
3. On **Loop**: do NOT use the built-in `.loop` property (it's inflexible with loop points). Instead, the rAF loop monitors `current_position` — when it reaches `loop_out`, it programmatically triggers a "seek to loop_in and play" cycle.

> **Why not use `source.loop`?** The Web Audio API's `loopStart`/`loopEnd` properties do work, but coordinating them with user-draggable loop points in the Leptos reactive system is cleaner when we control the re-schedule manually.

### 8.4 Speed Control (Vinyl Mode Only)

Pitch-preserving time stretch is not supported. Only vinyl-mode playback rate change is available: pitch follows speed exactly, like a real record.

```rust
// playbackRate range: 0.5 (−50%) to 2.0 (+100%), default 1.0
// Pitch fader signal value: -1.0 to +1.0
fn pitch_to_rate(fader: f32) -> f32 {
    if fader >= 0.0 { 1.0 + fader } else { 1.0 / (1.0 - fader) }
}
```

### 8.5 File Loading

```rust
// Triggered by <input type="file"> change event
async fn load_audio_file(
    file: File,
    deck: Rc<RefCell<AudioDeck>>,
    state: DeckState,
    ctx: AudioContext,
) {
    // 1. Read File → ArrayBuffer via FileReader (wrapped in a Future via Promise)
    // 2. ctx.decode_audio_data(&array_buffer) → Promise<AudioBuffer>
    // 3. Store AudioBuffer in AudioDeck
    // 4. Extract waveform peaks for canvas rendering (see 8.6)
    // 5. Update state.track_name, state.duration_secs
}
```

### 8.6 Waveform Peak Extraction

Done once on load. We extract a downsampled array of `(min, max)` peaks per pixel-column at the target canvas width (e.g., 800 samples → 800 column pairs). This is done synchronously from the decoded `AudioBuffer` float samples in WASM:

```rust
fn extract_peaks(buffer: &AudioBuffer, num_columns: usize) -> Vec<f32> {
    let channel_data = buffer.get_channel_data(0).unwrap(); // left channel
    let samples_per_col = channel_data.length() / num_columns;
    (0..num_columns).map(|i| {
        let start = i * samples_per_col;
        let end = (start + samples_per_col).min(channel_data.length());
        channel_data[start..end].iter().cloned().map(f32::abs).fold(0.0f32, f32::max)
    }).collect()
}
```

For stereo tracks, mix both channels before extracting peaks: `peak = max(abs(left[i]), abs(right[i]))`.

---

### 8.7 EQ Filter Configuration

The three-band EQ uses three `BiquadFilterNode`s with the following parameters. These are set on `AudioDeck` construction and updated reactively via Leptos `Effect`s.

| Band | Filter Type | Frequency | Q | Gain Range |
|---|---|---|---|---|
| High | `highshelf` | 8 000 Hz | — | −12 to +12 dB |
| Mid | `peaking` | 1 000 Hz | 0.7 | −12 to +12 dB |
| Low | `lowshelf` | 200 Hz | — | −12 to +12 dB |

> **Q = 0.7 for the mid peaking band** yields a broad, musical DJ-style bell curve — roughly ±1.5 octave bandwidth — wide enough to affect a whole vocal or instrument range without surgical narrowness.

The sweep filter (`filter_val` signal, range −1.0 to +1.0) dynamically switches filter type and frequency:

```rust
fn apply_sweep_filter(node: &BiquadFilterNode, filter_val: f32) {
    const BYPASS_THRESHOLD: f32 = 0.02;
    if filter_val.abs() < BYPASS_THRESHOLD {
        // Center = flat: use peaking at 0 dB gain
        node.set_type(BiquadFilterType::Peaking);
        node.gain().set_value(0.0);
    } else if filter_val < 0.0 {
        // Left: low-pass sweeps from 20 kHz (open) down to 200 Hz (closed)
        let t = 1.0 + filter_val; // remaps [-1,0] → [0,1]
        let freq = 200.0_f32 + t * (20_000.0 - 200.0);
        node.set_type(BiquadFilterType::Lowpass);
        node.frequency().set_value(freq);
        node.q().set_value(0.5);
    } else {
        // Right: high-pass sweeps from 20 Hz (open) up to 2 000 Hz (closed)
        let freq = 20.0_f32 + filter_val * (2_000.0 - 20.0);
        node.set_type(BiquadFilterType::Highpass);
        node.frequency().set_value(freq);
        node.q().set_value(0.5);
    }
}
```

---

### 8.8 Reverb — Procedural IR Generation

Rather than loading an impulse response audio file (licensing, asset bundling), a synthetic IR is generated at runtime using **exponentially decaying stereo white noise**. This approach is used by classic algorithmic reverbs and produces convincing room tails.

**Algorithm:**
```
IR[channel][t] = white_noise() × exp(−decay × t / duration_secs)
```

Where `white_noise()` ∈ [−1.0, 1.0] from an LCG, L and R channels use different PRNG seeds for stereo width.

```rust
fn generate_reverb_ir(ctx: &AudioContext, duration_secs: f32, decay: f32) -> AudioBuffer {
    let sample_rate = ctx.sample_rate();
    let num_samples = (sample_rate * duration_secs) as usize;
    let ir = ctx.create_buffer(2, num_samples as u32, sample_rate).expect("create_buffer");

    for channel in 0..2_u32 {
        // Different LCG seed per channel → stereo decorrelation
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
                let noise = (state >> 33) as f32 / (u32::MAX as f32) * 2.0 - 1.0;
                let t = i as f32 / num_samples as f32;
                noise * (-decay * t * duration_secs).exp()
            })
            .collect();
        let js_arr = js_sys::Float32Array::from(samples.as_slice());
        ir.copy_to_channel(&js_arr, channel as i32).expect("copy_to_channel");
    }
    ir
}
```

**Parameter guide:**

| `duration_secs` | `decay` | Character |
|---|---|---|
| 0.5 | 4.0 | Tight drum room |
| 1.2 | 2.5 | Medium hall |
| 2.5 | 1.5 | Large concert hall |
| 3.5 | 0.8 | Cathedral / dense wash |

The IR is regenerated when the user changes reverb parameters in Settings. **Dry/wet bypass** avoids clicks by using parallel `GainNode`s and `linearRampToValueAtTime` over 20 ms instead of disconnecting the `ConvolverNode`:

```
[source] ────────────────────────────────────────► [dry GainNode] ──► [sum] → ...
   │                                                                      ▲
   └──► [ConvolverNode] ──► [wet GainNode] ───────────────────────────►──┘
```

When reverb is off: `wet = 0.0`, `dry = 1.0`. Transitions ramp over 20 ms.

---

### 8.9 Echo / Delay

A feedback delay loop using standard Web Audio nodes.

```
[source] ─────────────────────────────────────────────► [output]
   │                                                       ▲
   └──► [DelayNode] ──► [wet GainNode] ──────────────────►┤
              │                                            │
              └──► [feedback GainNode] ──► (back to DelayNode input)
```

```rust
pub struct EchoNodes {
    pub delay:    DelayNode,   // maxDelayTime = 2.0s; delayTime controlled by user
    pub feedback: GainNode,    // gain = 0.0–0.85 (hard cap; > 0.85 risks runaway)
    pub wet:      GainNode,    // 0.0 (off) to 0.8 (max mix)
    pub dry:      GainNode,    // typically 1.0
}
```

**Key constraint:** Feedback gain must remain strictly below 1.0. The signal path forms a real feedback loop in the audio graph; values ≥ 1.0 cause exponential amplitude growth and browser audio context suspension.

**User-facing parameters:**
- `time`: `delay.delay_time().set_value(seconds)` — repeat interval (0.05–2.0 s)
- `feedback`: `feedback.gain().set_value(v)` — 0 = single echo, 0.85 = many fading repeats

---

### 8.10 Flanger

A flanger modulates a short delay time (0.5–10 ms) with a slow sine-wave LFO, creating a sweeping comb-filter/phase-shift effect. All nodes are standard Web Audio API.

```
[source] ──────────────────────────────────────────► [dry GainNode] ──► [output]
   │                                                                        ▲
   └──► [DelayNode (0.5–10 ms)] ──► [wet GainNode] ────────────────────►──┘
              ▲           │
              │         [feedback GainNode] ──► (back to DelayNode input)
        [depth GainNode]
              ▲
       [OscillatorNode — sine, 0.1–2 Hz]
```

```rust
pub struct FlangerNodes {
    pub delay:     DelayNode,      // maxDelayTime = 0.02s; base delay = 0.005s (5ms)
    pub lfo:       OscillatorNode, // type = Sine; frequency = 0.1–2.0 Hz
    pub lfo_depth: GainNode,       // scales LFO to delay modulation depth (0–0.005 s)
    pub feedback:  GainNode,       // 0.0–0.7 (higher = metallic resonance)
    pub wet:       GainNode,
}

fn build_flanger(ctx: &AudioContext) -> FlangerNodes {
    let delay = ctx.create_delay_with_max_delay_time(0.02).unwrap();
    delay.delay_time().set_value(0.005); // 5 ms center

    let lfo = ctx.create_oscillator().unwrap();
    lfo.set_type(OscillatorType::Sine);
    lfo.frequency().set_value(0.5); // 0.5 Hz sweep

    let lfo_depth = ctx.create_gain().unwrap();
    lfo_depth.gain().set_value(0.003); // ±3 ms depth

    // LFO → depth scaler → DelayNode.delayTime AudioParam
    lfo.connect_with_audio_node(&lfo_depth).unwrap();
    lfo_depth.connect_with_audio_param(&delay.delay_time()).unwrap();
    lfo.start().unwrap();
    // ...
}
```

**User parameters:**
- `rate`: LFO frequency (0.1–2 Hz) — how fast the sweep cycles
- `depth`: `lfo_depth.gain` value (0–0.008 s) — width of the sweep
- `feedback`: 0 = light shimmery flange; 0.7 = harsh jet-engine sound

---

### 8.11 Stutter Effect

The stutter effect gates audio in a tempo-synced rhythmic pattern. It uses the Web Audio API's **sample-accurate parameter scheduling** via `AudioParam.setValueAtTime` — far more precise than `setInterval`.

```rust
/// Pre-schedule a repeating gate pattern on a GainNode for `bars` bars.
/// Call this rolling — reschedule before the previous window expires.
fn schedule_stutter(
    gate: &GainNode,
    start_time: f64,    // AudioContext.currentTime
    bpm: f64,
    subdivision: f64,   // 4.0 = quarter, 8.0 = eighth, 16.0 = sixteenth notes
    duty: f64,          // fraction of gate period that is "open" (0.0–1.0)
    bars: f64,
) {
    let beat_dur    = 60.0 / bpm;
    let gate_period = beat_dur * 4.0 / subdivision;
    let gate_open   = gate_period * duty;
    let end_time    = start_time + bars * beat_dur * 4.0;

    let mut t = start_time;
    while t < end_time {
        gate.gain().set_value_at_time(1.0, t).unwrap();
        gate.gain().set_value_at_time(0.0, t + gate_open).unwrap();
        t += gate_period;
    }
}
```

**Lifecycle:**
1. On enable: `schedule_stutter(gate, ctx.current_time(), bpm, subdiv, 0.5, 4.0)` for 4 bars.
2. Before expiry, reschedule the next 4 bars (rolling lookahead).
3. On disable: `gate.gain().cancel_scheduled_values(0.0)` then ramp `gain` to 1.0 over 5 ms.
4. On subdivision change: cancel and reschedule from the next beat boundary.

**Subdivision options exposed in the FX panel:** 1/4, 1/8, 1/16, 1/32.

---

### 8.12 Scratch Simulation

Scratch simulation maps platter angular velocity to `AudioBufferSourceNode.playbackRate` using
a non-linear (square-root) compressive curve. True `playbackRate < 0` is not supported by
Web Audio API, so backward drag is implemented by swapping to a **pre-computed reversed
`AudioBuffer`** that was created on file load.

```rust
// Key constants (all in deck_audio.rs)
const SCRATCH_SENSITIVITY: f64 = 0.9;   // sqrt-curve scale factor
const SCRATCH_RATE_MAX:    f32 = 3.5;   // max playback rate
const SCRATCH_SMOOTH_SECS: f64 = 0.012; // 12 ms ramp for forward-phase smoothing

// Non-linear rate mapping: sqrt compresses high-velocity gestures
fn scratch_rate_nonlinear(normalized_vel: f64) -> f32 {
    // normalized_vel = |d_angle| / dt / (TAU * 0.55)
    // 1.0 = 33 RPM reference speed → rate ≈ 1.21
    (normalized_vel.sqrt() * SCRATCH_SENSITIVITY) as f32
        .clamp(0.0, SCRATCH_RATE_MAX)
}

// Fields added to AudioDeck for reverse scratch support
pub reversed_buffer:      Option<AudioBuffer>,  // pre-computed on file load
pub scratch_in_reverse:   bool,                 // true while using reversed buffer
pub scratch_position_secs: f64,                 // integrated track position (seconds)
pub scratch_was_playing:  bool,                 // deck was playing at scratch_start?
```

**On `scratch_move(angle, time)`:**
- Detects direction sign change (d_angle < 0 = backward)
- If changing fwd→rev and `reversed_buffer` is `Some`: stops current source, starts reversed buffer at `duration - scratch_position_secs`
- If changing rev→fwd: stops current source, starts forward buffer at `scratch_position_secs`
- Same direction: updates `playbackRate` only (12 ms ramp if forward, immediate if reverse)
- Integrates `scratch_position_secs` using `scratch_in_reverse` (actual buffer state) not
  `going_reverse` (user intent), so position stays correct when no reversed buffer is loaded

**On `scratch_end()`:**
- Always stops the source (which may be the reversed buffer)
- If `scratch_was_playing`: calls `play(scratch_position_secs, pre_scratch_rate)` to resume
- If paused: sets `offset_at_play = scratch_position_secs` to anchor waveform position

**On file load (`loader.rs`):**
- After `deck.buffer = Some(audio_buffer)`, calls `compute_reversed_buffer(&ctx, &audio_buffer)`
- Non-fatal: if computation fails (e.g. OOM), logs a warning and scratch falls back to
  forward-only behaviour

---

### 8.13 VU Meter — Level Analysis

The `AnalyserNode` sits just before the channel `GainNode` (post-EQ, post-FX). Each rAF frame, RMS level is computed from time-domain data and written to a Leptos signal that drives the meter `<div>` height.

```rust
fn read_vu_level(analyser: &AnalyserNode) -> f32 {
    let n = analyser.fft_size() as usize; // set to 256 — ≈5.8 ms at 44.1 kHz
    let mut buf = vec![0.0f32; n];
    analyser.get_float_time_domain_data(&mut buf);

    // RMS
    let rms = (buf.iter().map(|&s| s * s).sum::<f32>() / n as f32).sqrt();

    // dBFS, range −60..0; map to 0.0..1.0 for display
    let db = (20.0 * rms.max(1e-6_f32).log10()).max(-60.0);
    (db + 60.0) / 60.0
}
```

`AnalyserNode` initial configuration:
```rust
analyser.set_fft_size(256);                 // small FFT = low overhead in rAF
analyser.set_smoothing_time_constant(0.8);  // 0.8 = smooth bounce, not jumpy
```

---

### 8.14 BPM Detection Algorithm (Implemented in M4)

BPM detection runs automatically on file load. The algorithm uses **spectral flux onset detection** (restricted to the kick/bass frequency band) followed by **autocorrelation tempo estimation**, computed once against the decoded `AudioBuffer` PCM data.

#### Phase 1 — Spectral Flux Onset Strength

1. Frame the signal with a **1024-sample Hanning window**, 512-sample hop (50% overlap).
2. **FFT each frame** using `rustfft` (WASM-compatible).
3. Restrict analysis to **bins 1–32 (~43–1400 Hz)** — the kick drum and bass range. This excludes hi-hat, guitar, and melodic content that create sub-beat autocorrelation peaks.
4. For each frame `t`, compute **half-wave-rectified spectral flux**:
   ```
   flux[t] = Σ_k  max(0,  |X_t[k]| − |X_{t−1}[k]|)
   ```
5. **Smooth** the flux curve with a 5-frame centred moving average.

#### Phase 2 — Autocorrelation Tempo Estimation

6. Compute **autocorrelation** of the flux signal over lags corresponding to BPM 60–200.
7. Find **dominant lag** = `argmax r[lag]`.
8. **Sub-lag correction** (threshold 0.90): if `r[lag/2] ≥ 0.90 × r[lag]`, prefer the half-lag (doubles the BPM). This corrects integer-alignment artefacts where the double-period scores marginally higher than the true period. Threshold is set conservatively at 0.90 to avoid false-triggering on sub-harmonic rhythmic content in real music.
9. Convert to BPM and clamp to [60, 200].

#### Known Limitations

The algorithm is well-suited to **electronic music with clear kick drums** (house, techno, drum & bass, hip-hop). Known failure modes:

- **Syncopated funk/soul**: bass hits on off-beats create stronger sub-beat periodicities than the quarter-note pulse. Autocorrelation may lock onto a 3-beat or dotted-quarter period.
- **Orchestral / acoustic without a kick drum**: low-frequency flux is dominated by string and brass swells rather than a percussive beat, causing 2× errors.

TAP BPM is the intended manual override for these cases. Future improvement would require a multi-band flux + majority-vote approach or a comb-filter resonator.

#### Testing with Real Audio Files

Integration tests (`tests/bpm_real_tracks.rs`) decode real MP3 files natively using the `symphonia` crate (dev-dependency) and feed the resulting PCM directly into `compute_spectral_flux` + `estimate_bpm`. This exercises the same Rust code path that runs in the browser after `AudioContext.decodeAudioData`, with the only difference being the MP3 decoder (native vs browser — both produce equivalent PCM).

Test fixtures live in `tests/data/audio/`:

| File | Genre | Expected BPM | Status |
|---|---|---|---|
| `Cephalopod.mp3` | Jazz-funk | ~125 | ✅ passes |
| `Scheming Weasel faster.mp3` | Fast comedy | ~167 | ✅ passes |
| `Funkorama.mp3` | Funk | ~101 | `#[ignore]` — syncopated bass hard case |
| `Killers.mp3` | Orchestral | ~105 | `#[ignore]` — no kick drum |

All fixtures are CC BY 4.0 (Kevin MacLeod, incompetech.com). See `tests/data/audio/ATTRIBUTION.txt` and `bpm_ground_truth.json`. The two hard-case tests are retained as regression baselines for future algorithm improvements.

#### Crate requirements

```toml
# production
rustfft = { version = "6", default-features = false }

# dev only (native integration tests)
symphonia = { version = "0.5", features = ["mp3"], default-features = false }
```

**Computational budget:** A 4-minute track at 44 100 Hz produces ~10 200 frames of size 1024. At ~10 μs per FFT in WASM, total cost ≈ 100–200 ms. Acceptable as a one-time on-load computation.

---

### 8.15 Pitch-Preserving Time Stretch — **Dropped from Roadmap**

Pitch-preserving time stretch requires an `AudioWorklet` running WASM for real-time DSP. `AudioWorklet` with WASM needs `SharedArrayBuffer`, which requires `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` HTTP headers. These have been ruled out to keep static hosting simple.

**Only vinyl-mode speed change ships.** Pitch always follows playback rate. The `DeckState.playback_rate` signal and the vinyl `pitch_to_rate()` function in §8.4 are the complete speed-change implementation.

The WSOLA and Phase Vocoder algorithm descriptions have been removed to avoid dead design weight in the spec.

---

## 9. Canvas Rendering Architecture

### 9.1 Design Principle

Canvas rendering runs in a **single shared `requestAnimationFrame` loop** started on app mount. Each frame, the loop:
1. Reads current state from Leptos signals (`.get_untracked()` to avoid reactivity overhead).
2. Clears and redraws each canvas element.

This avoids the "reactive per-pixel update" anti-pattern that would be catastrophically slow.

### 9.2 rAF Loop Setup

```rust
// src/canvas/raf_loop.rs
pub fn start_raf_loop(
    deck_a_state: DeckState,
    deck_b_state: DeckState,
    deck_a_audio: Rc<RefCell<AudioDeck>>,
    deck_b_audio: Rc<RefCell<AudioDeck>>,
    platter_a_ref: NodeRef<html::Canvas>,
    platter_b_ref: NodeRef<html::Canvas>,
    waveform_a_ref: NodeRef<html::Canvas>,
    waveform_b_ref: NodeRef<html::Canvas>,
) {
    // Recursive closure using Rc<RefCell<Option<Closure>>>
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        // Update currentTime signal from AudioContext
        update_current_time(&deck_a_audio, &deck_a_state);
        update_current_time(&deck_b_audio, &deck_b_state);

        // Check loop boundaries
        check_loop(&deck_a_audio, &deck_a_state);
        check_loop(&deck_b_audio, &deck_b_state);

        // Draw canvases
        draw_platter(&platter_a_ref, &deck_a_state);
        draw_platter(&platter_b_ref, &deck_b_state);
        draw_waveform(&waveform_a_ref, &deck_a_state);
        draw_waveform(&waveform_b_ref, &deck_b_state);

        // Schedule next frame
        window().request_animation_frame(
            f.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap();
    }));

    window().request_animation_frame(
        g.borrow().as_ref().unwrap().as_ref().unchecked_ref()
    ).unwrap();
}
```

### 9.3 Platter Rendering

The platter is drawn on a `<canvas>` element each frame. It represents a spinning vinyl record with cartoon styling:

- Background: dark circle with concentric groove rings drawn in CSS-like arcs.
- A "label" circle in the center with the track name (abbreviated).
- A rotation angle is computed from elapsed playback time and playback rate.
- A tonearm drawn as a rotated line segment pivoting from the top-right.

```rust
fn draw_platter(canvas_ref: &NodeRef<html::Canvas>, state: &DeckState) {
    let canvas = canvas_ref.get_untracked().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

    let current = state.current_secs.get_untracked();
    let rate = state.playback_rate.get_untracked();
    let playing = state.is_playing.get_untracked();

    // 33 RPM ≈ 0.55 rotations/sec
    let rotations_per_sec = 0.55 * rate;
    let angle = if playing { current * rotations_per_sec * std::f64::consts::TAU } else { 0.0 };

    // draw vinyl record, grooves, label, tonearm...
}
```

### 9.4 Waveform Rendering

Rendered in two passes per frame:
1. **Static waveform** (only redrawn when track changes) — drawn from `waveform_peaks` signal into an offscreen canvas, then composited.
2. **Dynamic overlay** (every frame) — draws the playhead line, loop region highlight, and hot cue markers.

---

## 10. Component Architecture

### 10.1 Component Tree

```
<App>
 ├── <Header>                  (logo, nav links using fragment router)
 ├── <DeckView>                (main page — Route::Main)
 │    ├── <Deck side=Left>
 │    │    ├── <TrackLabel>
 │    │    ├── <Waveform>      (canvas, NodeRef)
 │    │    ├── <Platter>       (canvas, NodeRef)
 │    │    ├── <Controls>      (play/cue/stop/nudge)
 │    │    ├── <PitchFader>
 │    │    ├── <LoopControls>
 │    │    ├── <HotCues>
 │    │    ├── <EQ>
 │    │    └── <FxPanel>
 │    ├── <Mixer>
 │    │    ├── <Crossfader>
 │    │    ├── <ChannelFaders>
 │    │    ├── <MasterVolume>
 │    │    └── <BpmSync>
 │    └── <Deck side=Right>
 │         └── (mirror of Left)
 ├── <SettingsView>            (Route::Settings)
 ├── <AboutView>               (Route::About)
 └── <HelpView>                (Route::Help — quick-start guide, purely presentational)
```

### 10.2 Deck Component Props

```rust
#[component]
pub fn Deck(
    side: DeckSide,                          // Left | Right
    state: DeckState,                        // reactive signals
    audio: Rc<RefCell<AudioDeck>>,           // audio graph
    platter_ref: NodeRef<html::Canvas>,
    waveform_ref: NodeRef<html::Canvas>,
) -> impl IntoView { ... }
```

---

## 11. Keyboard Shortcuts

Registered globally via a `keydown` listener on `window` at app mount. The handler reads current signals to determine deck state and dispatches actions:

```rust
// src/utils/keyboard.rs
pub fn register_keyboard_shortcuts(deck_a: DeckState, deck_b: DeckState) {
    let _listener = gloo_events::EventListener::new(&window(), "keydown", move |e| {
        let e = e.dyn_ref::<KeyboardEvent>().unwrap();
        // Skip if focus is on an input element
        if is_input_focused() { return; }
        match e.code().as_str() {
            "Space"  => toggle_play(&deck_a),
            "Enter"  => toggle_play(&deck_b),
            "KeyQ"   => cue(&deck_a),
            "KeyP"   => cue(&deck_b),
            // ... etc
            _ => {}
        }
    });
}
```

---

## 12. CSS and Visual Design

All styling lives in `static/style.css`. No CSS-in-Rust, no Tailwind (to keep the build simple). The cartoon aesthetic is achieved via:

- **Black outline borders**: `border: 3px solid #111; box-shadow: 3px 3px 0 #111;`
- **Bold flat colors**: CSS custom properties (`--color-deck-a: #3b82f6; --color-deck-b: #f97316;`)
- **Rounded shapes**: `border-radius: 12px` on panels, `50%` on knobs/buttons.
- **Keyframe animations**:
  - `.button-press { animation: pop 80ms ease-out; }` — button bounce on click
  - `.hot-cue-burst { animation: burst 200ms ease-out; }` — star burst on hot cue trigger
  - `.fx-active { animation: wiggle 0.4s infinite; }` — active FX indicator wiggle
- **Canvas platter** provides the "spinning record" — CSS handles everything else.
- **VU meter**: A `<div>` with height driven by a Leptos signal updated from `AnalyserNode` data (sampled in the rAF loop, written to a signal, rendered reactively). The update rate from rAF is ~60fps — fine for a VU meter.

> **VU meter is the one intentional exception to the "rAF updates non-reactively" rule.** Its value is written to a signal in the rAF loop and rendered by Leptos. The signal update triggers a reactive DOM update to the meter `<div>` height style. This is acceptable because the element update is minimal (one style property) and avoids a second canvas.

---

## 13. Feasibility Assessment

### ✅ Straightforward

| Area | Why |
|---|---|
| CSR Leptos app with Trunk | Well-trodden path; many examples exist |
| Web Audio API via web-sys | All required nodes are in web-sys 0.3.x |
| Vinyl-mode speed control | Single `playbackRate` property on `AudioBufferSourceNode` |
| File loading + decoding | FileReader + `decodeAudioData` — standard Web Audio API |
| Fragment routing | ~30 lines of Rust |
| Static deployment | Trunk produces a self-contained `dist/` |

### ⚠️ Moderate Complexity

| Area | Why | Mitigation |
|---|---|---|
| rAF loop + Leptos signals sharing | Two execution contexts sharing state | Use `get_untracked()` in rAF; only write to signals from rAF for display-rate values |
| Manual loop re-scheduling | AudioBufferSourceNode is one-shot | Track `started_at` + `offset`, recreate source node on each loop restart |
| Canvas waveform rendering | Peak extraction in WASM + 2-pass draw | Do peak extraction once on load; cache in offscreen canvas |
| Reverb IR generation | Procedural IR must sound good | LCG stereo noise + exp decay; parameterised in Settings (see §8.8) |
| Stutter gate scheduling | Must be sample-accurate; setInterval is not | Use `AudioParam.setValueAtTime` pre-scheduling (see §8.11) |
| Flanger LFO modulation | AudioParam modulation via OscillatorNode | Standard Web Audio graph (see §8.10) |
| BPM auto-detection on load | Spectral flux + autocorrelation in WASM | Runs once post-decode; ~100-200 ms; `rustfft` crate; no AudioWorklet (see §8.14) |
| Crossfader equal-power curve | Math | `cos/sin` of crossfader value × π/2 |

### ❌ Dropped / Out of Scope

| Area | Reason |
|---|---|
| Pitch-preserving time stretch (WSOLA / Phase Vocoder) | **Dropped from roadmap.** Requires AudioWorklet + COOP/COEP headers. Vinyl-mode only. |
| AudioWorklet (WASM) | **Dropped.** Requires COOP/COEP headers which complicate static hosting. |
| True reverse scratch | ~~playbackRate < 0 unsupported~~ — **Implemented (Feature 001)** via pre-reversed buffer swap in `scratch_move()` |
| Recording / export | `MediaRecorder` API — not planned |
| `localStorage` persistence | Hot cues and loop points reset every session by design |

---

## 14. Development Workflow

```bash
# Install tools
cargo install trunk
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli  # version must match wasm-bindgen crate version

# Dev server (hot reload)
trunk serve

# Production build
trunk build --release

# Output: dist/
```

**Dev server** runs at `http://localhost:8080` by default. Trunk watches for Rust source changes and recompiles.

---

## 15. Build Optimization Notes

- `opt-level = "z"` + `lto = true` in release profile minimize WASM binary size.
- Trunk automatically runs `wasm-opt` (binaryen) in release mode for further shrinkage.
- Expected release binary size: **2–5 MB** WASM (mostly from Leptos + web-sys code). Acceptable for a web app.
- Audio data (loaded by user at runtime) never goes in the binary.
- Reverb IR (if bundled): keep it under 200 KB; use a short `include_bytes!()` embedded constant.

---

## 16. Resolved Decisions

All open questions have been closed.

| # | Question | Decision |
|---|---|---|
| 1 | Stutter effect implementation | **Resolved**: `AudioParam.setValueAtTime` pre-scheduling with rolling lookahead. See §8.11. |
| 2 | Flanger implementation | **Resolved**: `OscillatorNode` LFO → depth `GainNode` → `DelayNode.delayTime`. See §8.10. |
| 3 | Reverb IR | **Resolved**: procedural exponentially decaying stereo white noise. See §8.8. |
| 4 | Font choice | **Resolved**: **Bangers** (SIL OFL), self-hosted woff2 in `static/fonts/`. |
| 5 | Mobile / touch support | **Resolved**: responsive-friendly CSS only; no touch gestures in v1. |
| 6 | COOP/COEP headers | **Resolved**: not configuring — hosting stays simple. AudioWorklet features dropped from roadmap. |
| 7 | Pitch-preserving time stretch | **Dropped from roadmap entirely.** Vinyl-mode only (pitch follows rate). |
| 8 | Hot cue / loop point persistence | **No persistence.** Resets every session — no `localStorage`. |
| 9 | Canvas rendering approach | **Raw `web-sys` canvas calls.** No extra crate. |
| 10 | Minimum browser support | **Latest Chrome, Firefox, Safari.** No polyfills. |
| 11 | BPM auto-detection | **In v1.** Runs once on load in WASM (no AudioWorklet). See §8.14. |

---

## 17. DSP Crate Landscape

This section documents the Rust audio DSP crate ecosystem and explains what is (and is not) recommended for this project.

### 17.1 WASM-Compatible Crates

| Crate | Version | WASM? | Role | Notes |
|---|---|---|---|---|
| `web-sys` | 0.3.x | ✅ Required | Web Audio API node bindings | Already in v1; all EQ/reverb/delay/flanger nodes come from here |
| `rustfft` | 6.x | ✅ Yes (WASM SIMD opt-in) | FFT for onset detection, phase vocoder | Disable `avx`/`sse` features for WASM (`default-features = false`); `wasm_simd` feature is optional — only useful if COOP/COEP headers are served |
| `dasp` | 0.11.x | ✅ Yes (`no_std` available) | DSP primitives: sample types, interpolation, ring buffers, signal iterators | Useful for WSOLA overlap-add buffers and sinc interpolation in sample rate conversion |
| `biquad` | 0.4.x | ✅ Yes | Biquad IIR filter coefficient computation | Only needed if computing custom filter coefficients in WASM; for v1 all EQ uses Web Audio `BiquadFilterNode` directly |
| `fundsp` | 0.23.x | ✅ Yes (`no_std`, disable `files`/`fft` features) | Comprehensive audio DSP graph library — oscillators, filters, delays, reverbs | Graph notation is elegant for DSP algorithm prototyping; heavy dependency for what we need in v1; consider for v2 effects modules |

### 17.2 Crates Not Suitable for this Project

| Crate | Reason |
|---|---|
| `cpal` | Native audio I/O (ALSA, CoreAudio, WASAPI). Not applicable — the browser's Web Audio API owns output |
| `rodio` | High-level playback built on `cpal`. Same issue |
| `symphonia` | Audio file decoding (MP3, FLAC, etc.). Unnecessary — `AudioContext.decodeAudioData()` handles this natively |
| `aubio-sys` | Rust bindings to the `aubio` C library (onset/BPM detection). Requires a C toolchain and complex Emscripten WASM compilation; not worth the overhead given a pure-Rust spectral flux approach |
| `rubberband` | Bindings to the Rubber Band Library C++ time-stretcher. High quality but LGPL-licensed, requires complex WASM compilation via Emscripten; WSOLA in pure Rust is preferred |
| `hound` | WAV file I/O. Not needed; Web Audio handles decode |

### 17.3 Cargo.toml Additions (v1)

```toml
# BPM auto-detection (§8.14) — spectral flux + autocorrelation, runs once on file load
# Disable native SIMD features — not applicable to WASM; avoids binary size bloat
rustfft = { version = "6", default-features = false }
```

> **Binary size impact:** `rustfft` with `default-features = false` adds roughly 100–200 KB to the WASM binary. Acceptable given the 2–5 MB baseline.

`dasp` is not needed — WSOLA is dropped and Web Audio API handles all real-time DSP.

---

*This document evolves alongside the product spec. Architectural decisions made here supersede any conflicting guidance in the product spec.*
