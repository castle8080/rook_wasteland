# rw_teleidoscope — ASCII Wireframes

Steampunk aesthetic throughout the UI shell. All borders use heavy ornamental framing;
controls evoke gauges, levers, and valve handles. The WebGL canvas is undecorated.

---

## 1. Landing State — No Image Loaded

```
╔══════════════════════════════════════════════════════════════════════════════════╗
║  ⚙  T E L E I D O S C O P E           [ ⚙ LOAD IMAGE ]  [ 📷 USE CAMERA ]    ║
╠════════════════╦═════════════════════════════════════════════════════════════════╣
║                ║                                                                 ║
║  MIRRORS  ░░░  ║   ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐   ║
║  ╞══●═══════╡  ║                                                               ║
║       6        ║   │                                                           │   ║
║                ║        ╔═══════════════════════════════════════╗              ║
║  ROTATION ░░░  ║   │    ║                                       ║          │   ║
║  ╞════●════╡   ║        ║   ⚙  Drop an image here, or use      ║              ║
║      180°      ║   │    ║   Load Image / Use Camera above.     ║          │   ║
║                ║        ║                                       ║              ║
║  ZOOM     ░░░  ║   │    ║   Supported: PNG  JPEG  WebP         ║          │   ║
║  ╞══●═══════╡  ║        ╚═══════════════════════════════════════╝              ║
║      1.0×      ║   │                                                           │   ║
║                ║                                                               ║
║ ══╡ EFFECTS ╞══║   │                                                           │   ║
║ ▣ Spiral    ░  ║    ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─    ║
║ ▣ Radial    ░  ║                                                                 ║
║ ▣ Lens      ░  ║                                                                 ║
║ ▣ Ripple    ░  ║                                                                 ║
║ ▣ Möbius    ░  ║                                                                 ║
║ ▣ Recursive ░  ║                                                                 ║
║                ║                                                                 ║
║ ══╡  COLOR  ╞══║                                                                 ║
║  HUE           ║                                                                 ║
║  ╞═══════●══╡  ║                                                                 ║
║  SATURATION    ║                                                                 ║
║  ╞════●═════╡  ║                                                                 ║
║  BRIGHTNESS    ║                                                                 ║
║  ╞════●═════╡  ║                                                                 ║
║  POSTERIZE     ║                                                                 ║
║  ╞══●═══════╡  ║                                                                 ║
║                ║                                                                 ║
║ ╔════════════╗ ║                                                                 ║
║ ║ ⚡ RANDOM  ║ ║                                                                 ║
║ ╚════════════╝ ║                                                                 ║
║ ╔════════════╗ ║                                                                 ║
║ ║ ↓  EXPORT  ║ ║                                                                 ║
║ ╚════════════╝ ║                                                                 ║
╚════════════════╩═════════════════════════════════════════════════════════════════╝
```

> Controls are rendered but dimmed (░ indicators). Sliders and effect toggles are
> non-interactive until an image is loaded.

---

## 2. Main View — Image Loaded, Panel Open

