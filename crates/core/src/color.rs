//! Colour handling behind the *Functions* panel of the Wavelet tab: RGB align, RGB balance and the
//! recombination of a processed luminance with the colour (*LRGB*). [`adjust`] chains these with
//! the tone adjustments of [`crate::tone`].
//!
//! The algorithms are our own design. Images with fewer than three channels pass through the colour steps unchanged.

use crate::{
    Image,
    tone::{self, Levels},
};
use rayon::prelude::*;

/// Offset of one colour channel in pixels. Positive `x` moves the content to the right, positive
/// `y` down.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shift {
    pub x: f32,
    pub y: f32,
}

impl Shift {
    fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

/// How far each colour channel is moved, to cancel the colour fringes that atmospheric dispersion
/// puts at the limb of a planet.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RgbShifts {
    pub red: Shift,
    pub green: Shift,
    pub blue: Shift,
}

/// Linear gain per colour channel, 1.0 = unchanged.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbBalance {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Default for RgbBalance {
    fn default() -> Self {
        Self {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
        }
    }
}

/// *Color Mixing*: every output channel is a weighted sum of the three input channels. `matrix[out][in]`
/// holds the weights as factors (1.0 = 100 %), so the identity is the neutral state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelMix {
    pub matrix: [[f32; 3]; 3],
    /// Scales each pixel after mixing so that it keeps the luminance it had before.
    pub preserve_brightness: bool,
}

impl ChannelMix {
    pub const IDENTITY: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    /// Whether mixing would leave the image as it is. Preserving brightness changes nothing then.
    pub fn is_identity(&self) -> bool {
        self.matrix == Self::IDENTITY
    }
}

impl Default for ChannelMix {
    fn default() -> Self {
        Self {
            matrix: Self::IDENTITY,
            preserve_brightness: false,
        }
    }
}

/// Bilinear sample at a fractional position; positions outside the plane repeat the edge pixels.
fn sample(plane: &[f32], w: usize, h: usize, x: f32, y: f32) -> f32 {
    let x = x.clamp(0.0, (w - 1) as f32);
    let y = y.clamp(0.0, (h - 1) as f32);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let top = plane[y0 * w + x0] * (1.0 - fx) + plane[y0 * w + x1] * fx;
    let bottom = plane[y1 * w + x0] * (1.0 - fx) + plane[y1 * w + x1] * fx;
    top * (1.0 - fy) + bottom * fy
}

/// The samples of one channel as a plane.
fn plane(image: &Image, channel: usize) -> Vec<f32> {
    let stride = usize::from(image.channels);
    image
        .data
        .chunks_exact(stride)
        .map(|px| px[channel])
        .collect()
}

/// *RGB Align*, manual: moves the colour channels by the given shifts, with sub-pixel accuracy.
/// The image keeps its size; samples that come from outside repeat the edge.
pub fn shift_channels(image: &mut Image, shifts: &RgbShifts) {
    let stride = usize::from(image.channels);
    let (w, h) = (image.width as usize, image.height as usize);
    if stride < 3 || w == 0 || h == 0 {
        return;
    }
    for (channel, shift) in [shifts.red, shifts.green, shifts.blue]
        .into_iter()
        .enumerate()
    {
        if shift.is_zero() {
            continue;
        }
        let source = plane(image, channel);
        image
            .data
            .par_chunks_mut(w * stride)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..w {
                    row[x * stride + channel] =
                        sample(&source, w, h, x as f32 - shift.x, y as f32 - shift.y);
                }
            });
    }
}

/// Gradient magnitude of a plane, zero at the border. Edges are what the colour channels of a
/// planet have in common, whatever their brightness.
fn edges(plane: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0; w * h];
    if w < 3 || h < 3 {
        return out;
    }
    out.par_chunks_mut(w)
        .enumerate()
        .skip(1)
        .take(h - 2)
        .for_each(|(y, row)| {
            for x in 1..w - 1 {
                let gx = plane[y * w + x + 1] - plane[y * w + x - 1];
                let gy = plane[(y + 1) * w + x] - plane[(y - 1) * w + x];
                row[x] = (gx * gx + gy * gy).sqrt();
            }
        });
    out
}

