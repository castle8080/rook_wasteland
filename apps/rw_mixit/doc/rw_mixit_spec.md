# rw_mixit — Product Requirements Specification

**Project:** rw_mixit  
**Status:** Draft  
**Version:** 0.1.0

---

## 1. Overview

`rw_mixit` is a browser-based, old-school DJ mixing tool built with **Rust** and **Leptos**, compiled to **WebAssembly**. It runs entirely in the browser with no server required — all audio processing and UI logic executes client-side. The aesthetic draws from the golden era of hip-hop and turntablism: chunky cartoon-style graphics, animated platters, glowing VU meters, and tactile-feeling controls that evoke the look and feel of real vinyl DJ equipment reimagined as a Saturday-morning cartoon.

The application is intended for hobbyists, music enthusiasts, beatmakers, and anyone who wants to play with audio in a fun, tactile, and visually entertaining way.

---

## 2. Goals

- Provide a dual-deck audio mixing experience entirely in the browser.
- Allow users to load local audio files without any upload to a server.
- Offer real-time audio manipulation: speed, pitch, looping, and effects.
- Deliver a visually rich, animated cartoon-style UI that feels alive.
- Keep the technology stack focused: Rust + Leptos (WASM) + Web Audio API.

---

## 3. Non-Goals

- No server-side audio processing.
- No user accounts or cloud storage.
- No streaming or network audio sources (for now).
- Not a full DAW — deliberately limited and fun-focused.

---

## 4. Technology Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| UI Framework | Leptos (reactive, SSR-optional) |
| Compilation Target | WebAssembly (wasm32-unknown-unknown) |
| Audio Engine | Web Audio API (via `web-sys`) |
| Styling | CSS with keyframe animations |
| Build Tool | Trunk |

---

## 5. Core Concepts

### 5.1 The Two Decks

The application presents **two virtual turntable decks**, side by side — Deck A (left) and Deck B (right). Each deck is an independent unit capable of loading and playing one audio track. The decks share a central **mixer panel** between them.

Each deck visually resembles a cartoon turntable: a spinning platter with a vinyl record graphic, a tonearm that tracks across the record, and a panel of controls below it.

### 5.2 The Mixer Panel

The central mixer panel bridges the two decks. It contains:
- A **crossfader** — a horizontal slider that smoothly blends audio from Deck A to Deck B.
- **Channel faders** — individual vertical volume faders for each deck.
- A **master volume** knob.
- A **BPM display** area showing detected or manually set BPM for each deck.

---

## 6. Features

### 6.1 Audio File Loading

- Each deck has a **"Load Track"** button styled as a cartoon record crate or tape slot.
- Clicking it opens the browser's native file picker, restricted to common audio formats: `.mp3`, `.wav`, `.ogg`, `.flac`, `.aac`.
- Once loaded, the track name is displayed on the deck's label area (like a record label sticker).
- Audio is decoded in-browser using the Web Audio API's `decodeAudioData`.

### 6.2 Playback Controls

Each deck includes:

| Control | Description |
|---|---|
| **Play / Pause** | Starts or pauses playback. Animated platter spins when playing. |
| **Stop** | Stops playback and resets the playhead to the beginning. |
| **Cue** | Sets/jumps to a cue point (default: start of track). Hold to preview. |
| **Nudge (–/+)** | Temporarily speeds up or slows down to sync with the other deck. |

### 6.3 Speed / Pitch Control

- Each deck has a **pitch/tempo slider** (the "pitch fader"), styled as a chunky cartoon lever.
- Range: **-50% to +50%** of original BPM/speed.
- **Vinyl-style only**: pitch changes with speed, exactly like slowing or speeding a real record. Pitch-preserving time-stretch is not supported.
- Visual feedback: the platter animation speed changes in real time to match the adjusted tempo.

### 6.4 Loop Controls

- Users can define a **loop region** on each deck:
  - **Loop In**: Sets the start point of the loop.
  - **Loop Out**: Sets the end point and activates looping immediately.
  - **Loop Toggle**: Enables or disables the active loop without clearing it.
  - **Loop Length Shortcuts**: Buttons for quick loop lengths — ½ bar, 1 bar, 2 bars, 4 bars, 8 bars — based on detected BPM.