```
╔══════════════════════════════════════════════════════════════════════════════════╗
║  ⚙  T E L E I D O S C O P E           [ ⚙ LOAD IMAGE ]  [ 📷 USE CAMERA ]    ║
╠════════════════╦═════════════════════════════════════════════════════════════════╣
║  ◄ COLLAPSE    ║                                                                 ║
║                ║   ████████████████████████████████████████████████████████     ║
║  MIRRORS       ║   ████████████████████████████████████████████████████████     ║
║  ╞══●═══════╡  ║   ██████████████████▓▓▓▓████████████████████████████████      ║
║       6        ║   ████████████████▓▓░░░░▓▓██████████████████████████████      ║
║                ║   ██████████████▓░░░░░░░░░▓████████████████████████████       ║
║  ROTATION      ║   ████████████▓░░░░░░░░░░░░▓██████████████████████████        ║
║  ╞════●════╡   ║   ██████████▓░░░░░░  ░░░░░░░▓████████████████████████         ║
║      180°      ║   ████████▓░░░░  ░░░░░░  ░░░░▓██████████████████████          ║
║                ║   ████████▓░░░░░░░░░░░░░░░░░░░▓████████████████████           ║
║  ZOOM          ║   ██████▓░░░░  ░░░░░░░░░░  ░░░░▓██████████████████            ║
║  ╞══●═══════╡  ║   ████████▓░░░░░░░░░░░░░░░░░░░▓████████████████████           ║
║      1.0×      ║   ████████▓░░░░  ░░░░░░  ░░░░▓██████████████████████          ║
║                ║   ██████████▓░░░░░░  ░░░░░░░▓████████████████████████         ║
║ ══╡ EFFECTS ╞══║   ████████████▓░░░░░░░░░░░▓██████████████████████████         ║
║ ■ Spiral    ▓▓ ║   ██████████████▓░░░░░░░▓████████████████████████████         ║
║   ╞═══●═════╡  ║   ████████████████▓▓░░▓▓██████████████████████████████        ║
║ ▣ Radial    ░  ║   ████████████████████████████████████████████████████████     ║
║ ▣ Lens      ░  ║   ████████████████████████████████████████████████████████     ║
║ ▣ Ripple    ░  ║                                                                 ║
║ ▣ Möbius    ░  ║   ╔ drag to move center ──────────────────────────────────╗    ║
║ ▣ Recursive ░  ║   ║  ↖  center is currently at 50%, 50%                  ║    ║
║                ║   ╚──────────────────────────────────────────────────────╝    ║
║ ══╡  COLOR  ╞══║                                                                 ║
║  HUE           ║                                                                 ║
║  ╞═══════●══╡  ║                                                                 ║
║  SATURATION    ║                                                                 ║
║  ╞════●═════╡  ║                                                                 ║
║  BRIGHTNESS    ║                                                                 ║
║  ╞════●═════╡  ║                                                                 ║
║  POSTERIZE     ║                                                                 ║
║  ╞══●═══════╡  ║                                                                 ║
║                ║                                                                 ║
║ ╔════════════╗ ║                                                                 ║
║ ║ ⚡ RANDOM  ║ ║                                                                 ║
║ ╚════════════╝ ║                                                                 ║
║ ╔════════════╗ ║                                                                 ║
║ ║ ↓  EXPORT  ║ ║                                                                 ║
║ ╚════════════╝ ║                                                                 ║
╚════════════════╩═════════════════════════════════════════════════════════════════╝
```

> Panel is scrollable if all controls exceed the viewport height. The canvas hint
> tooltip fades after 3 seconds.

---

## 3. Main View — Panel Collapsed

```
╔══════════════════════════════════════════════════════════════════════════════════╗
║  ⚙  T E L E I D O S C O P E           [ ⚙ LOAD IMAGE ]  [ 📷 USE CAMERA ]    ║
╠══╦═══════════════════════════════════════════════════════════════════════════════╣
║  ║                                                                               ║
║► ║  ████████████████████████████████████████████████████████████████████████    ║
║  ║  ████████████████████████████████████████████████████████████████████████    ║
║  ║  ██████████████████████▓▓▓▓████████████████████████████████████████████      ║
║  ║  ████████████████████▓▓░░░░▓▓██████████████████████████████████████████      ║
║  ║  ██████████████████▓░░░░░░░░░▓████████████████████████████████████████       ║
║  ║  ████████████████▓░░░░░░░░░░░░▓██████████████████████████████████████        ║
║  ║  ██████████████▓░░░░░░    ░░░░░░▓████████████████████████████████████        ║
║  ║  ████████████▓░░░░   ░░░░░░   ░░░▓██████████████████████████████████         ║
║  ║  ██████████████▓░░░░░░░░░░░░░░░░▓████████████████████████████████████        ║
║  ║  ████████████████▓░░░░░░░░░░░░▓██████████████████████████████████████        ║
║  ║  ██████████████████▓░░░░░░░░▓████████████████████████████████████████        ║
║  ║  ████████████████████▓▓░░▓▓██████████████████████████████████████████        ║
║  ║  ████████████████████████████████████████████████████████████████████████    ║
║  ║  ████████████████████████████████████████████████████████████████████████    ║
║  ║                                                                               ║
╚══╩═══════════════════════════════════════════════════════════════════════════════╝
```

