//! Tone adjustments behind the *Functions* panel of the Wavelet tab: histogram, histogram stretch,
//! gamma and contrast/brightness.
//!
//! The formulas are our own design. Samples are normalised to `0.0..=1.0`; every adjustment that changes values clamps
//! the result to that range.

use crate::Image;
use rayon::prelude::*;

/// Number of bins of a [`Histogram`], one per 8-bit level.
pub const HISTOGRAM_BINS: usize = 256;

/// Finer resolution for [`auto_levels`], so that clipping a small share of pixels stays precise.
const LEVEL_BINS: usize = 4096;

/// Counts of samples per level, for each of the first three channels and for the luminance.
#[derive(Debug, Clone, PartialEq)]
pub struct Histogram {
    /// One entry per channel of the image (at most three), each with [`HISTOGRAM_BINS`] bins.
    pub channels: Vec<Vec<u32>>,
    /// The luminance of every pixel, see [`Image::to_luma`].
    pub luma: Vec<u32>,
}

fn bin(value: f32, bins: usize) -> usize {
    ((value.clamp(0.0, 1.0) * (bins - 1) as f32).round() as usize).min(bins - 1)
}

/// Counts the samples of an image per level.
pub fn histogram(image: &Image) -> Histogram {
    let stride = usize::from(image.channels).max(1);
    let shown = stride.min(3);
    let mut channels = vec![vec![0u32; HISTOGRAM_BINS]; shown];
    for px in image.data.chunks_exact(stride) {
        for (counts, &v) in channels.iter_mut().zip(px) {
            counts[bin(v, HISTOGRAM_BINS)] += 1;
        }
    }
    let mut luma = vec![0u32; HISTOGRAM_BINS];
    for v in image.to_luma().data {
        luma[bin(v, HISTOGRAM_BINS)] += 1;
    }
    Histogram { channels, luma }
}

/// Black and white point of a histogram stretch. Everything at or below `black` becomes 0,
/// everything at or above `white` becomes 1, the range in between is spread out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Levels {
    pub black: f32,
    pub white: f32,
}

impl Default for Levels {
    fn default() -> Self {
        Self {
            black: 0.0,
            white: 1.0,
        }
    }
}

/// Smallest distance between black and white point, so that the stretch never divides by zero.
const MIN_LEVEL_SPAN: f32 = 1.0 / 4096.0;

/// Spreads `levels.black..=levels.white` over the full range, the same for all channels so that
/// the colour balance stays as it is.
pub fn stretch_levels(image: &mut Image, levels: Levels) {
    if levels == Levels::default() {
        return;
    }
    let span = (levels.white - levels.black).max(MIN_LEVEL_SPAN);
    image
        .data
        .par_iter_mut()
        .for_each(|v| *v = ((*v - levels.black) / span).clamp(0.0, 1.0));
}

/// The levels that cut off the darkest and brightest `clip` share (`0.0..0.5`) of the pixels,
/// measured on the luminance. For an image without any range the default levels are returned.
pub fn auto_levels(image: &Image, clip: f32) -> Levels {
    let luma = image.to_luma();
    let total = luma.data.len() as u64;
    if total == 0 {
        return Levels::default();
    }
    let mut counts = vec![0u64; LEVEL_BINS];
    for &v in &luma.data {
        counts[bin(v, LEVEL_BINS)] += 1;
    }
    let clipped = (f64::from(clip.clamp(0.0, 0.49)) * total as f64) as u64;
    let mut cumulative = 0u64;
    let mut black = None;
    let mut white = LEVEL_BINS - 1;
    for (i, &count) in counts.iter().enumerate() {
        cumulative += count;
        if black.is_none() && cumulative > clipped {
            black = Some(i);
        }
        if cumulative >= total - clipped {
            white = i;
            break;
        }
    }
    let black = black.unwrap_or(0);
    if white <= black {
        return Levels::default();
    }
    Levels {
        black: black as f32 / (LEVEL_BINS - 1) as f32,
        white: white as f32 / (LEVEL_BINS - 1) as f32,
    }
}

/// Smallest gamma that is accepted, to keep the exponent finite.
const MIN_GAMMA: f32 = 0.01;

/// Applies `value^(1/gamma)`: values above 1.0 brighten the midtones, values below darken them.
/// Black and white stay where they are. The same for all channels.
pub fn apply_gamma(image: &mut Image, gamma: f32) {
    if gamma == 1.0 {
        return;
    }
    let exponent = 1.0 / gamma.max(MIN_GAMMA);
    image
        .data
        .par_iter_mut()
        .for_each(|v| *v = v.clamp(0.0, 1.0).powf(exponent));
}

