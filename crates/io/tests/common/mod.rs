//! Shared helpers of the regression tests: where `re-local/` is, the pipeline that produces the stored reference
//! results, and the comparison of 16-bit images. The generator (`reference_gen.rs`) and the tests use the same functions,
//! so a stored reference and the run that is checked against it can only differ by a change of the algorithms.

#![allow(dead_code)] // each test file uses a different part

use planets_core::{
    Image,
    align::{
        AlignParams, Control, PointParams, Reference, align_sequence, apply_threshold,
        default_threshold, find_alignpoints, rank_by_quality,
    },
    stack::{StackParams, stack_sequence},
    wavelet::WaveletSettings,
};
use planets_io::recording::Recording;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The first existing `re-local/` folder; the repository's own path if there is none (callers then skip).
///
/// Candidates: `$PLANETS_RE_LOCAL`, the repository itself, or - in a worktree below `.claude/worktrees/<name>/` - the
/// main checkout, five levels above `crates/io`.
pub fn re_local() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        std::env::var_os("PLANETS_RE_LOCAL").map(PathBuf::from),
        Some(manifest.join("../../re-local")),
        Some(manifest.join("../../../../../re-local")),
    ];
    let found = candidates.into_iter().flatten().find(|p| p.is_dir());
    found.unwrap_or_else(|| manifest.join("../../re-local"))
}

/// The Jupiter test recording (reference case R1), if it is there.
pub fn jupiter_video() -> Option<PathBuf> {
    Some(re_local().join("testdata/astropix/JUPITER.AVI")).filter(|p| p.exists())
}

/// Where the stored reference results live. They are produced by this program (`scripts/gen-reference.sh`).
pub fn reference_dir() -> PathBuf {
    re_local().join("reference-own")
}

/// The stored reference folder, if it has been generated.
pub fn existing_reference_dir() -> Option<PathBuf> {
    Some(reference_dir()).filter(|p| p.join("jupiter-stack.tif").exists())
}

/// Smallest PSNR (16-bit peak) a run may have against its stored reference. See `docs/stacking.md`.
///
/// Both sides come from the same code, so a run on the same machine matches bit for bit (infinite dB). What can differ
/// between platforms is float rounding (fused multiply-add, library maths, summation order), which moves single samples
/// by a few units of 65535, i.e. well above 90 dB. A change of the algorithms that is visible at all costs far more:
/// one grey level of 255 over the whole image is 48 dB, a quarter of a grey level 60 dB. 60 dB sits between the two.
pub const MIN_PSNR_DB: f64 = 60.0;

/// Stack of the recording as the app does it with the defaults: Set Alignpoints on frame 1, Align, all frames
/// (Limit keeps all ten of R1), Stack with local correction.
pub fn jupiter_stack(rec: &mut Recording) -> Image {
    let count = rec.info().frame_count;
    let first = rec.read_frame(0).unwrap();
    let candidates = find_alignpoints(&first, &PointParams::default());
    let points = apply_threshold(&candidates, default_threshold(candidates.len()));
    let reference = Reference::new(&first, points, &AlignParams::default());
    let frames: Vec<u32> = (0..count).collect();
    stack_sequence(
        &reference,
        &first,
        &StackParams::default(),
        &frames,
        |i| rec.read_frame(i),
        |_| Control::Continue,
    )
    .unwrap()
    .image
}

/// What the alignment of the recording is reduced to for the stored reference: counts and the quality order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlignSummary {
    /// Alignpoint candidates found on frame 1 with the default values.
    pub candidates: usize,
    /// Alignpoints left after the default threshold.
    pub points: usize,
    /// Frame numbers (1-based), best quality first.
    pub order: Vec<usize>,
}

/// Aligns the recording with the defaults and reduces the result to the summary.
pub fn jupiter_align(rec: &mut Recording) -> AlignSummary {
    let count = rec.info().frame_count;
    let first = rec.read_frame(0).unwrap();
    let candidates = find_alignpoints(&first, &PointParams::default());
    let points = apply_threshold(&candidates, default_threshold(candidates.len()));
    let (n_candidates, n_points) = (candidates.len(), points.len());
    let reference = Reference::new(&first, points, &AlignParams::default());
    let results = align_sequence(
        &reference,
        count,
        |i| rec.read_frame(i),
        |_| Control::Continue,
    )
    .unwrap();
    let order = rank_by_quality(&results).iter().map(|i| i + 1).collect();
    let summary = AlignSummary {
        candidates: n_candidates,
        points: n_points,
        order,
    };
    summary
}

/// The wavelet settings of the stored case "Layer 1 = 10.0": everything default, the slider of layer 1 at 10.0.
pub fn layer_one_at_ten() -> WaveletSettings {
    let mut settings = WaveletSettings::default();
    settings.layers[0].gain = 10.0;
    settings
}

/// Samples of an image as 16-bit values, the way *Save image* writes them.
pub fn to_u16(image: &Image) -> Vec<u16> {
    image
        .data
        .iter()
        .map(|v| (v.clamp(0.0, 1.0) * 65535.0).round() as u16)
        .collect()
}

/// A stored 16-bit RGB TIFF as an image (samples ÷ 65535) and as raw samples.
pub fn read_tiff(path: &std::path::Path) -> (Image, Vec<u16>) {
    let rgb = image::open(path).unwrap().into_rgb16();
    let samples = rgb.as_raw().clone();
    let image = Image {
        width: rgb.width(),
        height: rgb.height(),
        channels: 3,
        data: samples.iter().map(|&v| f32::from(v) / 65535.0).collect(),
    };
    (image, samples)
}

/// PSNR in dB between two 16-bit sample buffers (peak 65535); infinite if they are equal.
pub fn psnr_u16(a: &[u16], b: &[u16]) -> f64 {
    assert_eq!(a.len(), b.len(), "the images differ in size");
    let mse = a
        .iter()
        .zip(b)
        .map(|(&a, &b)| (f64::from(a) - f64::from(b)).powi(2))
        .sum::<f64>()
        / a.len() as f64;
    10.0 * (65535.0f64.powi(2) / mse).log10()
}
