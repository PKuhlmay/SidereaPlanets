//! Denoise/Deringing and Masking: the three finishing steps behind the *Adjustments* buttons of the Wavelet tab that work on the
//! sharpened image and the unsharpened stack it came from (`docs/wavelets.md`, "Denoise/Deringing and Masking").
//!
//! The original has no such panels in the version we studied, so the algorithms are our own design. All of them take samples
//! normalised to `0.0..=1.0`, never change the size of the image and return it unchanged (bit for bit) when their strength is zero.

use crate::Image;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// The *Masking* card: where the sharpening applies.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MaskSettings {
    /// Off: the whole image is sharpened.
    pub enabled: bool,
    /// Brightness of the unsharpened stack on a 0..255 scale: what is at least this bright (the planet) gets the sharpening.
    pub threshold: f32,
    /// Width of the soft edge of the mask in pixels (0 = hard edge).
    pub feather: f32,
    /// Sharpen what lies below the threshold instead.
    pub invert: bool,
}

impl Default for MaskSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 20.0,
            feather: 4.0,
            invert: false,
        }
    }
}

/// The *Denoise/Deringing* and *Masking* cards together.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RefineSettings {
    /// *Denoise* strength in percent (0 = off).
    pub denoise: f32,
    /// *Deringing* amount in percent (0 = off).
    pub deringing: f32,
    /// *Deringing* radius in pixels: how far from an edge the dark ring is looked for.
    pub deringing_radius: u32,
    pub mask: MaskSettings,
}

impl Default for RefineSettings {
    fn default() -> Self {
        Self {
            denoise: 0.0,
            deringing: 0.0,
            deringing_radius: 4,
            mask: MaskSettings::default(),
        }
    }
}

/// The ranges the cards offer.
pub const DERINGING_RADIUS: std::ops::RangeInclusive<u32> = 1..=10;
pub const MASK_FEATHER_MAX: f32 = 50.0;

impl RefineSettings {
    /// True when no step changes the image.
    pub fn is_neutral(&self) -> bool {
        self.denoise <= 0.0 && self.deringing <= 0.0 && !self.mask.enabled
    }

    /// The settings cut to the ranges of the cards; a value that is not a number becomes the default.
    pub fn clamped(&self) -> Self {
        let d = Self::default();
        let num = |v: f32, default: f32, lo: f32, hi: f32| {
            if v.is_finite() {
                v.clamp(lo, hi)
            } else {
                default
            }
        };
        Self {
            denoise: num(self.denoise, d.denoise, 0.0, 100.0),
            deringing: num(self.deringing, d.deringing, 0.0, 100.0),
            deringing_radius: self
                .deringing_radius
                .clamp(*DERINGING_RADIUS.start(), *DERINGING_RADIUS.end()),
            mask: MaskSettings {
                enabled: self.mask.enabled,
                threshold: num(self.mask.threshold, d.mask.threshold, 0.0, 255.0),
                feather: num(self.mask.feather, d.mask.feather, 0.0, MASK_FEATHER_MAX),
                invert: self.mask.invert,
            },
        }
    }
}

/// Deringing, then Denoise, then the mask: `sharpened` is the image to finish, `reference` the unsharpened stack it was made from.
/// Without a reference of the same size and channels only Denoise runs (Deringing and the mask compare with it).
pub fn refine(sharpened: &Image, reference: Option<&Image>, settings: &RefineSettings) -> Image {
    let s = settings.clamped();
    let adapted;
    let reference = match reference {
        Some(r) if same_shape(r, sharpened) => Some(r),
        Some(r) => {
            adapted = with_channels(r, sharpened.channels);
            adapted.as_ref()
        }
        None => None,
    };
    let mut out = sharpened.clone();
    if s.deringing > 0.0
        && let Some(reference) = reference
    {
        out = deringing(&out, reference, s.deringing / 100.0, s.deringing_radius);
    }
    if s.denoise > 0.0 {
        out = denoise(&out, s.denoise / 100.0);
    }
    if s.mask.enabled
        && let Some(reference) = reference
    {
        out = blend_by_mask(&out, reference, &mask(reference, &s.mask));
    }
    out
}

fn same_shape(a: &Image, b: &Image) -> bool {
    a.width == b.width && a.height == b.height && a.channels == b.channels
}