> The collapsed panel is a narrow strip containing only the ► expand button.
> The canvas fills the remaining full width.

---

## 4. Camera Overlay — Preview & Capture

Activated when the user clicks "📷 USE CAMERA". The kaleidoscope canvas dims and
a centred modal overlay appears with the live camera feed.

```
╔══════════════════════════════════════════════════════════════════════════════════╗
║  ⚙  T E L E I D O S C O P E           [ ⚙ LOAD IMAGE ]  [ 📷 USE CAMERA ]    ║
╠════════════════╦═════════════════════════════════════════════════════════════════╣
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║  (panel dims)  ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒╔═══════════════════════════════╗▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║  ══╡  📷  C A M E R A  ╞══  ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒╠═══════════════════════════════╣▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║                               ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   ┌───────────────────────┐   ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   │                       │   ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   │   live camera feed    │   ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   │   (<video> element)   │   ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   │                       │   ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   └───────────────────────┘   ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║                               ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   ╔═══════════╗ ╔══════════╗  ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   ║ 📷 CAPTURE ║ ║  CANCEL  ║  ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║   ╚═══════════╝ ╚══════════╝  ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒╚═══════════════════════════════╝▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
║                ║▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒║
╚════════════════╩═════════════════════════════════════════════════════════════════╝
```

> ▒ = dimmed background (canvas + panel behind the modal).  
> Clicking CAPTURE freezes the frame, dismisses the modal, and feeds the snapshot
> into the WebGL texture pipeline — identical to loading a file.  
> Clicking CANCEL dismisses the modal without changing the source image.  
> If `getUserMedia` is denied, the modal is replaced by an inline error message:

```
╔═══════════════════════════════════════════════════╗
║  ══╡  📷  C A M E R A  ╞══                       ║
╠═══════════════════════════════════════════════════╣
║                                                   ║
║  ⚠  Camera access was denied.                    ║
║                                                   ║
║  To use this feature, allow camera access in      ║
║  your browser's site permissions and try again.   ║
║                                                   ║
║                          ╔══════════╗             ║
║                          ║  CLOSE   ║             ║
║                          ╚══════════╝             ║
╚═══════════════════════════════════════════════════╝
```

---

## 5. Effects Controls — Detail View

Zoomed-in view of how a single active effect looks vs a disabled one.

```
 ╔═══════════════════════════╗
 ║  ══╡   E F F E C T S  ╞══ ║
 ╠═══════════════════════════╣
 ║                           ║
 ║  ■ Spiral Twist      ▓▓▓  ║  ← enabled; slider visible
 ║    ╞══════●══════╡        ║
 ║              0.52         ║
 ║  - - - - - - - - - - - -  ║
 ║  ■ Lens Distortion   ▓▓▓  ║  ← enabled
 ║    ╞══●═══════════╡       ║
 ║          0.23             ║
 ║  - - - - - - - - - - - -  ║
 ║  ▣ Radial Folding    ░░░  ║  ← disabled; slider hidden
 ║  - - - - - - - - - - - -  ║
 ║  ▣ Angular Ripple    ░░░  ║  ← disabled
 ║  - - - - - - - - - - - -  ║
 ║  ▣ Möbius Mirror     ░░░  ║  ← disabled
 ║  - - - - - - - - - - - -  ║
 ║  ▣ Recursive Reflect ░░░  ║  ← disabled
 ║                           ║
 ╚═══════════════════════════╝

  ■ = toggle ON    ▣ = toggle OFF
  ▓▓▓ = active     ░░░ = inactive/dimmed
```

---

## 6. Export Dropdown

Activated by clicking the ↓ EXPORT button.

