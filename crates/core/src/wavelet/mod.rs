//! Wavelet sharpening, the core of the reference program (`docs/wavelets.md`, `docs/workflow.md` section 4).
//!
//! The image is split into six detail layers and a remainder. Layer `n` is the difference between the
//! image smoothed `n - 1` times and `n` times, so a layer holds the structures of one scale. Each layer
//! gets a gain (slider), optional denoising and sharpening, and everything is added up again. With
//! all gains at 1.0 the sum is the original image.
//!
//! Two ways to get there:
//! - [`process`] for *Do All*: streams through the layers and only needs three buffers.
//! - [`Decomposition`] for the live preview: keeps the layers of the visible area, so moving a slider
//!   only recombines them (see [`preview::Previewer`]).

pub mod preview;
pub mod scheme;
pub mod settings;

pub use preview::{Preview, Previewer, Region, rgba_region};
pub use settings::{
    DEFAULT_SHARPEN, Filter, LAYERS, LayerSettings, Scheme, Structure, WaveletSettings,
};

use crate::Image;
use rayon::prelude::*;

/// Pixels per parallel work item when walking over samples.
const CHUNK_PIXELS: usize = 4096;
/// Largest number of samples per channel used to estimate the noise of a layer.
const NOISE_SAMPLES: usize = 65_536;
/// Converts the median absolute deviation of normally distributed values to their standard deviation.
const MAD_TO_SIGMA: f32 = 1.4826;

/// Smooths `src` with a separable kernel whose taps are `spacing` pixels apart. Edges are replicated.
/// `tmp` is scratch space of the same size.
fn smooth(
    src: &[f32],
    out: &mut [f32],
    tmp: &mut [f32],
    (width, height, channels): (usize, usize, usize),
    taps: &[f32],
    spacing: usize,
) {
    let row_len = width * channels;
    let reach = (taps.len() / 2) as isize;
    let step = spacing as isize;

    // Horizontal pass: src -> tmp.
    tmp.par_chunks_mut(row_len)
        .zip(src.par_chunks(row_len))
        .for_each(|(dst, row)| {
            for x in 0..width {
                for c in 0..channels {
                    let mut acc = 0.0;
                    for (t, &w) in taps.iter().enumerate() {
                        let xx = (x as isize + (t as isize - reach) * step)
                            .clamp(0, width as isize - 1) as usize;
                        acc += w * row[xx * channels + c];
                    }
                    dst[x * channels + c] = acc;
                }
            }
        });

    // Vertical pass: tmp -> out, whole rows at a time.
    let tmp: &[f32] = tmp;
    out.par_chunks_mut(row_len)
        .enumerate()
        .for_each(|(y, dst)| {
            dst.fill(0.0);
            for (t, &w) in taps.iter().enumerate() {
                let yy = (y as isize + (t as isize - reach) * step).clamp(0, height as isize - 1)
                    as usize;
                let row = &tmp[yy * row_len..(yy + 1) * row_len];
                for (d, &s) in dst.iter_mut().zip(row) {
                    *d += w * s;
                }
            }
        });
}

/// Builds the first `count` layers of `image` and hands each to `visit` (index, samples).
fn for_each_layer(
    image: &Image,
    structure: &Structure,
    count: usize,
    mut visit: impl FnMut(usize, &[f32]),
) {
    let dims = (
        image.width as usize,
        image.height as usize,
        usize::from(image.channels).max(1),
    );
    if count == 0 || dims.0 == 0 || dims.1 == 0 {
        return;
    }
    let mut current = image.data.clone();
    let mut next = vec![0.0; current.len()];
    let mut scratch = vec![0.0; current.len()];
    for k in 0..count {
        smooth(
            &current,
            &mut next,
            &mut scratch,
            dims,
            structure.taps(),
            structure.spacing(k),
        );
        scratch
            .par_chunks_mut(CHUNK_PIXELS)
            .zip(
                current
                    .par_chunks(CHUNK_PIXELS)
                    .zip(next.par_chunks(CHUNK_PIXELS)),
            )
            .for_each(|(layer, (a, b))| {
                for ((l, &a), &b) in layer.iter_mut().zip(a).zip(b) {
                    *l = a - b;
                }
            });
        visit(k, &scratch);
        std::mem::swap(&mut current, &mut next);
    }
}

/// Noise level of a layer per channel: the robust standard deviation (median absolute deviation) of its samples.
fn noise_sigma(layer: &[f32], channels: usize) -> Vec<f32> {
    let pixels = layer.len() / channels;
    // An odd stride avoids always hitting the same column of the image.
    let stride = (pixels / NOISE_SAMPLES).max(1) | 1;
    (0..channels)
        .map(|c| {
            let mut samples: Vec<f32> = (0..pixels)
                .step_by(stride)
                .map(|i| layer[i * channels + c].abs())
                .collect();
            if samples.is_empty() {
                return 0.0;
            }
            let mid = samples.len() / 2;
            let (_, median, _) = samples.select_nth_unstable_by(mid, f32::total_cmp);
            *median * MAD_TO_SIGMA
        })
        .collect()
}