/// `reference` as an image with `channels` channels, when it has the size of the image and only the channels differ: the window
/// holds every image as RGB, also the one of a gray recording, whose stack has one channel. `None` for anything else.
fn with_channels(reference: &Image, channels: u8) -> Option<Image> {
    match (reference.channels, channels) {
        (1, 3) => Some(Image {
            width: reference.width,
            height: reference.height,
            channels,
            data: reference.data.iter().flat_map(|&v| [v; 3]).collect(),
        }),
        (from, 1) if from >= 3 => Some(reference.to_luma()),
        _ => None,
    }
}

// --- Deringing ----------------------------------------------------------------------------------

/// Below this local contrast of the reference (highest minus lowest value within the radius) a sample is not near an edge and
/// Deringing leaves it alone; from [`EDGE_FULL`] on it works at full strength; in between it fades in smoothly. A planet limb
/// has a contrast of 0.3 and more, the belts and craters of a planet 0.05 to 0.15.
const EDGE_NONE: f32 = 0.10;
const EDGE_FULL: f32 = 0.30;

/// Dark rings: raises every sample of `sharpened` that fell below the lowest value `reference` has within `radius` pixels, but only
/// near strong edges of the reference. `amount` 0..1.
///
/// For a sample `s` with the lowest `lo` and highest `hi` reference value in the square window of `radius` around it and the edge
/// weight `w` = smoothstep of `hi - lo` between [`EDGE_NONE`] and [`EDGE_FULL`]: `s' = s + amount * w * max(0, lo - s)`.
/// Samples are only ever raised, and only those below the original minimum.
pub fn deringing(sharpened: &Image, reference: &Image, amount: f32, radius: u32) -> Image {
    let mut out = sharpened.clone();
    if amount <= 0.0 || !same_shape(sharpened, reference) {
        return out;
    }
    let amount = amount.min(1.0);
    let radius = radius.max(1) as usize;
    let (w, h, c) = (
        sharpened.width as usize,
        sharpened.height as usize,
        usize::from(sharpened.channels).max(1),
    );
    let lo = window_extreme(&reference.data, w, h, c, radius, f32::min);
    let hi = window_extreme(&reference.data, w, h, c, radius, f32::max);
    out.data
        .par_iter_mut()
        .zip(lo.par_iter().zip(hi.par_iter()))
        .for_each(|(s, (&lo, &hi))| {
            let under = lo - *s;
            if under > 0.0 {
                let t = ((hi - lo - EDGE_NONE) / (EDGE_FULL - EDGE_NONE)).clamp(0.0, 1.0);
                *s += amount * (t * t * (3.0 - 2.0 * t)) * under;
            }
        });
    out
}

/// The lowest or highest value (`pick` is `f32::min` or `f32::max`) of every sample within `radius` pixels in x and y, per channel.
/// Two passes of a one dimensional window; the window is cut at the border of the image.
fn window_extreme(
    data: &[f32],
    width: usize,
    height: usize,
    channels: usize,
    radius: usize,
    pick: fn(f32, f32) -> f32,
) -> Vec<f32> {
    let row_len = width * channels;
    let mut across = vec![0.0f32; data.len()];
    across
        .par_chunks_mut(row_len)
        .zip(data.par_chunks(row_len))
        .for_each(|(out, row)| {
            for x in 0..width {
                let from = x.saturating_sub(radius);
                let to = (x + radius).min(width - 1);
                for ch in 0..channels {
                    let mut v = row[from * channels + ch];
                    for xx in from + 1..=to {
                        v = pick(v, row[xx * channels + ch]);
                    }
                    out[x * channels + ch] = v;
                }
            }
        });
    let mut out = vec![0.0f32; data.len()];
    out.par_chunks_mut(row_len)
        .enumerate()
        .for_each(|(y, out)| {
            let from = y.saturating_sub(radius);
            let to = (y + radius).min(height - 1);
            out.copy_from_slice(&across[from * row_len..(from + 1) * row_len]);
            for yy in from + 1..=to {
                for (o, &v) in out
                    .iter_mut()
                    .zip(&across[yy * row_len..(yy + 1) * row_len])
                {
                    *o = pick(*o, v);
                }
            }
        });
    out
}

// --- Denoise ------------------------------------------------------------------------------------