/// Normalised correlation of `reference(x, y)` with `moving(x - dx, y - dy)` over the overlap,
/// or `None` where the overlap is too small or flat.
fn correlation(
    reference: &[f32],
    moving: &[f32],
    (w, h): (usize, usize),
    (dx, dy): (i32, i32),
) -> Option<f32> {
    let (w, h) = (w as i32, h as i32);
    let (x0, x1) = (0.max(dx), w.min(w + dx));
    let (y0, y1) = (0.max(dy), h.min(h + dy));
    if (x1 - x0) * (y1 - y0) < w * h / 4 {
        return None;
    }
    // Sums of many similar values cancel badly in f32 when the variance is taken from them.
    let (mut sa, mut sb, mut saa, mut sbb, mut sab) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for y in y0..y1 {
        let ra = &reference[(y * w + x0) as usize..(y * w + x1) as usize];
        let rb = &moving[((y - dy) * w + x0 - dx) as usize..((y - dy) * w + x1 - dx) as usize];
        for (&a, &b) in ra.iter().zip(rb) {
            let (a, b) = (f64::from(a), f64::from(b));
            sa += a;
            sb += b;
            saa += a * a;
            sbb += b * b;
            sab += a * b;
        }
    }
    let n = f64::from((x1 - x0) * (y1 - y0));
    let (va, vb) = (saa - sa * sa / n, sbb - sb * sb / n);
    (va > 1e-12 && vb > 1e-12).then(|| ((sab - sa * sb / n) / (va * vb).sqrt()) as f32)
}

/// Vertex of the parabola through three equally spaced scores, as an offset from the middle one
/// (`-0.5..=0.5`).
fn parabola_peak(before: f32, middle: f32, after: f32) -> f32 {
    let curvature = before - 2.0 * middle + after;
    if curvature >= 0.0 {
        return 0.0;
    }
    (0.5 * (before - after) / curvature).clamp(-0.5, 0.5)
}

/// The shift that moves `moving` onto `reference`, searched within `max_shift` pixels on the edge
/// maps and refined to sub-pixel accuracy. No shift if nothing correlates.
fn best_shift(reference: &[f32], moving: &[f32], size: (usize, usize), max_shift: i32) -> Shift {
    let side = (2 * max_shift + 1) as usize;
    let scores: Vec<Option<f32>> = (0..side * side)
        .into_par_iter()
        .map(|i| {
            let dx = (i % side) as i32 - max_shift;
            let dy = (i / side) as i32 - max_shift;
            correlation(reference, moving, size, (dx, dy))
        })
        .collect();
    let at = |dx: i32, dy: i32| -> Option<f32> {
        let (ix, iy) = (dx + max_shift, dy + max_shift);
        if ix < 0 || iy < 0 || ix >= side as i32 || iy >= side as i32 {
            return None;
        }
        scores[iy as usize * side + ix as usize]
    };
    let mut best: Option<(f32, i32, i32)> = None;
    for dy in -max_shift..=max_shift {
        for dx in -max_shift..=max_shift {
            if let Some(score) = at(dx, dy)
                && best.is_none_or(|(b, _, _)| score > b)
            {
                best = Some((score, dx, dy));
            }
        }
    }
    let Some((peak, dx, dy)) = best else {
        return Shift::default();
    };
    let refine = |before: Option<f32>, after: Option<f32>| match (before, after) {
        (Some(b), Some(a)) => parabola_peak(b, peak, a),
        _ => 0.0,
    };
    Shift {
        x: dx as f32 + refine(at(dx - 1, dy), at(dx + 1, dy)),
        y: dy as f32 + refine(at(dx, dy - 1), at(dx, dy + 1)),
    }
}