- The active loop region is highlighted visually on the **waveform display**.
- Loop points can be dragged on the waveform to adjust.

### 6.5 Waveform Display

- Each deck displays a **scrolling waveform** of the loaded audio.
- A playhead line shows the current position.
- The waveform is rendered on a `<canvas>` element.
- The loop region is highlighted with a translucent color overlay.
- Clicking anywhere on the waveform seeks to that position (while playing or paused).
- **Drag scrub**: pressing and dragging horizontally on the waveform continuously scrubs the playhead position. The cursor changes to a grab/grabbing style to indicate the interaction.
- **Zoom controls** allow zooming in/out of the waveform for fine loop-point editing.

### 6.6 Hot Cues

- Each deck supports **4 hot cue buttons** (colored: red, blue, green, yellow).
- **Setting**: Hold a color button to set a hot cue at the current playhead position.
- **Jumping**: Tap a set hot cue button to instantly jump to that position.
- **Clearing**: Double-tap (or right-click) to clear a hot cue.
- Hot cues are displayed as colored markers on the waveform.

### 6.7 EQ and Filter

Each deck channel includes a **3-band EQ** (cartoon-style rotary knobs):
- **High** — treble frequencies (~8 kHz+)
- **Mid** — midrange frequencies (~500 Hz–4 kHz)
- **Low** — bass frequencies (~200 Hz and below)

Plus a **Filter knob** that sweeps a low-pass/high-pass filter (center = flat, left = low-pass, right = high-pass). This is great for classic DJ filter sweeps.

### 6.8 Effects (FX)

A simple effects panel per deck with toggle buttons for:

| Effect | Description |
|---|---|
| **Echo / Delay** | Repeating echo effect, with adjustable feedback and time controls. |
| **Reverb** | Room/hall reverb for atmospheric effect. |
| **Flanger** | Sweeping comb-filter modulation. |
| **Stutter** | Rapidly gates the audio for a choppy rhythmic effect. |
| **Scratch (Sim)** | Simulated vinyl scratch effect triggered by mouse drag on the platter. |

Effects have simple on/off toggles plus one or two parameter knobs each, all styled as cartoon arcade buttons and dials.

### 6.9 BPM Detection

- On file load, the app attempts **automatic BPM detection** using onset analysis.
- The detected BPM is shown on the deck; the user can manually tap a **TAP BPM** button to override it.
- BPM values drive the loop length shortcut calculations.

### 6.10 Sync

- A **SYNC button** on each deck snaps that deck's playback speed to match the other deck's BPM, enabling easy beat matching.
- A **MASTER** indicator shows which deck is the tempo master.

---

## 7. Visual Design

### 7.1 Aesthetic

The UI must feel like a **Saturday-morning cartoon** or a comic book brought to life:
- Bold black outlines on all UI elements (cartoon "inking" style).
- Bright, saturated primary colors: cobalt blue, fire orange, lime green, hot pink, yellow.
- Slightly rounded, "puffy" shapes — no sharp corporate corners.
- Subtle drop shadows that give elements a 3D-ish comic book feel.
- Hand-drawn-inspired typefaces for labels and titles.

### 7.2 Animations

Animations are a first-class citizen:

| Element | Animation |
|---|---|
| **Platter** | Continuously rotates while playing; slows/speeds with pitch fader; stops gradually when playback is paused or stopped. |
| **Tonearm** | Slowly pivots across the platter over the track's duration. |
| **VU Meters** | Animated bar graphs bouncing to the audio level on each channel. |
| **Crossfader** | Glows and pulses when moved. |
| **Load button** | Bounces when hovered; record "slots in" with an animation on load. |
| **Loop active** | Loop region pulses with a heartbeat glow on the waveform. |
| **Hot cue trigger** | Button flashes and emits a cartoon "burst" star graphic on trigger. |
| **FX on** | Active FX buttons wiggle or glow. |

### 7.3 Layout