/// Radius of the window of [`denoise`] in pixels (a 5 x 5 window).
const DENOISE_RADIUS: i64 = 2;
/// Standard deviation of the spatial weights in pixels.
const DENOISE_SIGMA_SPACE: f32 = 1.2;
/// Standard deviation of the range weights, in sample units, at strength 0 and 1: the colour distance up to which neighbours still
/// count as the same surface. The larger the strength, the more difference is counted as noise.
const DENOISE_SIGMA_RANGE: (f32, f32) = (0.01, 0.15);

/// Smooths noise without blurring edges: a bilateral filter over a 5 x 5 window, mixed with the original by `strength` (0..1).
///
/// Every neighbour counts with the weight `exp(-d^2 / 2 sigma_s^2) * exp(-e^2 / 2 sigma_r^2)`: `d` is its distance in pixels and `e`
/// the root of the mean squared difference of all its channels to the centre sample, so a neighbour across a limb or a colour edge gets
/// no weight and the edge stays. `sigma_r` grows from 0.01 to 0.15 with the strength. The result is the weighted mean, mixed
/// with the original sample by `strength`.
pub fn denoise(image: &Image, strength: f32) -> Image {
    let strength = if strength.is_finite() {
        strength.clamp(0.0, 1.0)
    } else {
        0.0
    };
    if strength <= 0.0 {
        return image.clone();
    }
    let (w, h, c) = (
        image.width as usize,
        image.height as usize,
        usize::from(image.channels).max(1),
    );
    let sigma_r =
        DENOISE_SIGMA_RANGE.0 + (DENOISE_SIGMA_RANGE.1 - DENOISE_SIGMA_RANGE.0) * strength;
    let range_scale = -0.5 / (sigma_r * sigma_r);
    let space_scale = -0.5 / (DENOISE_SIGMA_SPACE * DENOISE_SIGMA_SPACE);
    let side = (2 * DENOISE_RADIUS + 1) as usize;
    let space: Vec<f32> = (0..side * side)
        .map(|i| {
            let dx = (i % side) as f32 - DENOISE_RADIUS as f32;
            let dy = (i / side) as f32 - DENOISE_RADIUS as f32;
            ((dx * dx + dy * dy) * space_scale).exp()
        })
        .collect();
    let mut out = image.clone();
    let src = &image.data;
    out.data
        .par_chunks_mut(w * c)
        .enumerate()
        .for_each(|(y, row)| {
            let mut sum = vec![0.0f32; c];
            for x in 0..w {
                let centre = &src[(y * w + x) * c..(y * w + x + 1) * c];
                sum.fill(0.0);
                let mut total = 0.0f32;
                for ky in 0..side {
                    let yy = y as i64 + ky as i64 - DENOISE_RADIUS;
                    if yy < 0 || yy >= h as i64 {
                        continue;
                    }
                    for kx in 0..side {
                        let xx = x as i64 + kx as i64 - DENOISE_RADIUS;
                        if xx < 0 || xx >= w as i64 {
                            continue;
                        }
                        let at = (yy as usize * w + xx as usize) * c;
                        let px = &src[at..at + c];
                        let e2 = px
                            .iter()
                            .zip(centre)
                            .map(|(a, b)| (a - b) * (a - b))
                            .sum::<f32>()
                            / c as f32;
                        let weight = space[ky * side + kx] * (e2 * range_scale).exp();
                        total += weight;
                        for (s, &v) in sum.iter_mut().zip(px) {
                            *s += weight * v;
                        }
                    }
                }
                for ch in 0..c {
                    let smooth = sum[ch] / total;
                    row[x * c + ch] = centre[ch] + strength * (smooth - centre[ch]);
                }
            }
        });
    out
}

// --- Mask ---------------------------------------------------------------------------------------

