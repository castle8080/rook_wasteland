# Background Images

RW Defender uses real NASA/ESA space imagery as in-game backgrounds. Each wave tier
cycles through a curated set of public-domain or freely-licensed space photographs.

## License

- **NASA APOD / JWST / Hubble images**: Public domain (US government works, NASA policy).
- **ESA Hubble images**: © ESA/Hubble, free for non-commercial educational use with credit.
  See: <https://esahubble.org/copyright/>

Credit line for ESA images: *"Credit: ESA/Hubble & NASA"*

---

## Image Sources

### Tier 1 — Warm (Waves 1–5)

| File | Description | Source |
|------|-------------|--------|
| `esa_orion_nebula_m42.jpg` | Orion Nebula (M42) — sweeping stellar nursery with orange and purple gas clouds | ESA Hubble `heic0601a` — screen size |
| `esa_mystic_mountain_carina.jpg` | Mystic Mountain in the Carina Nebula — towering pillar of gas and dust | ESA Hubble `heic1006a` |
| `esa_eagle_nebula_m16.jpg` | Eagle Nebula (M16) — iconic "Pillars of Creation" region | ESA Hubble `heic1501a` |
| `esa_carina_nebula_hubble.jpg` | Carina Nebula — vast stellar nursery panorama | ESA Hubble `heic0707a` — screen size |
| `esa_helix_nebula.jpg` | Helix Nebula — nearby planetary nebula with warm eye-like structure | ESA Hubble `heic0305a` |

### Tier 2 — Nebula (Waves 6–11)

| File | Description | Source |
|------|-------------|--------|
| `esa_butterfly_nebula.jpg` | Butterfly Nebula (NGC 6302) — dramatic bipolar nebula with bright blue wings | ESA Hubble `heic0910a` |
| `esa_crab_nebula.jpg` | Crab Nebula (M1) — remnant of 1054 AD supernova, blue filaments | ESA Hubble `heic0507a` |
| `esa_ring_nebula_m57.jpg` | Ring Nebula (M57) — classic planetary nebula ring | ESA Hubble `heic1310a` |
| `08_hubble_ring_nebula_m57.jpg` | Ring Nebula (M57) alternate Hubble view | NASA APOD / Hubble |
| `02_jwst_southern_ring_nebula.jpg` | Southern Ring Nebula (NGC 3132) — JWST NIRCam+MIRI composite | NASA JWST Early Release |
| `03_jwst_hidden_orion.jpg` | Hidden Orion / JWST Orion bar — JWST infrared detail of Orion | NASA JWST Early Release |

### Tier 3 — Deep Space / Galaxies (Waves 12–17)

| File | Description | Source |
|------|-------------|--------|
| `esa_sombrero_galaxy.jpg` | Sombrero Galaxy (M104) — edge-on galaxy with prominent dust lane | ESA Hubble `heic0310a` |
| `esa_andromeda_galaxy_m31.jpg` | Andromeda Galaxy (M31) — nearest large spiral | ESA Hubble `heic1502b` — screen size |
| `esa_whirlpool_galaxy_m51.jpg` | Whirlpool Galaxy (M51) — interacting galaxy pair | ESA Hubble `heic0506a` |
| `esa_antennae_galaxies.jpg` | Antennae Galaxies — two colliding galaxies in vivid detail | ESA Hubble `heic0615a` |
| `esa_ngc1300_barred_spiral.jpg` | NGC 1300 — grand-design barred spiral galaxy | ESA Hubble `heic0503b` |
| `05_vlt_ngc1232_spiral_galaxy.jpg` | NGC 1232 — face-on spiral galaxy, VLT image | ESO/VLT (public domain) |

### Tier 4 — Ultra Deep (Waves 18+)

| File | Description | Source |
|------|-------------|--------|
| `01_jwst_deep_field_smacs0723.jpg` | SMACS 0723 Deep Field — thousands of galaxies across 4.6 billion light years | NASA JWST First Deep Field |
| `04_hubble_cats_eye_nebula.jpg` | Cat's Eye Nebula (NGC 6543) — complex multi-shell planetary nebula | NASA/HST |
| `esa_ngc3603_stellar_nursery.jpg` | NGC 3603 Stellar Nursery — dense star-forming region in the Milky Way | ESA Hubble `heic0706a` |
| `esa_hoags_object_ring_galaxy.jpg` | Hoag's Object — rare perfect ring galaxy | ESA Hubble `opo2004a` |
| `esa_omega_centauri_cluster.jpg` | Omega Centauri — largest globular cluster of the Milky Way | ESA Hubble `heic0908a` |
| `18_hubble_omega_nebula_m17.jpg` | Omega Nebula (M17) — Hubble view of star-forming region | NASA/HST |

---

## Processing

Raw source images were downloaded at high (or screen) resolution and processed with a
custom Rust tool located at `tools/` in the project.

### Tool: `tools/src/main.rs`

**Purpose:** Normalize all images to 640×480 JPEG for use as game backgrounds.

**Processing steps:**
1. Load image using the `image` crate (v0.25) with `no_limits()` for large files
2. **Scale-to-cover**: compute scale factor so both dimensions ≥ target (640×480), then center-crop
3. **Desaturate 30%**: blend toward grayscale to reduce color competition with game sprites
4. **Darken 15%**: multiply all pixel values by 0.85 for readability
5. Save as JPEG quality 85

**Build and run:**
```bash
cd tools
cargo build --release
./target/release/process_backgrounds ../tools/raw_source_images ../assets/backgrounds
```

**Output:** 640×480 JPEG files, 19–95KB each (vs. source files of 44KB–218MB).

### Large Image Handling

Three images (Andromeda, Carina, Orion) were originally downloaded at full `large`
resolution (100–218MB). These exceeded the `image` crate's default memory limits.

**Fix applied:** Use `with_no_limits()` on the `ImageReader`, and additionally download
the ESA `screen` size (~1280px, 180–260KB) instead of `large` for these three images.
The `screen` size is still adequate for 640×480 output.

### Raw Source Images

Original downloads are stored in `tools/raw_source_images/` (not committed to git;
listed in `.gitignore`). To regenerate the processed images:

1. Re-run the tool against `tools/raw_source_images/`
2. Output goes to `assets/backgrounds/`

---

## Game Integration

### Trunk Asset Deployment

`index.html` includes a Trunk copy-dir directive:
```html
<link data-trunk rel="copy-dir" href="assets/backgrounds" />
```
This copies `assets/backgrounds/*.jpg` to `dist/backgrounds/` at build time,
making images available at `/backgrounds/<filename>` in the web app.

### Wave-to-Image Assignment

`src/graphics/background.rs` exports `background_image_for_wave(wave: u32) -> &'static str`
which returns the filename for the current wave, cycling through each tier's images.

`src/lib.rs` calls `maybe_update_background(wave)` each game frame, which detects wave
changes and updates the `#space-bg` `<img>` element's `src` attribute to switch images.

### Rendering Stack (z-index, back to front)

1. **`#space-bg`** (z-index 0) — NASA/ESA space image, CSS-filtered: `brightness(0.4) saturate(0.65)`
2. **`#background-canvas`** (z-index 1) — Animated parallax starfield overlay, `opacity: 0.55`
3. **`#game-canvas`** (z-index 2) — Game sprites, HUD, all interactive elements