```
┌─────────────────────────────────────────────────────────────┐
│                      [ rw_mixit logo ]                      │
│         [Settings]  [Help]  [About]   ← header nav          │
├──────────────────────┬──────────────┬───────────────────────┤
│      DECK  A         │   MIXER      │       DECK  B         │
│  [  waveform A  ]    │  [xfader]    │  [  waveform B  ]     │
│  [ platter anim ]    │  [vol A][B]  │  [ platter anim ]     │
│  [play][cue][stop]   │  [EQ/FX]     │  [play][cue][stop]    │
│  [pitch fader]       │  [master vol]│  [pitch fader]        │
│  [loop in/out]       │  [BPM sync]  │  [loop in/out]        │
│  [hot cues A]        │              │  [hot cues B]         │
│  [EQ knobs A ]       │              │  [EQ knobs B ]        │
│  [FX panel A ]       │              │  [FX panel B ]        │
└──────────────────────┴──────────────┴───────────────────────┘
```

**Navigation routes:**

| Hash | Route | View |
|---|---|---|
| `#/` | `Route::Main` | Dual-deck mixer (default) |
| `#/settings` | `Route::Settings` | Settings panel |
| `#/help` | `Route::Help` | Quick-start guide |
| `#/about` | `Route::About` | About / credits card |

The header nav order is: `[Settings]` · `[Help]` · `[About]`. Clicking the logo or any nav link sets `window.location.hash`; the `hashchange` listener updates the active route. `DeckView` stays mounted on all routes so audio continues playing.

---

## 8. Audio Architecture

The Web Audio API graph per deck:

```
[AudioBufferSourceNode]
        │
  [pitch/rate control]
        │
  [BiquadFilterNode × 3]   ← 3-band EQ (high, mid, low)
        │
  [BiquadFilterNode]       ← sweep filter
        │
  [ConvolverNode]          ← reverb (bypass when off)
        │
  [DelayNode + FeedbackGainNode]  ← echo/delay
        │
  [GainNode]               ← channel fader
        │
  [GainNode (xfade)]       ← crossfader blend (equal-power cos/sin per deck)
        │
  [GainNode]               ← master volume
        │
 [AudioContext.destination]
```

---

## 9. Keyboard Shortcuts

| Key | Action |
|---|---|
| `Space` | Play/Pause Deck A |
| `Enter` | Play/Pause Deck B |
| `Q` | Cue Deck A |
| `P` | Cue Deck B |
| `Z` | Loop In — Deck A |
| `X` | Loop Out — Deck A |
| `N` | Loop In — Deck B |
| `M` | Loop Out — Deck B |
| `1–4` | Hot Cues Deck A |
| `7–0` | Hot Cues Deck B |
| `←/→` | Nudge Deck A |
| `[/]` | Nudge Deck B |

---

## 10. Stretch Goals / Future Ideas

- **Recording / Export**: Record the mixed output and export as a WAV file using the `MediaRecorder` API.
- **Sampler Pads**: A small 4×4 pad grid for triggering audio samples (think mini MPC).
- **Visualizer**: Full-screen audio visualizer mode with animated frequency bars or waveform ribbons.
- **Playlist Mode**: Queue up tracks per deck to auto-load on track end.
- **Touch / Mobile Support**: Large tap targets and gesture support for use on tablets.
- **Preset FX Chains**: Save and recall effect chain settings per session.
- **Animated Character Mascot**: A cartoon DJ character that reacts to the mix (bobs head to BPM, throws hands up on a drop, etc.).

---

## 11. Resolved Decisions

| Question | Decision |
|---|---|
| Audio time-stretching (tempo-only mode) | **Dropped from roadmap.** AudioWorklet (required for real-time DSP) requires COOP/COEP HTTP headers which complicates static hosting. Only vinyl-mode speed change ships (pitch follows rate). |
| Hot cues / loop points — persist via `localStorage`? | **No persistence.** Hot cues and loop points reset every session. |
| Canvas rendering — Leptos crate vs raw `web-sys`? | **Raw `web-sys` canvas calls.** Zero extra dependencies; fully specified in the tech spec. |
| Minimum supported browser | **Latest Chrome, Firefox, Safari only.** No polyfills. |
| Mobile / touch support | **Responsive-friendly CSS, no touch gestures.** Layout must not break on narrow viewports but no swipe/pinch support in v1. |
| BPM auto-detection | **Include in v1.** Runs once on file load in normal WASM (no AudioWorklet required). TAP BPM remains as a manual override. |

---

*This document is a living specification. Details will evolve as design and implementation progress.*