/// The mask of the sharpening, one value 0..1 per pixel (1 = fully sharpened), from the brightness of `reference`.
///
/// A pixel is inside where its luminance is at least `threshold / 255`; that binary mask is softened with a Gaussian of standard
/// deviation `feather / 2` pixels (a hard edge for a feather of 0) and, with `invert`, turned around. A mask that is the same
/// everywhere stays exactly 0 or 1.
pub fn mask(reference: &Image, settings: &MaskSettings) -> Vec<f32> {
    let (w, h) = (reference.width as usize, reference.height as usize);
    let limit = settings.threshold.clamp(0.0, 255.0) / 255.0;
    let mut m: Vec<f32> = reference
        .to_luma()
        .data
        .iter()
        .map(|&l| if l >= limit { 1.0 } else { 0.0 })
        .collect();
    let sigma = settings.feather.clamp(0.0, MASK_FEATHER_MAX) / 2.0;
    if sigma > 0.0 && w > 0 && h > 0 {
        m = gaussian_blur(&m, w, h, sigma);
        for v in &mut m {
            // Rounding of the weights must not leave 0.9999999 where the mask is 1.
            *v = if *v >= 1.0 - 1e-5 {
                1.0
            } else if *v <= 1e-5 {
                0.0
            } else {
                *v
            };
        }
    }
    if settings.invert {
        for v in &mut m {
            *v = 1.0 - *v;
        }
    }
    m
}

/// A separable Gaussian of standard deviation `sigma` (cut at three sigma) over a one channel plane; the border repeats its edge.
fn gaussian_blur(plane: &[f32], width: usize, height: usize, sigma: f32) -> Vec<f32> {
    let reach = (3.0 * sigma).ceil() as i64;
    let mut kernel: Vec<f32> = (-reach..=reach)
        .map(|i| (-(i * i) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let total: f32 = kernel.iter().sum();
    for k in &mut kernel {
        *k /= total;
    }
    let mut across = vec![0.0f32; plane.len()];
    across
        .par_chunks_mut(width)
        .zip(plane.par_chunks(width))
        .for_each(|(out, row)| {
            for (x, o) in out.iter_mut().enumerate() {
                *o = kernel
                    .iter()
                    .enumerate()
                    .map(|(k, &wgt)| {
                        let xx = (x as i64 + k as i64 - reach).clamp(0, width as i64 - 1);
                        wgt * row[xx as usize]
                    })
                    .sum();
            }
        });
    let mut out = vec![0.0f32; plane.len()];
    out.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
        for (k, &wgt) in kernel.iter().enumerate() {
            let yy = (y as i64 + k as i64 - reach).clamp(0, height as i64 - 1) as usize;
            for (o, &v) in row.iter_mut().zip(&across[yy * width..(yy + 1) * width]) {
                *o += wgt * v;
            }
        }
    });
    out
}

/// `reference` where the mask is 0, `sharpened` where it is 1, in between `reference + mask * (sharpened - reference)`. The two images
/// must have the same size; the mask one value per pixel. Otherwise `sharpened` is returned.
pub fn blend_by_mask(sharpened: &Image, reference: &Image, mask: &[f32]) -> Image {
    let pixels = sharpened.width as usize * sharpened.height as usize;
    if !same_shape(sharpened, reference) || mask.len() != pixels {
        return sharpened.clone();
    }
    let c = usize::from(sharpened.channels).max(1);
    let mut out = sharpened.clone();
    out.data
        .par_chunks_mut(c)
        .zip(reference.data.par_chunks(c))
        .zip(mask.par_iter())
        .for_each(|((px, base), &m)| {
            if m <= 0.0 {
                px.copy_from_slice(base);
            } else if m < 1.0 {
                for (v, &b) in px.iter_mut().zip(base) {
                    *v = b + m * (*v - b);
                }
            }
        });
    out
}

/// The colour the mask is shown in over the image.
const OVERLAY_TINT: [f32; 3] = [0.1, 0.6, 1.0];
/// How strongly the tint covers the image where the mask is 1.
const OVERLAY_OPACITY: f32 = 0.55;