/// *RGB Align*, automatic: estimates how far red and blue have to move to lie on green, within
/// `max_shift` pixels, by correlating the edges of the channels. Green is the reference because it
/// is the sharpest channel of most colour cameras. The result is meant for [`shift_channels`].
pub fn estimate_rgb_shifts(image: &Image, max_shift: u32) -> RgbShifts {
    let stride = usize::from(image.channels);
    let size = (image.width as usize, image.height as usize);
    if stride < 3 || size.0 < 3 || size.1 < 3 || max_shift == 0 {
        return RgbShifts::default();
    }
    let max_shift = max_shift.min(i32::MAX as u32 / 2) as i32;
    let green = edges(&plane(image, 1), size.0, size.1);
    let towards_green = |channel: usize| {
        best_shift(
            &green,
            &edges(&plane(image, channel), size.0, size.1),
            size,
            max_shift,
        )
    };
    RgbShifts {
        red: towards_green(0),
        green: Shift::default(),
        blue: towards_green(2),
    }
}

/// *RGB Balance*: multiplies the channels by their gains and clamps the result to `0.0..=1.0`.
pub fn balance_rgb(image: &mut Image, balance: &RgbBalance) {
    let stride = usize::from(image.channels);
    if stride < 3 || *balance == RgbBalance::default() {
        return;
    }
    let gains = [balance.red, balance.green, balance.blue];
    image.data.par_chunks_mut(stride).for_each(|px| {
        for (v, gain) in px.iter_mut().zip(gains) {
            *v = (*v * gain).clamp(0.0, 1.0);
        }
    });
}

/// *Color Mixing*: replaces each pixel by the weighted sums of its channels. Negative sums become 0
/// (there is no negative light). With `preserve_brightness` the result is scaled so that its
/// luminance equals the one of the input; a pixel that mixes to black becomes gray at the input
/// luminance. Results are clamped to `0.0..=1.0`, which can bend the hue of pixels that would
/// exceed it. The identity and images with fewer than three channels pass through untouched.
pub fn mix_channels(image: &mut Image, mix: &ChannelMix) {
    let stride = usize::from(image.channels);
    if stride < 3 || mix.is_identity() {
        return;
    }
    let weights = crate::LUMA_WEIGHTS;
    let luma = |px: [f32; 3]| weights[0] * px[0] + weights[1] * px[1] + weights[2] * px[2];
    image.data.par_chunks_mut(stride).for_each(|px| {
        let input = [px[0], px[1], px[2]];
        let mut out = mix
            .matrix
            .map(|row| (row[0] * input[0] + row[1] * input[1] + row[2] * input[2]).max(0.0));
        if mix.preserve_brightness {
            let target = luma(input);
            let got = luma(out);
            if got > 1e-6 {
                let ratio = target / got;
                out = out.map(|v| v * ratio);
            } else {
                out = [target; 3];
            }
        }
        for (v, new) in px.iter_mut().zip(out) {
            *v = new.clamp(0.0, 1.0);
        }
    });
}

/// *LRGB*: gives `colour` the luminance of `luminance` and keeps its hue and saturation, by scaling
/// the three channels of each pixel with the ratio of the new to the old luminance. Pixels that are
/// black in `colour` become gray. `None` if `luminance` is not a single channel image of the same
/// size as `colour`, or `colour` has fewer than three channels.
pub fn with_luminance(colour: &Image, luminance: &Image) -> Option<Image> {
    let stride = usize::from(colour.channels);
    if stride < 3
        || luminance.channels != 1
        || luminance.width != colour.width
        || luminance.height != colour.height
    {
        return None;
    }
    let old = colour.to_luma();
    let mut out = colour.clone();
    out.data
        .par_chunks_mut(stride)
        .zip(old.data.par_iter().zip(luminance.data.par_iter()))
        .for_each(|(px, (&old, &new))| {
            if old > 1e-6 {
                let ratio = new / old;
                for v in &mut px[..3] {
                    *v = (*v * ratio).clamp(0.0, 1.0);
                }
            } else {
                px[..3].fill(new.clamp(0.0, 1.0));
            }
        });
    Some(out)
}

/// Everything the *Functions* panels set, with the unchanged state as default.
#[derive(Debug, Clone, PartialEq)]
pub struct Adjustments {
    pub shifts: RgbShifts,
    /// *Color Mixing*, applied after the shift and before the balance.
    pub mix: ChannelMix,
    pub balance: RgbBalance,
    /// Histogram stretch.
    pub levels: Levels,
    /// 1.0 = unchanged.
    pub gamma: f32,
    /// Factor, 1.0 = unchanged (shown as 100 in the panel).
    pub contrast: f32,
    /// Share of the full range, 0.0 = unchanged.
    pub brightness: f32,
}

