//! Background image processor for RW Defender.
//!
//! Resizes and adjusts NASA/ESA space images to work as game backgrounds.
//! Target: 640×480 JPEG at 85% quality (~80-120KB each).
//!
//! Processing pipeline per image:
//!   1. Load (JPEG or PNG)
//!   2. Scale to cover 640×480 (crop to fit)
//!   3. Slight desaturation (reduce color saturation by 30%)
//!   4. Slight darkening (multiply brightness by 0.85) — CSS also darkens at runtime
//!   5. Save as JPEG quality 85
//!
//! Usage: process_backgrounds <input_dir> <output_dir>
//!    or: process_backgrounds <input.jpg> <output_dir>

use image::{DynamicImage, GenericImageView, ImageReader};
use std::{env, fs, path::Path, path::PathBuf};

const TARGET_W: u32 = 640;
const TARGET_H: u32 = 480;
const JPEG_QUALITY: u8 = 85;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: process_backgrounds <input_dir_or_file> <output_dir>");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);

    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    let files: Vec<PathBuf> = if input_path.is_dir() {
        fs::read_dir(&input_path)
            .expect("Cannot read input directory")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                matches!(
                    p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "webp" | "tiff" | "tif"
                )
            })
            .collect()
    } else {
        vec![input_path]
    };

    if files.is_empty() {
        eprintln!("No supported image files found.");
        std::process::exit(1);
    }

    println!("Processing {} images → {}", files.len(), output_dir.display());

    let mut success = 0;
    let mut errors = 0;

    for file in &files {
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
        let output_file = output_dir.join(format!("{}.jpg", stem));

        match process_image(file, &output_file) {
            Ok(bytes) => {
                println!(
                    "  ✓ {} → {} ({:.1}KB)",
                    file.file_name().unwrap_or_default().to_string_lossy(),
                    output_file.file_name().unwrap_or_default().to_string_lossy(),
                    bytes as f64 / 1024.0,
                );
                success += 1;
            }
            Err(e) => {
                eprintln!("  ✗ {}: {}", file.display(), e);
                errors += 1;
            }
        }
    }

    println!("\nDone: {} succeeded, {} failed", success, errors);
}

fn process_image(input: &Path, output: &Path) -> Result<u64, Box<dyn std::error::Error>> {
    let reader = ImageReader::open(input)?;
    // Disable memory limits for very large NASA/ESA originals (some are >200MB)
    let mut no_limits = reader.with_guessed_format()?;
    no_limits.limits(image::io::Limits::no_limits());
    let img = no_limits.decode()?;

    // Scale to fill 640×480 (center-crop)
    let scaled = scale_to_cover(img, TARGET_W, TARGET_H);

    // Desaturate 30% and darken 15%
    let adjusted = adjust_image(scaled);

    // Save as JPEG
    adjusted.save_with_format(output, image::ImageFormat::Jpeg)?;

    // Re-save with specific quality using JPEG encoder
    let buf = encode_jpeg(&adjusted, JPEG_QUALITY)?;
    fs::write(output, &buf)?;

    Ok(buf.len() as u64)
}

/// Scale image to cover the target dimensions (center crop).
fn scale_to_cover(img: DynamicImage, target_w: u32, target_h: u32) -> DynamicImage {
    let (src_w, src_h) = img.dimensions();

    // Calculate scale to cover target (both dimensions >= target)
    let scale_w = target_w as f64 / src_w as f64;
    let scale_h = target_h as f64 / src_h as f64;
    let scale = scale_w.max(scale_h);

    let new_w = (src_w as f64 * scale).round() as u32;
    let new_h = (src_h as f64 * scale).round() as u32;

    let resized = img.resize(new_w, new_h, image::imageops::FilterType::Lanczos3);

    // Center crop
    let (rw, rh) = resized.dimensions();
    let x = (rw.saturating_sub(target_w)) / 2;
    let y = (rh.saturating_sub(target_h)) / 2;
    resized.crop_imm(x, y, target_w, target_h)
}

/// Slightly desaturate and darken the image for use as a game background.
fn adjust_image(img: DynamicImage) -> DynamicImage {
    let mut rgb = img.to_rgb8();
    for pixel in rgb.pixels_mut() {
        let [r, g, b] = pixel.0;
        let rf = r as f64 / 255.0;
        let gf = g as f64 / 255.0;
        let bf = b as f64 / 255.0;

        // Desaturate 30% (lerp toward luminance)
        let lum = 0.299 * rf + 0.587 * gf + 0.114 * bf;
        let desat = 0.30;
        let rf = rf + (lum - rf) * desat;
        let gf = gf + (lum - gf) * desat;
        let bf = bf + (lum - bf) * desat;

        // Darken 15%
        let dark = 0.85;
        pixel.0 = [
            (rf * dark * 255.0).round().clamp(0.0, 255.0) as u8,
            (gf * dark * 255.0).round().clamp(0.0, 255.0) as u8,
            (bf * dark * 255.0).round().clamp(0.0, 255.0) as u8,
        ];
    }
    DynamicImage::ImageRgb8(rgb)
}

/// Encode to JPEG bytes at the given quality level.
fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    encoder.encode_image(img)?;
    Ok(buf)
}