/// Scales the distance to mid gray by `contrast` (1.0 = unchanged), then adds `brightness`
/// (a share of the full range, 0.0 = unchanged).
pub fn contrast_brightness(image: &mut Image, contrast: f32, brightness: f32) {
    if contrast == 1.0 && brightness == 0.0 {
        return;
    }
    let contrast = contrast.max(0.0);
    image
        .data
        .par_iter_mut()
        .for_each(|v| *v = ((*v - 0.5) * contrast + 0.5 + brightness).clamp(0.0, 1.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gray(data: &[f32]) -> Image {
        Image {
            width: data.len() as u32,
            height: 1,
            channels: 1,
            data: data.to_vec(),
        }
    }

    #[test]
    fn histogram_counts_each_channel_and_the_luminance() {
        let img = Image {
            width: 2,
            height: 1,
            channels: 3,
            data: vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        };
        let h = histogram(&img);
        assert_eq!(h.channels.len(), 3);
        assert_eq!(h.channels[0][255], 2);
        assert_eq!(h.channels[1][0], 2);
        assert_eq!(h.channels[2][0], 2);
        // Pure red has the luminance 0.299, which lands in bin 76.
        assert_eq!(h.luma[76], 2);
        assert_eq!(h.luma.iter().sum::<u32>(), 2);
    }

    #[test]
    fn histogram_of_a_mono_image_has_one_channel() {
        let h = histogram(&gray(&[0.0, 0.5, 1.0]));
        assert_eq!(h.channels.len(), 1);
        assert_eq!(h.channels[0][0], 1);
        assert_eq!(h.channels[0][128], 1);
        assert_eq!(h.channels[0][255], 1);
    }

    #[test]
    fn stretch_spreads_the_chosen_range() {
        let mut img = gray(&[0.1, 0.2, 0.5, 0.8, 0.9]);
        stretch_levels(
            &mut img,
            Levels {
                black: 0.2,
                white: 0.8,
            },
        );
        let expected = [0.0, 0.0, 0.5, 1.0, 1.0];
        for (got, want) in img.data.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6, "{got} != {want}");
        }
    }

    #[test]
    fn stretch_with_equal_points_does_not_produce_nan() {
        let mut img = gray(&[0.1, 0.5, 0.9]);
        stretch_levels(
            &mut img,
            Levels {
                black: 0.5,
                white: 0.5,
            },
        );
        assert!(img.data.iter().all(|v| (0.0..=1.0).contains(v)));
    }

    #[test]
    fn auto_levels_without_clipping_finds_the_data_range() {
        let ramp: Vec<f32> = (0..=100).map(|i| 0.2 + 0.6 * i as f32 / 100.0).collect();
        let levels = auto_levels(&gray(&ramp), 0.0);
        assert!((levels.black - 0.2).abs() < 0.001, "{levels:?}");
        assert!((levels.white - 0.8).abs() < 0.001, "{levels:?}");
    }

    #[test]
    fn auto_levels_clipping_ignores_outliers() {
        // Eight values between 0.4 and 0.6 plus one black and one white outlier: clipping 10 %
        // of ten pixels drops one pixel on each side, which are exactly the outliers.
        let mut data: Vec<f32> = (0..8).map(|i| 0.4 + 0.2 * i as f32 / 7.0).collect();
        data.extend([0.0, 1.0]);
        let levels = auto_levels(&gray(&data), 0.1);
        assert!((levels.black - 0.4).abs() < 0.001, "{levels:?}");
        assert!((levels.white - 0.6).abs() < 0.001, "{levels:?}");
    }

    #[test]
    fn auto_levels_of_a_flat_image_are_the_default() {
        assert_eq!(auto_levels(&gray(&[0.4; 10]), 0.0), Levels::default());
    }

    #[test]
    fn gamma_moves_the_midtones_and_keeps_the_ends() {
        let mut img = gray(&[0.0, 0.25, 1.0]);
        apply_gamma(&mut img, 2.0);
        assert_eq!(img.data[0], 0.0);
        assert!((img.data[1] - 0.5).abs() < 1e-6);
        assert_eq!(img.data[2], 1.0);

        let mut img = gray(&[0.5]);
        apply_gamma(&mut img, 0.5);
        assert!((img.data[0] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn neutral_contrast_and_brightness_change_nothing() {
        let mut img = gray(&[0.0, 0.3, 1.2]);
        let before = img.clone();
        contrast_brightness(&mut img, 1.0, 0.0);
        assert_eq!(img, before);
    }

    #[test]
    fn contrast_pivots_around_mid_gray_and_brightness_shifts() {
        let mut img = gray(&[0.25, 0.5, 0.75]);
        contrast_brightness(&mut img, 2.0, 0.0);
        let expected = [0.0, 0.5, 1.0];
        for (got, want) in img.data.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6);
        }

        let mut img = gray(&[0.25, 0.5, 0.9]);
        contrast_brightness(&mut img, 1.0, 0.25);
        let expected = [0.5, 0.75, 1.0];
        for (got, want) in img.data.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6);
        }
    }
}