/// `image` with the mask tinted blue on top, for the preview of the card: where the mask is 1 the sharpening applies. Always three
/// channels (a gray image becomes a colour one).
pub fn mask_overlay(image: &Image, mask: &[f32]) -> Image {
    let c = usize::from(image.channels).max(1);
    let mut out = Image::new(image.width, image.height, 3);
    // A mask of another size covers nothing.
    let none;
    let mask = if mask.len() == image.width as usize * image.height as usize {
        mask
    } else {
        none = vec![0.0; image.width as usize * image.height as usize];
        &none
    };
    out.data
        .par_chunks_mut(3)
        .zip(image.data.par_chunks(c))
        .zip(mask.par_iter())
        .for_each(|((px, src), &m)| {
            let alpha = OVERLAY_OPACITY * m.clamp(0.0, 1.0);
            for (ch, v) in px.iter_mut().enumerate() {
                let base = if c >= 3 { src[ch] } else { src[0] };
                *v = base + alpha * (OVERLAY_TINT[ch] - base);
            }
        });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: u32 = 24;

    /// A 1 channel image of `W` x `h` whose value only depends on the column.
    fn columns(h: u32, f: impl Fn(u32) -> f32) -> Image {
        let mut image = Image::new(W, h, 1);
        for (i, v) in image.data.iter_mut().enumerate() {
            *v = f(i as u32 % W);
        }
        image
    }

    /// Dark sky left of column 12, a bright planet right of it.
    fn limb(x: u32) -> f32 {
        if x < 12 { 0.05 } else { 0.8 }
    }

    /// What sharpening does at a limb: a dip below the sky next to the edge and an overshoot above the planet.
    fn ringing(x: u32) -> f32 {
        match x {
            9..=11 => 0.0,
            12..=13 => 0.9,
            _ => limb(x),
        }
    }

    /// Pseudo random noise in -0.5..0.5, always the same.
    fn noise(i: usize) -> f32 {
        let mut x = (i as u32).wrapping_mul(2_654_435_761).wrapping_add(12_345);
        x ^= x >> 15;
        x = x.wrapping_mul(2_246_822_519);
        x ^= x >> 13;
        (x % 10_000) as f32 / 10_000.0 - 0.5
    }

    fn std_dev(values: &[f32]) -> f32 {
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        (values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32).sqrt()
    }

    /// The lowest value of `image` within `radius` pixels (square window), found the slow way.
    fn local_min(image: &Image, x: u32, y: u32, radius: u32) -> f32 {
        let mut lowest = f32::MAX;
        for yy in y.saturating_sub(radius)..=(y + radius).min(image.height - 1) {
            for xx in x.saturating_sub(radius)..=(x + radius).min(image.width - 1) {
                lowest = lowest.min(image.data[(yy * image.width + xx) as usize]);
            }
        }
        lowest
    }

    // --- Settings -------------------------------------------------------------------------------

    #[test]
    fn the_defaults_change_nothing() {
        assert!(RefineSettings::default().is_neutral());
        let mut s = RefineSettings::default();
        s.mask.enabled = true;
        assert!(!s.is_neutral());
        let s = RefineSettings {
            denoise: 1.0,
            ..RefineSettings::default()
        };
        assert!(!s.is_neutral());
        let s = RefineSettings {
            deringing: 1.0,
            ..RefineSettings::default()
        };
        assert!(!s.is_neutral());
    }

    #[test]
    fn values_are_cut_to_the_ranges_of_the_cards() {
        let c = RefineSettings {
            denoise: 250.0,
            deringing: -3.0,
            deringing_radius: 99,
            mask: MaskSettings {
                enabled: true,
                threshold: f32::NAN,
                feather: 500.0,
                invert: true,
            },
        }
        .clamped();
        assert_eq!(
            (c.denoise, c.deringing, c.deringing_radius),
            (100.0, 0.0, 10)
        );
        assert_eq!(c.mask.threshold, MaskSettings::default().threshold);
        assert_eq!(c.mask.feather, MASK_FEATHER_MAX);
        assert!(c.mask.enabled && c.mask.invert);
    }

    #[test]
    fn the_settings_read_from_partial_json() {
        let s: RefineSettings =
            serde_json::from_str(r#"{"denoise": 30, "mask": {"enabled": true}}"#).unwrap();
        assert_eq!(s.denoise, 30.0);
        assert_eq!(s.deringing_radius, 4);
        assert!(s.mask.enabled);
        assert_eq!(s.mask.threshold, 20.0);
        let none: RefineSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(none, RefineSettings::default());
    }

    // --- Deringing ------------------------------------------------------------------------------

    #[test]
    fn deringing_never_leaves_a_sample_below_the_local_minimum_of_the_original() {
        let reference = columns(5, limb);
        let sharpened = columns(5, ringing);
        let out = deringing(&sharpened, &reference, 1.0, 3);
        for y in 0..5 {
            for x in 0..W {
                let i = (y * W + x) as usize;
                let floor = local_min(&reference, x, y, 3);
                assert!(
                    out.data[i] >= floor.min(sharpened.data[i]),
                    "({x},{y}): {} below the original minimum {floor}",
                    out.data[i]
                );
                assert!(out.data[i] >= sharpened.data[i], "({x},{y}) got darker");
            }
        }
        // The dip next to the limb is gone: it is back at the level of the sky.
        assert_eq!(out.data[(2 * W + 10) as usize], 0.05);
    }

    #[test]
    fn deringing_only_raises_what_fell_below_and_leaves_the_overshoot() {
        let reference = columns(3, limb);
        let sharpened = columns(3, ringing);
        let out = deringing(&sharpened, &reference, 1.0, 3);
        for x in 9..12 {
            assert!(
                out.data[(W + x) as usize] > 0.0,
                "the dip at column {x} is raised"
            );
        }
        for x in (0..9).chain(12..W) {
            let i = (W + x) as usize;
            assert_eq!(out.data[i], sharpened.data[i], "column {x} changed");
        }
    }

    #[test]
    fn deringing_amount_is_the_share_of_the_correction() {
        let reference = columns(3, limb);
        let sharpened = columns(3, ringing);
        let i = (W + 10) as usize;
        let full = deringing(&sharpened, &reference, 1.0, 3).data[i];
        let half = deringing(&sharpened, &reference, 0.5, 3).data[i];
        assert_eq!(full, 0.05, "the full correction lifts the dip to the sky");
        assert!(
            (half - (0.0 + full) / 2.0).abs() < 1e-6,
            "half {half}, full {full}"
        );
    }

    #[test]
    fn deringing_leaves_weak_edges_alone() {
        // A texture of amplitude 0.06: sharpening may make it deeper, this is detail and not a ring.
        let reference = columns(5, |x| 0.4 + if x % 2 == 0 { 0.03 } else { -0.03 });
        let sharpened = columns(5, |x| 0.4 + if x % 2 == 0 { 0.06 } else { -0.06 });
        assert_eq!(deringing(&sharpened, &reference, 1.0, 3), sharpened);
    }

    #[test]
    fn deringing_at_zero_returns_the_image_bit_exact() {
        let reference = columns(5, limb);
        let sharpened = columns(5, ringing);
        assert_eq!(deringing(&sharpened, &reference, 0.0, 3), sharpened);
    }

    #[test]
    fn deringing_works_on_every_channel_of_a_colour_image() {
        let mut reference = Image::new(W, 3, 3);
        let mut sharpened = Image::new(W, 3, 3);
        for (i, px) in reference.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
            px.fill(limb(i as u32 % W));
        }
        for (i, px) in sharpened.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
            px.fill(ringing(i as u32 % W));
        }
        let out = deringing(&sharpened, &reference, 1.0, 3);
        let at = (W + 10) as usize * 3;
        assert_eq!(&out.data[at..at + 3], &[0.05, 0.05, 0.05]);
    }

    // --- Denoise --------------------------------------------------------------------------------

    fn noisy_flat(level: f32, amplitude: f32) -> Image {
        let mut image = Image::new(W, 16, 1);
        for (i, v) in image.data.iter_mut().enumerate() {
            *v = level + amplitude * noise(i);
        }
        image
    }

    #[test]
    fn denoise_at_zero_returns_the_image_bit_exact() {
        let image = noisy_flat(0.5, 0.1);
        assert_eq!(denoise(&image, 0.0), image);
    }

    #[test]
    fn denoise_reduces_noise_in_a_flat_area() {
        let image = noisy_flat(0.5, 0.08);
        let out = denoise(&image, 1.0);
        assert!(
            std_dev(&out.data) < 0.6 * std_dev(&image.data),
            "{} against {}",
            std_dev(&out.data),
            std_dev(&image.data)
        );
    }

    #[test]
    fn denoise_removes_more_noise_with_more_strength() {
        let image = noisy_flat(0.5, 0.08);
        let deviations: Vec<f32> = [0.0, 0.25, 0.5, 1.0]
            .map(|s| std_dev(&denoise(&image, s).data))
            .to_vec();
        for pair in deviations.windows(2) {
            assert!(pair[1] < pair[0], "{deviations:?}");
        }
    }

    #[test]
    fn denoise_keeps_a_strong_edge() {
        let image = columns(8, limb);
        let out = denoise(&image, 1.0);
        for (a, b) in out.data.iter().zip(&image.data) {
            assert!((a - b).abs() < 0.01, "{a} against {b}");
        }
    }

    #[test]
    fn denoise_leaves_a_flat_image_flat() {
        let image = columns(8, |_| 0.37);
        for (a, b) in denoise(&image, 1.0).data.iter().zip(&image.data) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    // --- Mask -----------------------------------------------------------------------------------

    fn settings(threshold: f32, feather: f32, invert: bool) -> MaskSettings {
        MaskSettings {
            enabled: true,
            threshold,
            feather,
            invert,
        }
    }

    #[test]
    fn a_threshold_of_zero_selects_everything_and_the_top_of_the_scale_nothing() {
        let reference = columns(6, limb);
        assert!(
            mask(&reference, &settings(0.0, 0.0, false))
                .iter()
                .all(|&m| m == 1.0)
        );
        assert!(
            mask(&reference, &settings(255.0, 0.0, false))
                .iter()
                .all(|&m| m == 0.0)
        );
        // The same with a soft edge: a uniform mask stays exactly 0 or 1.
        assert!(
            mask(&reference, &settings(0.0, 7.0, false))
                .iter()
                .all(|&m| m == 1.0)
        );
        assert!(
            mask(&reference, &settings(255.0, 7.0, false))
                .iter()
                .all(|&m| m == 0.0)
        );
    }

    #[test]
    fn a_hard_mask_follows_the_threshold_and_invert_turns_it_around() {
        // 0.05 is 12.75 on the 0..255 scale, 0.8 is 204.
        let reference = columns(4, limb);
        let m = mask(&reference, &settings(100.0, 0.0, false));
        let inverted = mask(&reference, &settings(100.0, 0.0, true));
        for (i, (&a, &b)) in m.iter().zip(&inverted).enumerate() {
            let planet = i as u32 % W >= 12;
            assert_eq!(a, if planet { 1.0 } else { 0.0 }, "pixel {i}");
            assert_eq!(b, 1.0 - a, "pixel {i}");
        }
    }

    #[test]
    fn the_mask_is_made_from_the_brightness_of_a_colour_image() {
        let mut reference = Image::new(2, 1, 3);
        reference
            .data
            .copy_from_slice(&[0.9, 0.9, 0.9, 0.1, 0.1, 0.1]);
        assert_eq!(
            mask(&reference, &settings(128.0, 0.0, false)),
            vec![1.0, 0.0]
        );
    }

    #[test]
    fn feathering_makes_the_edge_rise_steadily_and_wider_with_the_radius() {
        let reference = columns(9, limb);
        let mut widths = Vec::new();
        for feather in [1.0, 3.0, 6.0, 12.0] {
            let m = mask(&reference, &settings(100.0, feather, false));
            // The middle row, from the sky to the planet.
            let row = &m[(4 * W) as usize..(5 * W) as usize];
            for pair in row.windows(2) {
                assert!(pair[1] >= pair[0], "feather {feather}: {row:?}");
            }
            assert!(row.iter().all(|m| (0.0..=1.0).contains(m)));
            widths.push(row.iter().filter(|&&m| m > 0.0 && m < 1.0).count());
        }
        assert!(widths[0] > 0, "{widths:?}");
        for pair in widths.windows(2) {
            assert!(pair[1] >= pair[0], "{widths:?}");
        }
        assert!(widths[3] > widths[0], "{widths:?}");
    }

    #[test]
    fn a_soft_edge_is_half_way_at_the_threshold_line() {
        let reference = columns(9, limb);
        let m = mask(&reference, &settings(100.0, 4.0, false));
        let row = &m[(4 * W) as usize..(5 * W) as usize];
        assert!(row[11] > 0.25 && row[11] < 0.5, "{}", row[11]);
        assert!(row[12] > 0.5 && row[12] < 0.75, "{}", row[12]);
    }

    #[test]
    fn blending_by_a_mask_of_zero_and_one_gives_exactly_one_of_the_two_images() {
        let reference = columns(4, limb);
        let sharpened = columns(4, ringing);
        let ones = vec![1.0; (W * 4) as usize];
        let zeros = vec![0.0; (W * 4) as usize];
        assert_eq!(blend_by_mask(&sharpened, &reference, &ones), sharpened);
        assert_eq!(blend_by_mask(&sharpened, &reference, &zeros), reference);
        let halves = vec![0.5; (W * 4) as usize];
        let mixed = blend_by_mask(&sharpened, &reference, &halves);
        let i = 10;
        assert!((mixed.data[i] - (reference.data[i] + sharpened.data[i]) / 2.0).abs() < 1e-6);
    }

    #[test]
    fn the_overlay_leaves_what_the_mask_does_not_cover_and_tints_the_rest() {
        let image = columns(2, |_| 0.5);
        let overlay = mask_overlay(&image, &[0.0, 1.0, 0.0, 1.0].repeat(12));
        assert_eq!(overlay.channels, 3);
        assert_eq!(&overlay.data[0..3], &[0.5, 0.5, 0.5]);
        let tinted = &overlay.data[3..6];
        assert!(
            tinted[2] > tinted[1] && tinted[1] > tinted[0],
            "blue on top of gray: {tinted:?}"
        );
    }

    // --- All together ---------------------------------------------------------------------------

    #[test]
    fn default_settings_return_the_sharpened_image_bit_exact() {
        let reference = columns(5, limb);
        let sharpened = columns(5, ringing);
        assert_eq!(
            refine(&sharpened, Some(&reference), &RefineSettings::default()),
            sharpened
        );
    }

    #[test]
    fn the_mask_keeps_the_stack_where_it_does_not_apply() {
        let reference = columns(5, limb);
        let sharpened = columns(5, ringing);
        let settings = RefineSettings {
            mask: settings(100.0, 0.0, false),
            ..RefineSettings::default()
        };
        let out = refine(&sharpened, Some(&reference), &settings);
        for i in 0..out.data.len() {
            let planet = i as u32 % W >= 12;
            assert_eq!(
                out.data[i],
                if planet {
                    sharpened.data[i]
                } else {
                    reference.data[i]
                }
            );
        }
    }

    #[test]
    fn without_a_matching_reference_only_denoise_runs() {
        let sharpened = noisy_flat(0.5, 0.08);
        let settings = RefineSettings {
            denoise: 100.0,
            deringing: 100.0,
            mask: settings(100.0, 0.0, false),
            ..RefineSettings::default()
        };
        let alone = refine(&sharpened, None, &settings);
        assert_eq!(alone, denoise(&sharpened, 1.0));
        let other_size = Image::new(3, 3, 1);
        assert_eq!(refine(&sharpened, Some(&other_size), &settings), alone);
    }

    #[test]
    fn a_gray_stack_is_the_reference_of_the_colour_image_made_from_it() {
        // The window holds every image as RGB, also the one of a gray recording.
        let reference = columns(5, limb);
        let mut sharpened = Image::new(W, 5, 3);
        for (i, px) in sharpened.data.as_chunks_mut::<3>().0.iter_mut().enumerate() {
            px.fill(ringing(i as u32 % W));
        }
        let settings = RefineSettings {
            mask: settings(100.0, 0.0, false),
            ..RefineSettings::default()
        };
        let out = refine(&sharpened, Some(&reference), &settings);
        assert_eq!(out.channels, 3);
        // Column 10 is the dip of the sharpened image, in the sky: the mask puts the stack back.
        assert_eq!(
            &out.data[30..33],
            &[0.05, 0.05, 0.05],
            "the sky is the stack"
        );
        let planet = 13 * 3;
        assert_eq!(&out.data[planet..planet + 3], &[0.9, 0.9, 0.9]);
    }

    #[test]
    fn the_steps_run_in_the_order_deringing_denoise_mask() {
        let reference = columns(6, limb);
        let sharpened = columns(6, ringing);
        let settings = RefineSettings {
            denoise: 60.0,
            deringing: 100.0,
            deringing_radius: 3,
            mask: settings(100.0, 2.0, false),
        };
        let expected = blend_by_mask(
            &denoise(&deringing(&sharpened, &reference, 1.0, 3), 0.6),
            &reference,
            &mask(&reference, &settings.mask),
        );
        assert_eq!(refine(&sharpened, Some(&reference), &settings), expected);
    }
}
