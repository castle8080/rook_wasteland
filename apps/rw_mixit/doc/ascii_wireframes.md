# rw_mixit — ASCII Wireframes

> Interface sketches for `rw_mixit`, a browser-based cartoon-style DJ mixer.
> Covers: App Shell, Main DJ View (full + deck detail + mixer detail), Settings, and About.

---

## 1. Application Shell

Wraps every view. Rendered by `<App>` / `<Header>`.

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ★ rw_mixit ★                               [ Settings ]  [ About ]     │
│  ════════════════════════════════════════════════════════════════════    │
│                                                                          │
│  [ view content renders here ]                                           │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

| Element | Action |
|---|---|
| `★ rw_mixit ★` | Clicking navigates to `#/` (main DJ view) |
| `[ Settings ]` | Navigates to `#/settings` |
| `[ About ]` | Navigates to `#/about` |

---

## 2. Main DJ View (`#/`)

### 2a. Full Layout Overview

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│  ★ rw_mixit ★                                           [ Settings ]  [ About ]        │
│  ══════════════════════════════════════════════════════════════════════════════════     │
│                                                                                        │
│  ┌────────────────────────────┐  ┌───────────────────┐  ┌────────────────────────────┐│
│  │          DECK  A           │  │      M I X E R    │  │          DECK  B           ││
│  │                            │  │                   │  │                            ││
│  │  ♪ track name       3:42  │  │  MASTER VOL (○)   │  │  ♪ track name       4:10  ││
│  │                            │  │                   │  │                            ││
│  │  ┌──────────────────────┐  │  │  CH-A  ▲  CH-B   │  │  ┌──────────────────────┐  ││
│  │  │▁▂▃▅▄▃▂▁▄▆▇█▇▆▄▂▁▂▃▅│  │  │  ┌───┐  │  ┌───┐  │  │  │▁▂▃▅▄▃▂▁▄▆▇█▇▆▄▂▁▂▃▅│  ││
│  │  │       ║  ░░loop░░   │  │  │  │   │  │  │   │  │  │  │       ║  ░░loop░░   │  ││
│  │  │   ▲   ║             │  │  │  │█  │     │  █│  │  │  │   ▲   ║             │  ││
│  │  └──────────────────────┘  │  │  │█  │     │  █│  │  │  └──────────────────────┘  ││
│  │                            │  │  │█  │     │  █│  │  │                            ││
│  │        ╭───────────╮       │  │  └───┘     └───┘  │  │        ╭───────────╮       ││
│  │       ╱  ╭───────╮  ╲      │  │                   │  │       ╱  ╭───────╮  ╲      ││
│  │      │  │ LABEL  │  │     │  │  A ◄──[●]──► B    │  │      │  │ LABEL  │  │     ││
│  │      │  │   ⊙    │  │     │  │    CROSSFADER      │  │      │  │   ⊙    │  │     ││
│  │       ╲  ╰───────╯  ╱      │  │                   │  │       ╲  ╰───────╯  ╱      ││
│  │        ╰───────────╯       │  │  A: 128.0          │  │        ╰───────────╯       ││
│  │                            │  │  B: 130.0          │  │                            ││
│  │  VU: ▐▐▐▐▐▐▌▌▌░░░          │  │  [MASTER: A]      │  │  VU: ▐▐▐▐▐▐▌▌▌░░░          ││
│  │                            │  │                   │  │                            ││
│  │  [▶ PLAY] [■ STOP] [◉ CUE] │  │                   │  │  [▶ PLAY] [■ STOP] [◉ CUE] ││
│  │  [◄◄ NUDGE]  [NUDGE ►►]    │  │                   │  │  [◄◄ NUDGE]  [NUDGE ►►]    ││
│  │                            │  │                   │  │                            ││
│  │  -50%──[●]────────+50%     │  │                   │  │  -50%──────────[●]─+50%    ││
│  │  PITCH  (vinyl mode)       │  │                   │  │  PITCH  (vinyl mode)       ││
│  │                            │  │                   │  │                            ││
│  │  [LOOP IN] [LOOP OUT] [ON] │  │                   │  │  [LOOP IN] [LOOP OUT] [ON] ││
│  │  [½] [1] [2] [4] [8] bars  │  │                   │  │  [½] [1] [2] [4] [8] bars  ││
│  │                            │  │                   │  │                            ││
│  │  [●R] [●B] [●G] [●Y] cues  │  │                   │  │  [●R] [●B] [●G] [●Y] cues  ││
│  │                            │  │                   │  │                            ││
│  │  (HI○) (MID○) (LOW○) (F○) │  │                   │  │  (HI○) (MID○) (LOW○) (F○) ││
│  │  EQ knobs        FILTER    │  │                   │  │  EQ knobs        FILTER    ││
│  │                            │  │                   │  │                            ││
│  │  [ECHO][VERB][FLNG]        │  │                   │  │  [ECHO][VERB][FLNG]        ││
│  │  [STUT][SCRTCH]            │  │                   │  │  [STUT][SCRTCH]            ││
│  │                            │  │                   │  │                            ││
│  │  [ ▼  LOAD TRACK  ▼ ]      │  │                   │  │  [ ▼  LOAD TRACK  ▼ ]      ││
│  └────────────────────────────┘  └───────────────────┘  └────────────────────────────┘│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 2b. Deck Detail (Deck A shown; Deck B is a mirror image)