impl Default for Adjustments {
    fn default() -> Self {
        Self {
            shifts: RgbShifts::default(),
            mix: ChannelMix::default(),
            balance: RgbBalance::default(),
            levels: Levels::default(),
            gamma: 1.0,
            contrast: 1.0,
            brightness: 0.0,
        }
    }
}

impl Adjustments {
    /// The steps up to, not including, the histogram stretch: what the histogram panel looks at
    /// when it places the black and white point.
    pub fn before_levels(&self) -> Self {
        Self {
            levels: Levels::default(),
            gamma: 1.0,
            contrast: 1.0,
            brightness: 0.0,
            ..self.clone()
        }
    }
}

/// Applies all adjustments in a fixed order: RGB align, colour mixing, RGB balance, histogram stretch, gamma,
/// contrast/brightness. The order is our choice. Steps at their default are skipped, so the default [`Adjustments`]
/// return the image unchanged.
pub fn adjust(image: &Image, adjustments: &Adjustments) -> Image {
    let mut out = image.clone();
    shift_channels(&mut out, &adjustments.shifts);
    mix_channels(&mut out, &adjustments.mix);
    balance_rgb(&mut out, &adjustments.balance);
    tone::stretch_levels(&mut out, adjustments.levels);
    tone::apply_gamma(&mut out, adjustments.gamma);
    tone::contrast_brightness(&mut out, adjustments.contrast, adjustments.brightness);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A smooth test scene: a few Gaussian blobs, evaluated at `(x - ox, y - oy)` so that the
    /// content sits `(ox, oy)` further right and down. Exact for any fractional offset.
    fn scene(x: f32, y: f32) -> f32 {
        const BLOBS: [(f32, f32, f32, f32); 4] = [
            (30.0, 28.0, 7.0, 0.9),
            (46.0, 40.0, 5.0, 0.6),
            (24.0, 44.0, 4.0, 0.5),
            (40.0, 22.0, 4.5, 0.4),
        ];
        BLOBS
            .iter()
            .map(|&(cx, cy, sigma, amp)| {
                amp * (-((x - cx).powi(2) + (y - cy).powi(2)) / (2.0 * sigma * sigma)).exp()
            })
            .sum::<f32>()
            .min(1.0)
    }

    /// 72x64 RGB image whose channels hold the scene at the given content offsets.
    fn rgb_scene(red: (f32, f32), green: (f32, f32), blue: (f32, f32)) -> Image {
        let (w, h) = (72u32, 64u32);
        let mut img = Image::new(w, h, 3);
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize * 3;
                for (c, (ox, oy)) in [red, green, blue].into_iter().enumerate() {
                    img.data[i + c] = scene(x as f32 - ox, y as f32 - oy);
                }
            }
        }
        img
    }

    fn max_difference(a: &Image, b: &Image, ca: usize, cb: usize) -> f32 {
        let stride = usize::from(a.channels);
        a.data
            .chunks_exact(stride)
            .zip(b.data.chunks_exact(stride))
            .map(|(pa, pb)| (pa[ca] - pb[cb]).abs())
            .fold(0.0, f32::max)
    }

    #[test]
    fn integer_shift_moves_one_channel_exactly() {
        let mut img = Image {
            width: 3,
            height: 2,
            channels: 3,
            data: (0..18).map(|i| i as f32 / 20.0).collect(),
        };
        let before = img.clone();
        shift_channels(
            &mut img,
            &RgbShifts {
                red: Shift { x: 1.0, y: 0.0 },
                ..RgbShifts::default()
            },
        );
        // Red moves one pixel right; the left column repeats the edge.
        for y in 0..2 {
            let row = |img: &Image, x: usize| img.data[(y * 3 + x) * 3];
            assert_eq!(row(&img, 0), row(&before, 0));
            assert_eq!(row(&img, 1), row(&before, 0));
            assert_eq!(row(&img, 2), row(&before, 1));
        }
        // Green and blue are untouched.
        for i in 0..6 {
            assert_eq!(img.data[i * 3 + 1], before.data[i * 3 + 1]);
            assert_eq!(img.data[i * 3 + 2], before.data[i * 3 + 2]);
        }
    }

    #[test]
    fn half_pixel_shift_interpolates() {
        let mut img = Image {
            width: 3,
            height: 1,
            channels: 3,
            data: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        };
        shift_channels(
            &mut img,
            &RgbShifts {
                red: Shift { x: 0.5, y: 0.0 },
                ..RgbShifts::default()
            },
        );
        let red: Vec<f32> = (0..3).map(|i| img.data[i * 3]).collect();
        assert_eq!(red, vec![0.0, 0.5, 0.5]);
    }

    #[test]
    fn mono_images_pass_through_the_colour_steps() {
        let mut img = Image {
            width: 2,
            height: 1,
            channels: 1,
            data: vec![0.2, 0.8],
        };
        let before = img.clone();
        shift_channels(
            &mut img,
            &RgbShifts {
                red: Shift { x: 1.0, y: 0.0 },
                ..RgbShifts::default()
            },
        );
        balance_rgb(
            &mut img,
            &RgbBalance {
                red: 2.0,
                ..RgbBalance::default()
            },
        );
        assert_eq!(img, before);
        assert_eq!(estimate_rgb_shifts(&img, 5), RgbShifts::default());
    }

    #[test]
    fn auto_align_finds_integer_offsets() {
        // Red content sits 2 right and 1 up of green, blue 3 left: the shifts that undo this.
        let img = rgb_scene((2.0, -1.0), (0.0, 0.0), (-3.0, 0.0));
        let shifts = estimate_rgb_shifts(&img, 6);
        assert_eq!(shifts.green, Shift::default());
        assert!((shifts.red.x + 2.0).abs() < 0.2, "{shifts:?}");
        assert!((shifts.red.y - 1.0).abs() < 0.2, "{shifts:?}");
        assert!((shifts.blue.x - 3.0).abs() < 0.2, "{shifts:?}");
        assert!(shifts.blue.y.abs() < 0.2, "{shifts:?}");
    }

    #[test]
    fn auto_align_finds_sub_pixel_offsets_and_aligning_removes_them() {
        let img = rgb_scene((1.5, 0.5), (0.0, 0.0), (0.0, -2.5));
        let shifts = estimate_rgb_shifts(&img, 5);
        assert!((shifts.red.x + 1.5).abs() < 0.3, "{shifts:?}");
        assert!((shifts.red.y + 0.5).abs() < 0.3, "{shifts:?}");
        assert!((shifts.blue.y - 2.5).abs() < 0.3, "{shifts:?}");

        let mut aligned = img.clone();
        shift_channels(&mut aligned, &shifts);
        // Inside the image the channels now agree; the border is excluded because the shifted
        // channel repeats its edge there.
        let before = max_difference(&img, &img, 0, 1);
        let mut worst = 0.0f32;
        for y in 8..56usize {
            for x in 8..64usize {
                let i = (y * 72 + x) * 3;
                worst = worst.max((aligned.data[i] - aligned.data[i + 1]).abs());
                worst = worst.max((aligned.data[i + 2] - aligned.data[i + 1]).abs());
            }
        }
        assert!(before > 0.05, "the test scene must start misaligned");
        assert!(
            worst < before / 3.0,
            "channels differ by {worst} after aligning, {before} before"
        );
    }

    #[test]
    fn auto_align_of_an_aligned_image_stays_put() {
        let img = rgb_scene((0.0, 0.0), (0.0, 0.0), (0.0, 0.0));
        let shifts = estimate_rgb_shifts(&img, 5);
        assert!(
            shifts.red.x.abs() < 0.1 && shifts.red.y.abs() < 0.1,
            "{shifts:?}"
        );
        assert!(
            shifts.blue.x.abs() < 0.1 && shifts.blue.y.abs() < 0.1,
            "{shifts:?}"
        );
    }

    #[test]
    fn balance_scales_channels_and_clamps() {
        let mut img = Image {
            width: 1,
            height: 1,
            channels: 3,
            data: vec![0.5, 0.5, 0.5],
        };
        balance_rgb(
            &mut img,
            &RgbBalance {
                red: 0.5,
                green: 1.0,
                blue: 3.0,
            },
        );
        assert_eq!(img.data, vec![0.25, 0.5, 1.0]);
    }

    #[test]
    fn lrgb_keeps_the_hue_and_takes_the_new_luminance() {
        let colour = Image {
            width: 2,
            height: 1,
            channels: 3,
            data: vec![0.4, 0.2, 0.1, 0.0, 0.0, 0.0],
        };
        let old = colour.to_luma();
        let target = Image {
            width: 2,
            height: 1,
            channels: 1,
            data: vec![old.data[0] * 0.5, 0.3],
        };
        let out = with_luminance(&colour, &target).unwrap();
        // First pixel: every channel halves, so the ratio between them is unchanged.
        for (got, want) in out.data[..3].iter().zip([0.2, 0.1, 0.05]) {
            assert!((got - want).abs() < 1e-6);
        }
        // Black pixel becomes gray at the new luminance.
        assert_eq!(&out.data[3..], &[0.3, 0.3, 0.3]);
    }

    #[test]
    fn lrgb_rejects_mismatching_inputs() {
        let colour = Image::new(2, 2, 3);
        assert!(with_luminance(&colour, &Image::new(3, 2, 1)).is_none());
        assert!(with_luminance(&colour, &Image::new(2, 2, 3)).is_none());
        assert!(with_luminance(&Image::new(2, 2, 1), &Image::new(2, 2, 1)).is_none());
    }

    #[test]
    fn default_adjustments_leave_the_image_untouched() {
        let img = rgb_scene((0.0, 0.0), (0.0, 0.0), (0.0, 0.0));
        assert_eq!(adjust(&img, &Adjustments::default()), img);
    }

    #[test]
    fn adjustments_apply_in_the_documented_order() {
        let img = Image {
            width: 1,
            height: 1,
            channels: 3,
            data: vec![0.2, 0.2, 0.2],
        };
        let out = adjust(
            &img,
            &Adjustments {
                balance: RgbBalance {
                    red: 2.0,
                    green: 1.0,
                    blue: 1.0,
                },
                levels: Levels {
                    black: 0.2,
                    white: 0.4,
                },
                brightness: 0.1,
                ..Adjustments::default()
            },
        );
        // Balance: red 0.4. Stretch 0.2..0.4: red 1.0, green and blue 0.0. Brightness +0.1.
        for (got, want) in out.data.iter().zip([1.0, 0.1, 0.1]) {
            assert!((got - want).abs() < 1e-6, "{:?}", out.data);
        }
    }

    #[test]
    fn before_levels_drops_only_the_tone_steps() {
        let all = Adjustments {
            shifts: RgbShifts {
                red: Shift { x: 1.0, y: 0.0 },
                ..RgbShifts::default()
            },
            levels: Levels {
                black: 0.1,
                white: 0.9,
            },
            gamma: 2.0,
            contrast: 1.5,
            brightness: 0.2,
            ..Adjustments::default()
        };
        let before = all.before_levels();
        assert_eq!(before.shifts, all.shifts);
        assert_eq!(
            Adjustments {
                shifts: RgbShifts::default(),
                ..before
            },
            Adjustments::default()
        );
    }

    fn mix_of(matrix: [[f32; 3]; 3], preserve_brightness: bool) -> ChannelMix {
        ChannelMix {
            matrix,
            preserve_brightness,
        }
    }

    #[test]
    fn identity_mix_is_bit_exact() {
        let img = rgb_scene((0.0, 0.0), (0.0, 0.0), (0.0, 0.0));
        let mut out = img.clone();
        mix_channels(&mut out, &ChannelMix::default());
        assert_eq!(out, img);
        // Preserving brightness changes nothing when there is nothing to preserve.
        mix_channels(&mut out, &mix_of(ChannelMix::IDENTITY, true));
        assert_eq!(out, img);
        // The default adjustments, which contain the identity mix, return the image as it was.
        assert_eq!(adjust(&img, &Adjustments::default()), img);
    }

    #[test]
    fn mixing_matrix_gives_the_weighted_sums() {
        let mut img = Image {
            width: 1,
            height: 1,
            channels: 3,
            data: vec![0.2, 0.4, 0.6],
        };
        // Output red = 0.5 R + 0.5 B, green = G, blue = 1.0 R - 0.5 G + 0.5 B.
        mix_channels(
            &mut img,
            &mix_of([[0.5, 0.0, 0.5], [0.0, 1.0, 0.0], [1.0, -0.5, 0.5]], false),
        );
        for (got, want) in img.data.iter().zip([0.4, 0.4, 0.3]) {
            assert!((got - want).abs() < 1e-6, "{:?}", img.data);
        }
    }

    #[test]
    fn mixing_swaps_channels_and_clamps() {
        let mut img = Image {
            width: 1,
            height: 1,
            channels: 3,
            data: vec![0.9, 0.1, 0.5],
        };
        // Red and blue swap places; green becomes 3 R - 2 B = 1.7 - 1.0 -> clamped, and a negative sum becomes 0.
        mix_channels(
            &mut img,
            &mix_of([[0.0, 0.0, 1.0], [3.0, 0.0, -2.0], [-1.0, 0.0, 0.0]], false),
        );
        assert_eq!(img.data, vec![0.5, 1.0, 0.0]);
    }

    #[test]
    fn preserve_brightness_keeps_the_luminance() {
        let source = Image {
            width: 3,
            height: 1,
            channels: 3,
            data: vec![0.30, 0.20, 0.10, 0.05, 0.15, 0.25, 0.0, 0.0, 0.0],
        };
        let matrix = [[1.0, 0.5, 0.0], [0.0, 0.2, 0.0], [0.5, 0.0, 1.5]];
        let mut plain = source.clone();
        mix_channels(&mut plain, &mix_of(matrix, false));
        let mut kept = source.clone();
        mix_channels(&mut kept, &mix_of(matrix, true));
        let before = source.to_luma();
        let (after_plain, after_kept) = (plain.to_luma(), kept.to_luma());
        for i in 0..2 {
            assert!(
                (after_kept.data[i] - before.data[i]).abs() < 1e-6,
                "pixel {i}: {} vs {}",
                after_kept.data[i],
                before.data[i]
            );
        }
        // Without it the luminance moves, so the option does something.
        assert!((after_plain.data[0] - before.data[0]).abs() > 0.01);
        // Black stays black.
        assert_eq!(&kept.data[6..], &[0.0, 0.0, 0.0]);
    }

    #[test]
    fn preserve_brightness_turns_a_pixel_that_mixes_to_black_gray() {
        let mut img = Image {
            width: 1,
            height: 1,
            channels: 3,
            data: vec![0.4, 0.2, 0.1],
        };
        let zero = [[0.0; 3]; 3];
        mix_channels(&mut img, &mix_of(zero, true));
        let want = 0.299 * 0.4 + 0.587 * 0.2 + 0.114 * 0.1;
        for v in &img.data {
            assert!((v - want).abs() < 1e-6, "{:?}", img.data);
        }
    }

    #[test]
    fn mixing_leaves_mono_images_alone() {
        let mut img = Image {
            width: 2,
            height: 1,
            channels: 1,
            data: vec![0.2, 0.8],
        };
        let before = img.clone();
        mix_channels(&mut img, &mix_of([[0.0; 3]; 3], false));
        assert_eq!(img, before);
    }

    #[test]
    fn mixing_runs_between_align_and_balance() {
        let img = Image {
            width: 1,
            height: 1,
            channels: 3,
            data: vec![0.2, 0.4, 0.1],
        };
        let out = adjust(
            &img,
            &Adjustments {
                mix: mix_of([[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], false),
                balance: RgbBalance {
                    red: 0.5,
                    ..RgbBalance::default()
                },
                ..Adjustments::default()
            },
        );
        // Red and green swap first (0.4, 0.2, 0.1), then the balance halves red.
        for (got, want) in out.data.iter().zip([0.2, 0.2, 0.1]) {
            assert!((got - want).abs() < 1e-6, "{:?}", out.data);
        }
    }
}
