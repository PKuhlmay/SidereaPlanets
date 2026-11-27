//! Regression test of wavelet sharpening against stored reference results for reference case R1 (Jupiter test recording,
//! `docs/wavelets.md`). The reference files live in `re-local/` and are not in the repository; without
//! them the tests are skipped. Input is the stored reference stack, so the stacking does not count.

use planets_core::{
    Image,
    wavelet::{ColourMode, WaveletSettings, do_all, scheme},
};
use std::path::PathBuf;

/// Smallest PSNR (16-bit peak) the ticket asks for, `docs/wavelets.md`.
const THRESHOLD_DB: f64 = 45.0;
/// Layer 1 = 10.0 against the reference. The stack itself is 55.6 dB away from the reference neutral result and
/// 52.8 dB from the Layer 1 = 10 result, so only a value clearly above that shows that the sharpening is right.
const STRICT_THRESHOLD_DB: f64 = 65.0;
/// Neutral settings against the reference. The tone curve is a fit to exactly this image, so the value is high; it
/// guards the curve, not the sharpening.
const NEUTRAL_THRESHOLD_DB: f64 = 85.0;

mod common;

fn reference_dir() -> Option<PathBuf> {
    Some(common::re_local().join("reference")).filter(|p| p.join("jupiter-stack.tif").exists())
}

/// 16-bit RGB samples of a stored reference TIFF.
fn read_tiff(path: PathBuf) -> (u32, u32, Vec<u16>) {
    let rgb = image::open(path).unwrap().into_rgb16();
    (rgb.width(), rgb.height(), rgb.into_raw())
}

fn to_image(width: u32, height: u32, samples: &[u16]) -> Image {
    Image {
        width,
        height,
        channels: 3,
        data: samples.iter().map(|&v| f32::from(v) / 65535.0).collect(),
    }
}

fn to_u16(image: &Image) -> Vec<u16> {
    image
        .data
        .iter()
        .map(|v| (v.clamp(0.0, 1.0) * 65535.0).round() as u16)
        .collect()
}

fn psnr(a: &[u16], b: &[u16]) -> f64 {
    let mse = a
        .iter()
        .zip(b)
        .map(|(&a, &b)| (f64::from(a) - f64::from(b)).powi(2))
        .sum::<f64>()
        / a.len() as f64;
    10.0 * (65535.0f64.powi(2) / mse).log10()
}

struct Reference {
    stack: Image,
    stack_u16: Vec<u16>,
    neutral: Vec<u16>,
    layer1: Vec<u16>,
    layer1_settings: WaveletSettings,
}

fn reference() -> Option<Reference> {
    let dir = reference_dir().or_else(|| {
        eprintln!("re-local/reference not found, skipped");
        None
    })?;
    let (w, h, stack_u16) = read_tiff(dir.join("jupiter-stack.tif"));
    let (_, _, neutral) = read_tiff(dir.join("jupiter-wavelet-default.tif"));
    let (_, _, layer1) = read_tiff(dir.join("jupiter-wavelet-l1-10.tif"));
    let rwv = std::fs::read_to_string(dir.join("jupiter-l1-10.rwv")).unwrap();
    Some(Reference {
        stack: to_image(w, h, &stack_u16),
        stack_u16,
        neutral,
        layer1,
        layer1_settings: scheme::from_rwv(&rwv).unwrap(),
    })
}

#[test]
fn default_settings_match_the_stored_reference() {
    let Some(r) = reference() else { return };
    let result = to_u16(&do_all(
        &r.stack,
        &WaveletSettings::default(),
        ColourMode::Colour,
    ));
    let db = psnr(&result, &r.neutral);
    eprintln!("default settings against the reference: {db:.2} dB");
    assert!(db >= THRESHOLD_DB, "{db:.2} dB");
    assert!(db >= NEUTRAL_THRESHOLD_DB, "{db:.2} dB");
}

#[test]
fn layer_one_at_ten_matches_the_stored_reference() {
    let Some(r) = reference() else { return };
    let result = to_u16(&do_all(&r.stack, &r.layer1_settings, ColourMode::Colour));

    let db = psnr(&result, &r.layer1);
    let unsharpened = psnr(&r.stack_u16, &r.layer1);
    eprintln!(
        "Layer 1 = 10.0 against the reference: {db:.2} dB (the unsharpened stack: {unsharpened:.2} dB)"
    );
    assert!(db >= THRESHOLD_DB, "{db:.2} dB");
    assert!(db > unsharpened, "the result must be closer than the stack");
    assert!(db >= STRICT_THRESHOLD_DB, "{db:.2} dB");
}