```
┌────────────────────────────────────────────────────────────┐
│  DECK A                                   BPM: 128.0       │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ♪  Boombap Vol. 1 - Track 03.mp3              3:42       │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │▁▂▃▅▄▃▂▁▂▄▆▇█▇▆▄▂▁│▂▃▅▄▃▂▁▂▄▆▇█▇▆▄▂▁▂▃▄▅▃▂▁▂▄▅▃▂│  │
│  │                   ║░░░░░ loop region ░░░░░          │  │
│  │         ▲ 1:14    ║              ◆ GRN  ◆ RED       │  │
│  └──────────────────────────────────────────────────────┘  │
│  [−] zoom [+]   click or drag to seek (playing or paused)     │
│  ◆ = hot cue markers (fixed; hold button to set at current position)                           │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│               PLATTER (canvas, animated)                   │
│                                                            │
│                   ╭───────────────╮                        │
│                  ╱  ╭───────────╮  ╲                       │
│                 │  │             │  │                      │
│                 │  │  Boombap    │  │  ╲                   │
│                 │  │  Vol. 1     │  │   ◝ tonearm          │
│                 │  │     ⊙       │  │   (pivots from       │
│                 │  │             │  │    top-right,        │
│                  ╲  ╰───────────╯  ╱    tracks over        │
│                   ╰───────────────╯     full duration)     │
│                                                            │
│  VU METER  ▐▐▐▐▐▐▐▌▌▌▌▌░░░░  (live, from AnalyserNode)   │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  TRANSPORT                                                 │
│                                                            │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│  │  ▶  PLAY   │  │  ■  STOP   │  │  ◉  CUE    │           │
│  └────────────┘  └────────────┘  └────────────┘           │
│  [ ◄◄ NUDGE ]                        [ NUDGE ►► ]         │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  PITCH FADER  (vinyl mode — pitch follows speed)           │
│                                                            │
│  -50% ───────────────────[●]──────────────────── +50%     │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  BPM                                                       │
│                                                            │
│  Detected: 128.0    [ TAP BPM ]    [ SYNC ]               │
│                                    snaps speed             │
│                                    to other deck           │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  LOOP CONTROLS                                             │
│                                                            │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐        │
│  │ LOOP IN  │  │ LOOP OUT │  │   LOOP  [ ON/OFF ]│        │
│  └──────────┘  └──────────┘  └───────────────────┘        │
│                                                            │
│  Quick lengths:  [ ½ ] [ 1 ] [ 2 ] [ 4 ] [ 8 ]  bars     │
│                  (based on detected BPM)                   │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  HOT CUES  (tap to jump · hold to set · dbl-tap to clear) │
│                                                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐      │
│  │ ●  CUE 1│  │ ●  CUE 2│  │ ●  CUE 3│  │ ●  CUE 4│      │
│  │  RED    │  │  BLUE   │  │  GREEN  │  │  YELLOW │      │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘      │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  EQ  (rotary knobs, −12 dB to +12 dB)                     │
│                                                            │
│      (○)       (○)       (○)            (○)               │
│      HI        MID       LOW          FILTER               │
│    +12/−12   +12/−12   +12/−12      LP ◄──► HP             │
│                                     (center = flat)        │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  FX  (toggle on/off; param knobs appear when active)       │
│                                                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                    │
│  │  ECHO   │  │  REVERB │  │ FLANGER │                    │
│  │  [OFF]  │  │  [OFF]  │  │  [OFF]  │                    │
│  │ time(○) │  │  dur(○) │  │ rate(○) │                    │
│  │  fb (○) │  │ decay(○)│  │depth(○) │                    │
│  └─────────┘  └─────────┘  └─────────┘                    │
│                                                            │
│  ┌─────────┐  ┌───────────────────────────────────────┐   │
│  │ STUTTER │  │  SCRATCH (SIM)                        │   │
│  │  [OFF]  │  │  [OFF]  drag platter to activate      │   │
│  │ gate(○) │  └───────────────────────────────────────┘   │
│  └─────────┘                                              │
│                                                            │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                                                            │
│  ┌────────────────────────────────────────────────────┐   │
│  │             ▼   LOAD TRACK   ▼                     │   │
│  └────────────────────────────────────────────────────┘   │
│  Opens browser file picker (.mp3 .wav .ogg .flac .aac)    │
└────────────────────────────────────────────────────────────┘
```