```
  ╔════════════╗
  ║ ↓  EXPORT  ║  ← button
  ╚════╤═══════╝
       │
  ╔════╧══════════════╗
  ║  ◉ PNG            ║
  ║  ○ JPEG           ║
  ║  ○ WebP           ║
  ║ ─────────────── ─ ║
  ║  [ ↓  DOWNLOAD  ] ║
  ╚═══════════════════╝
```

> Dropdown appears directly below the Export button.

---

## 8. Mobile Layout — Drawer Closed (≤ 768 px)

```
┌────────────────────────┐
│ ⚙ TELEIDOSCOPE  ⚙  📷 │  ← compact header
├────────────────────────┤
│                        │
│                        │
│                        │
│    kaleidoscope        │
│      canvas            │
│   (fills viewport)     │
│                        │
│                        │
│                        │
├────────────────────────┤
│    ────────            │  ← drawer handle strip (pip indicator)
└────────────────────────┘
```

Tapping the handle strip slides the controls drawer up from the bottom.

---

## 9. Mobile Layout — Drawer Open (≤ 768 px)

```
┌────────────────────────┐
│ ⚙ TELEIDOSCOPE  ⚙  📷 │
├────────────────────────┤
│                        │
│    kaleidoscope        │  ← dimmed by backdrop (tap to close)
│      canvas            │
├────────────────────────┤
│  MIRRORS  ╞══●═════╡   │
│  ROTATION ╞════●═══╡   │  ← controls drawer (~55 vh, scrollable)
│  ZOOM     ╞══●═════╡   │
│  ─────────────────── ─ │
│  ▣ Spiral           ░  │
│  ▣ Radial           ░  │
│  ─────────────────── ─ │
│  [⚡ SURPRISE ME]      │
│  [ EXPORT ▾ ]          │
├────────────────────────┤
│    ────────            │  ← drawer handle strip
└────────────────────────┘
```

Tapping the dimmed canvas area (backdrop) slides the drawer back down.

---

## 10. Mobile — Camera Overlay (≤ 768 px)

```
┌────────────────────────┐
│░░░░░░░░░░░░░░░░░░░░░░░░│
│░░ ┌──────────────┐ ░░░│
│░░ │  CAMERA      │ ░░░│  ← modal: min(340px, 90vw) wide
│░░ ├──────────────┤ ░░░│
│░░ │  video feed  │ ░░░│
│░░ │  (4:3 ratio) │ ░░░│
│░░ ├──────────────┤ ░░░│
│░░ │[CAPTURE] [✕] │ ░░░│
│░░ └──────────────┘ ░░░│
│░░░░░░░░░░░░░░░░░░░░░░░░│
└────────────────────────┘
```

Modal uses `min(340px, 90vw)` so it fits 320 px screens without horizontal scroll.
> Format selection persists across sessions.

---

## 7. Mirrors Gauge — Detail

The mirrors control styled as a pressure gauge dial, as an aspirational reference
for how slider labels might be decorated in the steampunk style.

```
      ╔═══════════════════╗
      ║   M I R R O R S   ║
      ╠═══════════════════╣
      ║                   ║
      ║   2  3  4  5  6   ║
      ║   |  |  |  |  |   ║
      ║  ─┼──┼──┼──┼──┼─  ║
      ║             ●      ║
      ║   7  8  9  10      ║
      ║   |  |  |  |       ║
      ╠═══════════════════╣
      ║       6 ⚙         ║
      ╚═══════════════════╝
```

---

## Notes

- All heavy-line borders (╔═╗ etc.) represent the CSS styled UI chrome with a dark
  brass/metal aesthetic.
- ░ indicates a dimmed/inactive state; ▓ indicates an active/highlighted state.
- The canvas area (filled with ██) carries no steampunk styling — it is a plain
  `<canvas>` element displaying WebGL output.
- All interactive drag zones (center-of-symmetry, sliders) are on the canvas or
  within the panel respectively; no overlay chrome on the canvas itself.