/// Adds the change one layer makes to `out`: `gain * denoised(layer) - layer`. Starting from the
/// unchanged image, adding this for every layer gives the result.
fn add_delta(
    out: &mut [f32],
    layer: &[f32],
    channels: usize,
    settings: &LayerSettings,
    noise: &[f32],
) {
    let gain = settings.effective_gain();
    let thresholds: Vec<f32> = noise
        .iter()
        .map(|s| s * settings.denoise.max(0.0))
        .collect();
    out.par_chunks_mut(CHUNK_PIXELS * channels)
        .zip(layer.par_chunks(CHUNK_PIXELS * channels))
        .for_each(|(out, layer)| {
            for (o, l) in out
                .chunks_exact_mut(channels)
                .zip(layer.chunks_exact(channels))
            {
                for c in 0..channels {
                    let v = l[c];
                    let shrunk = (v.abs() - thresholds[c]).max(0.0).copysign(v);
                    o[c] += gain * shrunk - v;
                }
            }
        });
}

/// Applies the settings to the whole image (*Do All*). The result is not clamped to `0.0..=1.0`.
pub fn process(image: &Image, settings: &WaveletSettings) -> Image {
    let settings = settings.clone().clamped();
    let mut result = image.clone();
    let channels = usize::from(image.channels).max(1);
    for_each_layer(
        image,
        &settings.structure,
        settings.layers_needed(),
        |k, layer| {
            let layer_settings = &settings.layers[k];
            if layer_settings.is_neutral() {
                return;
            }
            let noise = if layer_settings.denoise > 0.0 {
                noise_sigma(layer, channels)
            } else {
                vec![0.0; channels]
            };
            add_delta(&mut result.data, layer, channels, layer_settings, &noise);
        },
    );
    result
}

/// The layers of an image, kept so that a change of gain, *Denoise* or *Sharpen* only recombines them.
pub struct Decomposition {
    structure: Structure,
    base: Image,
    layers: Vec<Vec<f32>>,
    noise: Vec<Vec<f32>>,
}

impl Decomposition {
    /// Builds all [`LAYERS`] layers of `image` for the given structure.
    pub fn new(image: &Image, structure: Structure) -> Self {
        let channels = usize::from(image.channels).max(1);
        let mut layers = Vec::with_capacity(LAYERS);
        let mut noise = Vec::with_capacity(LAYERS);
        for_each_layer(image, &structure, LAYERS, |_, layer| {
            noise.push(noise_sigma(layer, channels));
            layers.push(layer.to_vec());
        });
        Self {
            structure,
            base: image.clone(),
            layers,
            noise,
        }
    }

    /// The structure the layers were built for. Settings with another one need a new decomposition.
    pub fn structure(&self) -> &Structure {
        &self.structure
    }