**Deck element reference:**

| Element | Description | Interaction |
|---|---|---|
| Track label | Shows filename and duration | Display only; set on load |
| Waveform | Scrolling canvas of audio peaks | Click or drag to seek (playing or paused); cursor `grab`/`grabbing`; zoom in/out with [−]/[+] |
| Loop region | Translucent highlight on waveform | Pulses with heartbeat glow when active |
| Hot cue markers `◆` | Colored pins on waveform | Draggable to adjust position |
| Platter | Spinning vinyl canvas at 33 RPM × playbackRate | Drag to simulate scratch (when Scratch FX on) |
| Tonearm | Pivots from top-right over track duration | Visual only |
| VU meter | Bar driven by `AnalyserNode` data | Display only |
| PLAY / PAUSE | Toggle playback | Platter spins; waveform scrolls |
| STOP | Stop + reset to start | Platter stops |
| CUE | Jump to / hold to preview cue point | Hold to audition, release to snap back |
| NUDGE ◄► | Temporarily push BPM up/down | Hold for continuous nudge |
| Pitch fader | Horizontal slider, −50% to +50% | Drag; platter speed follows in real time |
| TAP BPM | Override BPM by tapping in time | Each tap refines BPM estimate |
| SYNC | Snap speed to match the other deck's BPM | One-shot adjustment |
| LOOP IN / OUT | Set loop boundary at current playhead | Instant |
| LOOP ON/OFF | Enable/disable without clearing points | Toggle |
| Quick bar buttons | Set loop length from ½ to 8 bars | Calculated from current BPM |
| Hot cue buttons | 4 colored buttons (R/B/G/Y) | Tap to jump · Hold to set · Double-tap to clear |
| EQ knobs HI/MID/LOW | ±12 dB rotary knobs | Drag or scroll |
| Filter knob | LP/HP sweep filter | Center = flat; left = low-pass; right = high-pass |
| FX toggles | Echo / Reverb / Flanger / Stutter / Scratch | Toggle on/off; param knobs animate in when on |
| LOAD TRACK | Opens file picker | Decodes audio, extracts waveform peaks, sets BPM |

---

### 2c. Mixer Panel Detail

```
┌─────────────────────────┐
│         MIXER           │
├─────────────────────────┤
│                         │
│    MASTER VOLUME        │
│                         │
│          (○)            │
│       0 ──── 100        │
│                         │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                         │
│   CHANNEL FADERS        │
│                         │
│    A              B     │
│    ▲              ▲     │
│  ┌───┐          ┌───┐   │
│  │   │          │   │   │
│  │   │          │   │   │
│  │ █ │          │ █ │   │
│  │ █ │          │ █ │   │
│  │ █ │          │ █ │   │
│  │ █ │  ──────  │ █ │   │
│  │ █ │  100%    │ █ │   │
│  └───┘  75%     └───┘   │
│                         │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                         │
│   CROSSFADER            │
│                         │
│  A ◄────────[●]────► B  │
│  (equal-power curve)    │
│                         │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│                         │
│   BPM SYNC              │
│                         │
│  Deck A:  128.0  [SYNC] │
│  Deck B:  130.0  [SYNC] │
│                         │
│  MASTER: [ A ] ← active │
│                         │
│  [ TAP A ]  [ TAP B ]   │
│                         │
└─────────────────────────┘
```

**Mixer element reference:**

| Element | Description | Interaction |
|---|---|---|
| Master volume | Output gain for entire mix | Rotary knob |
| Channel faders A / B | Per-deck volume | Vertical drag sliders |
| Crossfader | Blends A↔B using equal-power cos/sin curve | Horizontal drag; glows when moved |
| SYNC buttons | Snap that deck's speed to match the other | One-shot; respects MASTER designation |
| MASTER indicator | Shows which deck is the tempo reference | Click to transfer master to other deck |
| TAP A / TAP B | Set BPM for that deck by tapping | Each tap refines the estimate |

