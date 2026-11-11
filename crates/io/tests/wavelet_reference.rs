//! Compares wavelet sharpening with the results the reference program gave for reference case R1 (`docs/testdaten.md`,
//! `docs/wavelets.md`). The reference files live in `re-local/` and are not in the repository; without
//! them the tests are skipped. Input is the original's stacked image, so the stacking does not count.

use planets_core::{
    Image,
    wavelet::{WaveletSettings, process, scheme},
};
use std::path::PathBuf;

/// Smallest PSNR (16-bit peak) the ticket asks for, `docs/wavelets.md`.
const THRESHOLD_DB: f64 = 45.0;
/// Threshold after the tone curve of the original is taken out. The stack itself is 55.6 dB away from the
/// original's neutral result and 52.8 dB from the Layer 1 = 10 result, so only a value clearly above that
/// shows that the sharpening is right.
const STRICT_THRESHOLD_DB: f64 = 65.0;

fn reference_dir() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [
        std::env::var_os("PLANETS_RE_LOCAL").map(PathBuf::from),
        Some(manifest.join("../../re-local")),
        // In a worktree below `.claude/worktrees/<name>/`.
        Some(manifest.join("../../../../re-local")),
    ]
    .into_iter()
    .flatten()
    .map(|p| p.join("reference"))
    .find(|p| p.join("jupiter-stack.tif").exists())
}

/// 16-bit RGB samples as written by the original.
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

/// The tone curve the original applies even with neutral settings (a function of the sample value
/// alone, the same for all channels), read from the stack and the neutral result.
fn tone_curve(stack: &[u16], neutral: &[u16]) -> impl Fn(u16) -> u16 {
    let mut points: Vec<(u16, u16)> = stack.iter().copied().zip(neutral.iter().copied()).collect();
    points.sort_unstable();
    points.dedup_by_key(|p| p.0);
    move |x| match points.binary_search_by_key(&x, |p| p.0) {
        Ok(i) => points[i].1,
        Err(0) => points[0].1,
        Err(i) if i == points.len() => points[i - 1].1,
        Err(i) => {
            let ((x0, y0), (x1, y1)) = (points[i - 1], points[i]);
            let t = f64::from(x - x0) / f64::from(x1 - x0);
            (f64::from(y0) + t * (f64::from(y1) - f64::from(y0))).round() as u16
        }
    }
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
fn default_settings_match_the_original_within_its_tone_curve() {
    let Some(r) = reference() else { return };
    let result = process(&r.stack, &WaveletSettings::default());
    assert!(
        result == r.stack,
        "default settings must not change the image"
    );
    let db = psnr(&to_u16(&result), &r.neutral);
    eprintln!("default settings against the original: {db:.2} dB");
    assert!(db >= THRESHOLD_DB, "{db:.2} dB");
}

#[test]
fn layer_one_at_ten_matches_the_original() {
    let Some(r) = reference() else { return };
    let result = to_u16(&process(&r.stack, &r.layer1_settings));

    let db = psnr(&result, &r.layer1);
    let unsharpened = psnr(&r.stack_u16, &r.layer1);
    eprintln!(
        "Layer 1 = 10.0 against the original: {db:.2} dB (the unsharpened stack: {unsharpened:.2} dB)"
    );
    assert!(db >= THRESHOLD_DB, "{db:.2} dB");
    assert!(db > unsharpened, "the result must be closer than the stack");

    let curve = tone_curve(&r.stack_u16, &r.neutral);
    let toned: Vec<u16> = result.iter().map(|&v| curve(v)).collect();
    let strict = psnr(&toned, &r.layer1);
    eprintln!("with the tone curve of the original: {strict:.2} dB");
    assert!(strict >= STRICT_THRESHOLD_DB, "{strict:.2} dB");
}