    /// Recombines the layers with the given gains. Equal to [`process`] on the image the layers came from.
    pub fn compose(&self, settings: &WaveletSettings) -> Image {
        let settings = settings.clone().clamped();
        let channels = usize::from(self.base.channels).max(1);
        let mut result = self.base.clone();
        for (k, layer) in self.layers.iter().enumerate() {
            let layer_settings = &settings.layers[k];
            if !layer_settings.is_neutral() {
                add_delta(
                    &mut result.data,
                    layer,
                    channels,
                    layer_settings,
                    &self.noise[k],
                );
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic test image with structure at several scales.
    fn test_image(width: u32, height: u32, channels: u8) -> Image {
        let mut img = Image::new(width, height, channels);
        let ch = usize::from(channels);
        for y in 0..height as usize {
            for x in 0..width as usize {
                for c in 0..ch {
                    let v = 0.5
                        + 0.2 * ((x as f32 * 0.9 + c as f32).sin() * (y as f32 * 0.7).cos())
                        + 0.2 * ((x as f32 * 0.05).sin() + (y as f32 * 0.04).cos()) / 2.0;
                    img.data[(y * width as usize + x) * ch + c] = v;
                }
            }
        }
        img
    }

    fn max_diff(a: &Image, b: &Image) -> f32 {
        a.data
            .iter()
            .zip(&b.data)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max)
    }

    fn boost(layer: usize, gain: f32) -> WaveletSettings {
        let mut s = WaveletSettings::default();
        s.layers[layer].gain = gain;
        s
    }

    #[test]
    fn default_settings_leave_the_image_unchanged() {
        let img = test_image(40, 30, 3);
        assert_eq!(process(&img, &WaveletSettings::default()), img);
    }

    #[test]
    fn a_flat_image_has_no_detail_to_amplify() {
        let mut flat = Image::new(20, 20, 1);
        flat.data.fill(0.4);
        let structure = WaveletSettings::default().structure;
        for_each_layer(&flat, &structure, LAYERS, |_, layer| {
            assert!(layer.iter().all(|v| v.abs() < 1e-6));
        });
        let out = process(&flat, &boost(0, 50.0));
        assert!(max_diff(&out, &flat) < 1e-5);
    }

    #[test]
    fn raising_layer_one_sharpens_the_finest_structures() {
        let img = test_image(64, 64, 1);
        let out = process(&img, &boost(0, 5.0));
        // The first layer is img - smooth(img); the result must be img + 4 * that layer.
        let structure = WaveletSettings::default().structure;
        let mut layer = Vec::new();
        for_each_layer(&img, &structure, 1, |_, l| layer = l.to_vec());
        for ((o, i), l) in out.data.iter().zip(&img.data).zip(&layer) {
            assert!((o - (i + 4.0 * l)).abs() < 1e-5);
        }
        assert!(max_diff(&out, &img) > 0.01);
    }

    #[test]
    fn gain_zero_removes_the_layer() {
        let img = test_image(64, 64, 1);
        let out = process(&img, &boost(0, 0.0));
        let structure = WaveletSettings::default().structure;
        let mut layer = Vec::new();
        for_each_layer(&img, &structure, 1, |_, l| layer = l.to_vec());
        for ((o, i), l) in out.data.iter().zip(&img.data).zip(&layer) {
            assert!((o - (i - l)).abs() < 1e-5);
        }
    }

    #[test]
    fn a_layer_that_is_off_is_not_applied() {
        let img = test_image(32, 32, 3);
        let mut s = boost(1, 8.0);
        s.layers[1].enabled = false;
        assert_eq!(process(&img, &s), img);
    }

    #[test]
    fn denoise_removes_small_detail_and_keeps_strong_edges() {
        // Fine ripple (small) next to a strong step edge. Denoising removes all of layer 1, so what is left
        // is the image smoothed once. The kernel 0.05 / 0.9 / 0.05 turns a checkerboard into
        // (0.9 - 0.1) = 0.8 of itself per axis, so 0.8 * 0.8 = 0.64 of the ripple stays. That holds in the
        // interior; in the first and last row the replicated edge row breaks the cancellation (0.9 instead
        // of 0.8 vertically), so the ripple is measured in row 4.
        let mut img = Image::new(64, 8, 1);
        for y in 0..8usize {
            for x in 0..64usize {
                let edge = if x < 32 { 0.2 } else { 0.8 };
                let noise = if (x + y) % 2 == 0 { 0.004 } else { -0.004 };
                img.data[y * 64 + x] = edge + noise;
            }
        }
        let mut s = WaveletSettings::default();
        s.layers[0].denoise = 3.0;
        let out = process(&img, &s);
        let ripple = |im: &Image| (im.data[4 * 64 + 10] - im.data[4 * 64 + 11]).abs();
        let kept = ripple(&out) / ripple(&img);
        assert!(
            (kept - 0.64).abs() < 0.02,
            "{kept} of the ripple is left, expected 0.64"
        );
        // The step survives.
        assert!((out.data[40] - out.data[24]) > 0.5);
    }

    #[test]
    fn decomposition_compose_equals_process() {
        let img = test_image(50, 40, 3);
        let mut s = WaveletSettings::default();
        s.layers[0].gain = 6.0;
        s.layers[2].gain = -1.0;
        s.layers[3].denoise = 1.0;
        s.layers[3].gain = 2.0;
        s.layers[5].sharpen = 0.2;
        s.layers[5].gain = 3.0;
        let decomposition = Decomposition::new(&img, s.structure);
        assert!(max_diff(&decomposition.compose(&s), &process(&img, &s)) < 1e-5);
    }

    #[test]
    fn dyadic_and_the_default_filter_also_sum_correctly() {
        let img = test_image(60, 60, 1);
        let mut s = boost(2, 4.0);
        s.structure.scheme = Scheme::Dyadic;
        s.structure.filter = Filter::Default;
        let decomposition = Decomposition::new(&img, s.structure);
        assert!(max_diff(&decomposition.compose(&s), &process(&img, &s)) < 1e-5);
        assert_ne!(process(&img, &s), img);
    }

    #[test]
    fn colour_channels_are_processed_independently() {
        let img = test_image(40, 40, 3);
        let mono_of = |c: usize| Image {
            width: 40,
            height: 40,
            channels: 1,
            data: img.data.iter().skip(c).step_by(3).copied().collect(),
        };
        let s = boost(0, 4.0);
        let out = process(&img, &s);
        for c in 0..3 {
            let expected = process(&mono_of(c), &s);
            for (i, e) in expected.data.iter().enumerate() {
                assert!((out.data[i * 3 + c] - e).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn noise_estimate_matches_the_spread_of_the_samples() {
        // Alternating +-0.5 has a median absolute value of 0.5.
        let layer: Vec<f32> = (0..1000)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        let sigma = noise_sigma(&layer, 1);
        assert!((sigma[0] - 0.5 * MAD_TO_SIGMA).abs() < 1e-6);
    }
}