---

## 3. Settings View (`#/settings`)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ★ rw_mixit ★                               [ Settings ]  [ About ]     │
│  ════════════════════════════════════════════════════════════════════    │
│                                                                          │
│  ╔══════════════════════════════════════════════════════════════════╗   │
│  ║  SETTINGS                                                        ║   │
│  ╠══════════════════════════════════════════════════════════════════╣   │
│  ║                                                                  ║   │
│  ║  Audio                                                           ║   │
│  ║  ──────────────────────────────────────────────────────────     ║   │
│  ║  Reverb tail length:    ──────────[●]──────────  2.0s           ║   │
│  ║  Reverb decay:          ─────────[●]───────────  0.7            ║   │
│  ║  Crossfader curve:      (●) Equal-power  ( ) Linear             ║   │
│  ║                                                                  ║   │
│  ║  Keyboard Shortcuts                                              ║   │
│  ║  ──────────────────────────────────────────────────────────     ║   │
│  ║  Space        →  Play / Pause   Deck A                          ║   │
│  ║  Enter        →  Play / Pause   Deck B                          ║   │
│  ║  Q / P        →  Cue            Deck A / B                      ║   │
│  ║  Z / X        →  Loop In / Out  Deck A                          ║   │
│  ║  N / M        →  Loop In / Out  Deck B                          ║   │
│  ║  1 – 4        →  Hot Cues       Deck A                          ║   │
│  ║  7 – 0        →  Hot Cues       Deck B                          ║   │
│  ║  ← / →        →  Nudge          Deck A                          ║   │
│  ║  [ / ]        →  Nudge          Deck B                          ║   │
│  ║                                                                  ║   │
│  ╚══════════════════════════════════════════════════════════════════╝   │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

**Settings element reference:**

| Element | Description | Interaction |
|---|---|---|
| Reverb tail length | IR generator `duration_secs` param (0.5 – 3.0 s) | Slider; updates reverb IR in real time |
| Reverb decay | IR generator `decay` param (tightness of tail) | Slider |
| Crossfader curve | Equal-power (cos/sin) or linear blend | Radio toggle |
| Keyboard shortcuts | Read-only reference table | Display only |

---

## 4. About View (`#/about`)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ★ rw_mixit ★                               [ Settings ]  [ About ]     │
│  ════════════════════════════════════════════════════════════════════    │
│                                                                          │
│  ╔══════════════════════════════════════════════════════════════════╗   │
│  ║                                                                  ║   │
│  ║           ★   r w _ m i x i t   ★                               ║   │
│  ║           v 0 . 1 . 0                                            ║   │
│  ║                                                                  ║   │
│  ║  A browser-based cartoon DJ mixer.                               ║   │
│  ║  Built with Rust + Leptos (WebAssembly).                         ║   │
│  ║  No server. No installs. Just vibes.                             ║   │
│  ║                                                                  ║   │
│  ║  ──────────────────────────────────────────────────────────     ║   │
│  ║  Stack                                                           ║   │
│  ║    • Rust (stable ≥ 1.78)                                        ║   │
│  ║    • Leptos 0.8  (CSR / wasm32)                                  ║   │
│  ║    • Web Audio API  (via web-sys)                                ║   │
│  ║    • Trunk 0.21                                                  ║   │
│  ║    • Bangers typeface  (SIL OFL, self-hosted)                    ║   │
│  ║                                                                  ║   │
│  ║  ──────────────────────────────────────────────────────────     ║   │
│  ║  Part of the rook_wasteland project.                             ║   │
│  ║                                                                  ║   │
│  ╚══════════════════════════════════════════════════════════════════╝   │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 5. State / Interaction Flow Summary

```
  User loads file               User hits Play
       │                              │
       ▼                              ▼
  <input type="file">         toggle_play(deck_state)
  → FileReader                → recreate AudioBufferSourceNode
  → decodeAudioData()         → .start(0, offset)
  → extract waveform peaks    → is_playing.set(true)
  → update DeckState          → platter begins spinning (rAF)
  → waveform canvas redraws   → waveform playhead scrolls (rAF)

  rAF loop (60 fps)
  ─────────────────
  read current_time from AudioContext
  → update current_secs signal
  → check loop_out boundary → re-schedule if looping
  → draw_platter() on canvas
  → draw_waveform() on canvas
  → update VU meter signal (→ reactive DOM height update)
```
