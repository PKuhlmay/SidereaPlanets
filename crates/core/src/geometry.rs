//! Geometry: flipping, rotating and resizing the finished image (MAC-017).
//!
//! These are the last steps of *Save image*, after the colour and tone values and the crop: refine, colours, [`Orientation`], crop
//! ([`crate::crop`]), [`Resize`]. The crop rectangle is therefore set on the oriented picture, which is the one the window shows.
//! Everything that does not change a pixel (no flip, an angle of 0, a resize to the size the image has) returns the image unchanged, bit
//! for bit; turns by a multiple of 90 degrees and flips only move pixels and never resample.

use crate::{Image, crop::CropRect};
use rayon::prelude::*;

/// How pixels are looked up between the pixels of the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Resampling {
    Nearest,
    Bilinear,
    Bicubic,
    #[default]
    Lanczos3,
}

/// *Flip and Rotate*: mirror first (in the frame of the image), then turn clockwise by `angle` degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orientation {
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    /// Clockwise, in degrees. Multiples of 90 turn exactly; anything between is resampled with `method`.
    pub angle: f32,
    /// A free angle leaves black corners; with this the result is cut to the largest rectangle of the original shape inside the turned
    /// image, so there are none. Has no effect on turns by a multiple of 90.
    pub crop_to_fit: bool,
    pub method: Resampling,
}

impl Default for Orientation {
    fn default() -> Self {
        Self {
            flip_horizontal: false,
            flip_vertical: false,
            angle: 0.0,
            crop_to_fit: false,
            method: Resampling::Bilinear,
        }
    }
}

/// The size *Resize Image* asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeTarget {
    /// Both sides scaled by the same percentage ([`PERCENT_RANGE`]).
    Percent(f32),
    /// A size in pixels. A side that is `None` follows the other so that the shape is kept; both `None` is no change.
    Pixels {
        width: Option<u32>,
        height: Option<u32>,
    },
}

/// *Resize Image*.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resize {
    pub target: ResizeTarget,
    pub method: Resampling,
}

/// The percentages the card offers.
pub const PERCENT_RANGE: (f32, f32) = (10.0, 400.0);
/// The largest side of a resized image in pixels.
pub const MAX_SIDE: u32 = 16_384;

/// Orientation and resize together, as *Save image* takes them.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Geometry {
    pub orientation: Orientation,
    pub resize: Option<Resize>,
}

/// Angles closer than this (degrees) to a multiple of 90 count as one: a turn that exact is not resampled.
const EXACT_DEGREES: f32 = 1e-3;
/// How far outside the source a sample may lie (in pixels) and still count as inside; absorbs rounding at the edge.
const EDGE_SLACK: f32 = 1e-3;

/// The angle as quarter turns clockwise (0..4) and the free rest in degrees (within 45 either way; 0 if there is none).
fn split_angle(angle: f32) -> (u32, f32) {
    let angle = if angle.is_finite() { angle } else { 0.0 };
    let quarters = (angle / 90.0).round();
    let rest = angle - quarters * 90.0;
    let turns = (quarters as i64).rem_euclid(4) as u32;
    (
        turns,
        if rest.abs() < EXACT_DEGREES {
            0.0
        } else {
            rest
        },
    )
}

/// The canvas of an image of `width` x `height` turned by `degrees` (not a multiple of 90). It keeps the parity of the source, so the
/// centre of the source lies on the centre of a pixel of the result.
fn free_size(width: u32, height: u32, degrees: f32, crop_to_fit: bool) -> (u32, u32) {
    let (sin, cos) = degrees.to_radians().sin_cos();
    let (s, c) = (f64::from(sin.abs()), f64::from(cos.abs()));
    let (w, h) = (f64::from(width), f64::from(height));
    let (out_w, out_h) = if crop_to_fit {
        // The largest copy of the original shape (a * w x a * h) whose corners stay inside the turned image.
        let a = (w / (w * c + h * s)).min(h / (w * s + h * c)).min(1.0);
        (
            (w * a - 0.01).floor().max(1.0),
            (h * a - 0.01).floor().max(1.0),
        )
    } else {
        ((w * c + h * s - 1e-3).ceil(), (w * s + h * c - 1e-3).ceil())
    };
    let match_parity = |out: f64, source: u32| {
        let out = out as u32;
        if out % 2 == source % 2 {
            out
        } else if crop_to_fit && out > 1 {
            out - 1
        } else {
            out + 1
        }
    };
    (match_parity(out_w, width), match_parity(out_h, height))
}

impl Orientation {
    /// Whether the image stays as it is.
    pub fn is_neutral(&self) -> bool {
        let (turns, rest) = split_angle(self.angle);
        !self.flip_horizontal && !self.flip_vertical && turns == 0 && rest == 0.0
    }

    /// The size of the oriented image for a source of `width` x `height`.
    pub fn output_size(&self, width: u32, height: u32) -> (u32, u32) {
        let (turns, rest) = split_angle(self.angle);
        let (w, h) = if turns % 2 == 1 {
            (height, width)
        } else {
            (width, height)
        };
        if rest == 0.0 {
            (w, h)
        } else {
            free_size(w, h, rest, self.crop_to_fit)
        }
    }
}

impl Resize {
    /// The size of the resized image for a source of `width` x `height`.
    pub fn output_size(&self, width: u32, height: u32) -> (u32, u32) {
        let side = |value: f64| (value.round() as u32).clamp(1, MAX_SIDE);
        let (w, h) = (f64::from(width), f64::from(height));
        match self.target {
            ResizeTarget::Percent(p) => {
                let share = f64::from(p.clamp(PERCENT_RANGE.0, PERCENT_RANGE.1)) / 100.0;
                (side(w * share), side(h * share))
            }
            ResizeTarget::Pixels {
                width: Some(to_w),
                height: Some(to_h),
            } => (to_w.clamp(1, MAX_SIDE), to_h.clamp(1, MAX_SIDE)),
            ResizeTarget::Pixels {
                width: Some(to_w),
                height: None,
            } => {
                let to_w = to_w.clamp(1, MAX_SIDE);
                (to_w, side(h * f64::from(to_w) / w.max(1.0)))
            }
            ResizeTarget::Pixels {
                width: None,
                height: Some(to_h),
            } => {
                let to_h = to_h.clamp(1, MAX_SIDE);
                (side(w * f64::from(to_h) / h.max(1.0)), to_h)
            }
            ResizeTarget::Pixels {
                width: None,
                height: None,
            } => (width, height),
        }
    }

    /// Whether the image stays as it is.
    pub fn is_neutral(&self, width: u32, height: u32) -> bool {
        self.output_size(width, height) == (width, height)
    }
}

impl Image {
    /// Mirrors the image left to right and/or top to bottom.
    pub fn flipped(&self, horizontal: bool, vertical: bool) -> Image {
        if !horizontal && !vertical {
            return self.clone();
        }
        let (width, height) = (self.width as usize, self.height as usize);
        let channels = usize::from(self.channels).max(1);
        let mut data = Vec::with_capacity(self.data.len());
        for y in 0..height {
            let from = if vertical { height - 1 - y } else { y };
            let row = &self.data[from * width * channels..(from + 1) * width * channels];
            if horizontal {
                for px in row.chunks_exact(channels).rev() {
                    data.extend_from_slice(px);
                }
            } else {
                data.extend_from_slice(row);
            }
        }
        Image {
            data,
            ..self.clone_shape()
        }
    }

    /// Turns the image clockwise by `quarters` times 90 degrees without resampling.
    pub fn turned(&self, quarters: u32) -> Image {
        let channels = usize::from(self.channels).max(1);
        let (w, h) = (self.width as usize, self.height as usize);
        let pixel =
            |x: usize, y: usize| &self.data[(y * w + x) * channels..(y * w + x + 1) * channels];
        match quarters % 4 {
            0 => self.clone(),
            2 => self.flipped(true, true),
            turns => {
                let mut data = Vec::with_capacity(self.data.len());
                // The result is `h` wide and `w` high. Clockwise: its row `y` is column `y` of the source read from the bottom up;
                // counter-clockwise: column `w - 1 - y` read from the top down.
                for y in 0..w {
                    for x in 0..h {
                        let px = if turns == 1 {
                            pixel(y, h - 1 - x)
                        } else {
                            pixel(w - 1 - y, x)
                        };
                        data.extend_from_slice(px);
                    }
                }
                Image {
                    width: self.height,
                    height: self.width,
                    channels: self.channels,
                    data,
                }
            }
        }
    }

    /// The image flipped and rotated as `orientation` says.
    pub fn oriented(&self, orientation: &Orientation) -> Image {
        let (turns, rest) = split_angle(orientation.angle);
        let mut image = self.flipped(orientation.flip_horizontal, orientation.flip_vertical);
        if turns != 0 {
            image = image.turned(turns);
        }
        if rest == 0.0 {
            return image;
        }
        let (width, height) = free_size(image.width, image.height, rest, orientation.crop_to_fit);
        image.rotated_free(rest, orientation.method, width, height)
    }

    /// The image scaled as `resize` says.
    pub fn resized(&self, resize: &Resize) -> Image {
        let (width, height) = resize.output_size(self.width, self.height);
        if (width, height) == (self.width, self.height) {
            return self.clone();
        }
        let across = Taps::new(self.width as usize, width as usize, resize.method);
        let down = Taps::new(self.height as usize, height as usize, resize.method);
        let channels = usize::from(self.channels).max(1);
        // Horizontal pass, then vertical; an axis that keeps its size is skipped (and stays exact).
        let wide = if width == self.width {
            self.data.clone()
        } else {
            let in_row = self.width as usize * channels;
            let out_row = width as usize * channels;
            let mut out = vec![0.0; out_row * self.height as usize];
            out.par_chunks_mut(out_row)
                .enumerate()
                .for_each(|(y, row)| {
                    let source = &self.data[y * in_row..(y + 1) * in_row];
                    for x in 0..width as usize {
                        for ch in 0..channels {
                            let mut sum = 0.0;
                            for (i, weight) in across.taps(x) {
                                sum += weight * source[i * channels + ch];
                            }
                            row[x * channels + ch] = sum;
                        }
                    }
                });
            out
        };
        let data = if height == self.height {
            wide
        } else {
            let row_len = width as usize * channels;
            let mut out = vec![0.0; row_len * height as usize];
            out.par_chunks_mut(row_len)
                .enumerate()
                .for_each(|(y, row)| {
                    for (i, weight) in down.taps(y) {
                        let source = &wide[i * row_len..(i + 1) * row_len];
                        for (target, value) in row.iter_mut().zip(source) {
                            *target += weight * value;
                        }
                    }
                });
            out
        };
        let mut out = Image {
            width,
            height,
            channels: self.channels,
            data,
        };
        // Bicubic and Lanczos overshoot at hard edges.
        out.data.iter_mut().for_each(|v| *v = v.clamp(0.0, 1.0));
        out
    }

    /// A copy of the size and channels without the samples.
    fn clone_shape(&self) -> Image {
        Image {
            width: self.width,
            height: self.height,
            channels: self.channels,
            data: Vec::new(),
        }
    }

    /// Turns the image clockwise by `degrees` into a canvas of `width` x `height` pixels around the same centre; what lies outside the
    /// source is black.
    fn rotated_free(&self, degrees: f32, method: Resampling, width: u32, height: u32) -> Image {
        let channels = usize::from(self.channels).max(1);
        let (sin, cos) = degrees.to_radians().sin_cos();
        let (half_in_w, half_in_h) = (self.width as f32 / 2.0, self.height as f32 / 2.0);
        let (half_out_w, half_out_h) = (width as f32 / 2.0, height as f32 / 2.0);
        let row_len = width as usize * channels;
        let mut data = vec![0.0; row_len * height as usize];
        data.par_chunks_mut(row_len)
            .enumerate()
            .for_each(|(oy, row)| {
                let py = oy as f32 + 0.5 - half_out_h;
                for ox in 0..width as usize {
                    let px = ox as f32 + 0.5 - half_out_w;
                    // The inverse of the clockwise turn, from the canvas back to the source (y points down).
                    let sx = cos * px + sin * py + half_in_w;
                    let sy = -sin * px + cos * py + half_in_h;
                    let inside = sx >= -EDGE_SLACK
                        && sy >= -EDGE_SLACK
                        && sx <= self.width as f32 + EDGE_SLACK
                        && sy <= self.height as f32 + EDGE_SLACK;
                    if inside {
                        self.sample(
                            sx - 0.5,
                            sy - 0.5,
                            method,
                            &mut row[ox * channels..(ox + 1) * channels],
                        );
                    }
                }
            });
        Image {
            width,
            height,
            channels: self.channels,
            data,
        }
    }

    /// The samples at the position (`fx`, `fy`) in pixel units (the centre of the first pixel is 0, 0); positions beyond the edge read
    /// the edge pixel.
    fn sample(&self, fx: f32, fy: f32, method: Resampling, out: &mut [f32]) {
        let channels = out.len();
        let (w, h) = (self.width as usize, self.height as usize);
        let (xs, ys) = (Kernel::at(method, fx), Kernel::at(method, fy));
        out.fill(0.0);
        for (iy, wy) in ys.iter() {
            let y = iy.clamp(0, h as i64 - 1) as usize;
            for (ix, wx) in xs.iter() {
                let x = ix.clamp(0, w as i64 - 1) as usize;
                let weight = wx * wy;
                let px = &self.data[(y * w + x) * channels..(y * w + x + 1) * channels];
                for (o, v) in out.iter_mut().zip(px) {
                    *o += weight * v;
                }
            }
        }
        out.iter_mut().for_each(|v| *v = v.clamp(0.0, 1.0));
    }
}

/// How far a kernel reaches to each side of its centre, in source pixels.
fn radius(method: Resampling) -> f32 {
    match method {
        Resampling::Nearest => 0.5,
        Resampling::Bilinear => 1.0,
        Resampling::Bicubic => 2.0,
        Resampling::Lanczos3 => 3.0,
    }
}

/// The weight of a source pixel at distance `t` (in source pixels, scaled by the filter width) from the centre.
fn kernel(method: Resampling, t: f32) -> f32 {
    let t = t.abs();
    match method {
        Resampling::Nearest => {
            if t < 0.5 {
                1.0
            } else {
                0.0
            }
        }
        Resampling::Bilinear => (1.0 - t).max(0.0),
        // Catmull-Rom (a = -0.5): passes through the samples and does not blur more than it has to.
        Resampling::Bicubic => {
            if t < 1.0 {
                1.5 * t * t * t - 2.5 * t * t + 1.0
            } else if t < 2.0 {
                -0.5 * t * t * t + 2.5 * t * t - 4.0 * t + 2.0
            } else {
                0.0
            }
        }
        Resampling::Lanczos3 => {
            if t < 1e-6 {
                1.0
            } else if t < 3.0 {
                let x = std::f32::consts::PI * t;
                3.0 * x.sin() * (x / 3.0).sin() / (x * x)
            } else {
                0.0
            }
        }
    }
}

/// The source pixels (and their weights) around one position, for one axis of a rotation.
struct Kernel {
    first: i64,
    count: usize,
    weights: [f32; 6],
}

impl Kernel {
    fn at(method: Resampling, f: f32) -> Kernel {
        if method == Resampling::Nearest {
            return Kernel {
                first: f.round() as i64,
                count: 1,
                weights: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            };
        }
        let reach = radius(method) as i64;
        let first = f.floor() as i64 - reach + 1;
        let count = 2 * reach as usize;
        let mut weights = [0.0; 6];
        let mut sum = 0.0;
        for (k, weight) in weights.iter_mut().enumerate().take(count) {
            *weight = kernel(method, f - (first + k as i64) as f32);
            sum += *weight;
        }
        if sum.abs() > 1e-6 {
            weights[..count].iter_mut().for_each(|w| *w /= sum);
        }
        Kernel {
            first,
            count,
            weights,
        }
    }

    fn iter(&self) -> impl Iterator<Item = (i64, f32)> + '_ {
        (0..self.count).map(|k| (self.first + k as i64, self.weights[k]))
    }
}

/// The weights of every output pixel along one axis of a resize: which source pixels it reads and how much of each.
struct Taps {
    first: Vec<usize>,
    start: Vec<usize>,
    weights: Vec<f32>,
}

impl Taps {
    /// Shrinking stretches the kernel by the factor of the reduction, so that every source pixel counts (no aliasing).
    fn new(input: usize, output: usize, method: Resampling) -> Taps {
        let scale = input as f32 / output as f32;
        let stretch = scale.max(1.0);
        let reach = radius(method) * stretch;
        let mut taps = Taps {
            first: Vec::with_capacity(output),
            start: Vec::with_capacity(output + 1),
            weights: Vec::new(),
        };
        for o in 0..output {
            let centre = (o as f32 + 0.5) * scale;
            taps.start.push(taps.weights.len());
            if method == Resampling::Nearest {
                taps.first.push((centre.floor() as usize).min(input - 1));
                taps.weights.push(1.0);
                continue;
            }
            let lo = ((centre - reach).floor().max(0.0)) as usize;
            let hi = (((centre + reach).ceil()) as usize).min(input);
            let begin = taps.weights.len();
            for i in lo..hi {
                taps.weights
                    .push(kernel(method, (i as f32 + 0.5 - centre) / stretch));
            }
            let sum: f32 = taps.weights[begin..].iter().sum();
            if sum.abs() > 1e-6 {
                taps.weights[begin..].iter_mut().for_each(|w| *w /= sum);
            }
            taps.first.push(lo);
        }
        taps.start.push(taps.weights.len());
        taps
    }

    fn taps(&self, o: usize) -> impl Iterator<Item = (usize, f32)> + '_ {
        let (from, to) = (self.start[o], self.start[o + 1]);
        self.weights[from..to]
            .iter()
            .enumerate()
            .map(move |(k, &w)| (self.first[o] + k, w))
    }
}

impl Geometry {
    /// Orientation, then `crop` (set on the oriented image), then resize. `None` if the crop has no pixel in the oriented image.
    pub fn finish(&self, image: &Image, crop: Option<CropRect>) -> Option<Image> {
        let oriented = image.oriented(&self.orientation);
        let cropped = match crop {
            Some(rect) => oriented.cropped(rect)?,
            None => oriented,
        };
        Some(match &self.resize {
            Some(resize) => cropped.resized(resize),
            None => cropped,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pixel `i` holds `i`, so every move of a pixel shows.
    fn numbered(width: u32, height: u32, channels: u8) -> Image {
        let mut image = Image::new(width, height, channels);
        for (i, v) in image.data.iter_mut().enumerate() {
            *v = i as f32;
        }
        image
    }

    fn constant(width: u32, height: u32, channels: u8, value: f32) -> Image {
        let mut image = Image::new(width, height, channels);
        image.data.fill(value);
        image
    }

    fn pixel(image: &Image, x: u32, y: u32) -> Vec<f32> {
        let c = usize::from(image.channels);
        let at = (y as usize * image.width as usize + x as usize) * c;
        image.data[at..at + c].to_vec()
    }

    fn turn(angle: f32) -> Orientation {
        Orientation {
            angle,
            ..Orientation::default()
        }
    }

    // --- Flip ---

    #[test]
    fn flipping_horizontally_mirrors_each_row() {
        let out = numbered(3, 2, 1).flipped(true, false);
        assert_eq!(out.data, vec![2.0, 1.0, 0.0, 5.0, 4.0, 3.0]);
    }

    #[test]
    fn flipping_vertically_swaps_the_rows() {
        let out = numbered(3, 2, 1).flipped(false, true);
        assert_eq!(out.data, vec![3.0, 4.0, 5.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn flipping_keeps_the_channels_of_a_pixel_together() {
        let out = numbered(2, 1, 3).flipped(true, false);
        assert_eq!(out.data, vec![3.0, 4.0, 5.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn flipping_twice_is_the_identity_bit_for_bit() {
        for channels in [1u8, 3] {
            let image = numbered(5, 4, channels);
            for (h, v) in [(true, false), (false, true), (true, true)] {
                assert_eq!(image.flipped(h, v).flipped(h, v), image, "h {h} v {v}");
            }
        }
    }

    #[test]
    fn no_flip_is_the_identity() {
        let image = numbered(5, 4, 3);
        assert_eq!(image.flipped(false, false), image);
    }

    // --- Quarter turns ---

    #[test]
    fn a_quarter_turn_clockwise_moves_known_pixels() {
        // 3 wide, 2 high:  0 1 2 / 3 4 5   turned clockwise is 2 wide, 3 high:  3 0 / 4 1 / 5 2
        let out = numbered(3, 2, 1).turned(1);
        assert_eq!((out.width, out.height), (2, 3));
        assert_eq!(out.data, vec![3.0, 0.0, 4.0, 1.0, 5.0, 2.0]);
    }

    #[test]
    fn three_quarter_turns_are_a_quarter_turn_back() {
        // 3 wide, 2 high turned counter-clockwise: 2 5 / 1 4 / 0 3
        let out = numbered(3, 2, 1).turned(3);
        assert_eq!((out.width, out.height), (2, 3));
        assert_eq!(out.data, vec![2.0, 5.0, 1.0, 4.0, 0.0, 3.0]);
    }

    #[test]
    fn a_half_turn_reverses_the_pixels() {
        let out = numbered(3, 2, 1).turned(2);
        assert_eq!(out.data, vec![5.0, 4.0, 3.0, 2.0, 1.0, 0.0]);
        assert_eq!(out, numbered(3, 2, 1).flipped(true, true));
    }

    #[test]
    fn four_quarter_turns_are_the_identity_bit_for_bit() {
        for channels in [1u8, 3] {
            let image = numbered(7, 4, channels);
            let mut turned = image.clone();
            for _ in 0..4 {
                turned = turned.turned(1);
            }
            assert_eq!(turned, image);
            assert_eq!(image.turned(0), image);
            assert_eq!(image.turned(4), image);
        }
    }

    #[test]
    fn a_quarter_turn_keeps_the_channels_of_a_pixel_together() {
        let out = numbered(2, 1, 3).turned(1);
        assert_eq!((out.width, out.height), (1, 2));
        assert_eq!(out.data, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    // --- Orientation ---

    #[test]
    fn an_angle_of_zero_and_no_flip_is_the_identity_bit_for_bit() {
        let image = numbered(6, 5, 3);
        assert!(Orientation::default().is_neutral());
        assert_eq!(image.oriented(&Orientation::default()), image);
        for method in [
            Resampling::Nearest,
            Resampling::Bicubic,
            Resampling::Lanczos3,
        ] {
            let orientation = Orientation {
                method,
                ..Orientation::default()
            };
            assert_eq!(image.oriented(&orientation), image);
        }
    }

    #[test]
    fn angles_that_are_multiples_of_90_turn_exactly() {
        let image = numbered(3, 2, 3);
        assert_eq!(image.oriented(&turn(90.0)), image.turned(1));
        assert_eq!(image.oriented(&turn(-90.0)), image.turned(3));
        assert_eq!(image.oriented(&turn(180.0)), image.turned(2));
        assert_eq!(image.oriented(&turn(-180.0)), image.turned(2));
    }

    #[test]
    fn flipping_comes_before_turning() {
        let image = numbered(3, 2, 1);
        let orientation = Orientation {
            flip_horizontal: true,
            angle: 90.0,
            ..Orientation::default()
        };
        assert_eq!(
            image.oriented(&orientation),
            image.flipped(true, false).turned(1)
        );
    }

    #[test]
    fn a_free_angle_makes_the_canvas_as_big_as_the_turned_image() {
        // 100 x 50 turned by 30 degrees: 100 cos + 50 sin = 111.6 wide, 100 sin + 50 cos = 93.3 high.
        let image = constant(100, 50, 1, 1.0);
        let out = image.oriented(&turn(30.0));
        assert_eq!((out.width, out.height), (112, 94));
        assert_eq!(turn(30.0).output_size(100, 50), (112, 94));
        assert_eq!(turn(-30.0).output_size(100, 50), (112, 94));
        assert_eq!(turn(90.0).output_size(100, 50), (50, 100));
        assert_eq!(turn(0.0).output_size(100, 50), (100, 50));
        assert_eq!(turn(180.0).output_size(100, 50), (100, 50));
    }

    #[test]
    fn a_free_angle_keeps_the_pixel_in_the_centre() {
        for method in [
            Resampling::Nearest,
            Resampling::Bilinear,
            Resampling::Bicubic,
            Resampling::Lanczos3,
        ] {
            let mut image = constant(21, 21, 1, 0.2);
            image.data[10 * 21 + 10] = 0.9;
            for angle in [15.0, -37.5, 60.0, 135.0] {
                let out = image.oriented(&Orientation {
                    angle,
                    method,
                    ..Orientation::default()
                });
                let centre = pixel(&out, out.width / 2, out.height / 2)[0];
                // The canvas grows, and keeps the parity of the source so that its centre is the centre of a pixel.
                assert!(
                    out.width > 21 && out.height > 21 && out.width % 2 == 1 && out.height % 2 == 1,
                    "{angle}: {}x{}",
                    out.width,
                    out.height
                );
                assert!((centre - 0.9).abs() < 1e-4, "{method:?} {angle}: {centre}");
            }
        }
    }

    #[test]
    fn a_free_angle_turns_clockwise_and_leaves_black_corners() {
        // A bright mark at the top centre of a bright image moves to the right of the centre when turned clockwise.
        let mut image = Image::new(41, 41, 1);
        image.data.fill(0.0);
        image.data[2 * 41 + 20] = 1.0;
        let out = image.oriented(&turn(30.0));
        let (cx, cy) = (out.width as f32 / 2.0, out.height as f32 / 2.0);
        let (mut best, mut at) = (0.0, (0.0, 0.0));
        for y in 0..out.height {
            for x in 0..out.width {
                let v = pixel(&out, x, y)[0];
                if v > best {
                    best = v;
                    at = (x as f32 + 0.5, y as f32 + 0.5);
                }
            }
        }
        assert!(at.0 > cx + 5.0, "mark at {at:?}, centre ({cx}, {cy})");
        assert!(at.1 < cy, "mark at {at:?}, centre ({cx}, {cy})");

        let white = constant(41, 41, 1, 1.0).oriented(&turn(30.0));
        assert_eq!(pixel(&white, 0, 0), vec![0.0]);
        assert_eq!(pixel(&white, white.width / 2, white.height / 2), vec![1.0]);
    }

    #[test]
    fn crop_to_fit_leaves_no_black_corner() {
        let white = constant(100, 60, 3, 1.0);
        for angle in [10.0, -25.5, 44.0, 100.0, 170.0, -135.0] {
            for method in [Resampling::Bilinear, Resampling::Bicubic] {
                let orientation = Orientation {
                    angle,
                    method,
                    crop_to_fit: true,
                    ..Orientation::default()
                };
                let out = white.oriented(&orientation);
                assert!(out.width < 100 + 60 && out.height < 100 + 60);
                assert!(
                    out.data.iter().all(|&v| (v - 1.0).abs() < 1e-5),
                    "{angle} {method:?}: a pixel is not white"
                );
                assert_eq!((out.width, out.height), orientation.output_size(100, 60));
            }
        }
        // 100 x 60 at 10 degrees: the largest rectangle of that shape inside is smaller than the canvas but keeps the shape.
        let out = white.oriented(&Orientation {
            angle: 10.0,
            crop_to_fit: true,
            ..Orientation::default()
        });
        assert!(out.width < 100 && out.height < 60);
        let shape = out.width as f32 / out.height as f32;
        assert!((shape - 100.0 / 60.0).abs() < 0.05, "{shape}");
    }

    #[test]
    fn crop_to_fit_does_not_change_exact_turns() {
        let image = numbered(5, 3, 1);
        for angle in [0.0, 90.0, 180.0, -90.0] {
            let fitted = Orientation {
                angle,
                crop_to_fit: true,
                ..Orientation::default()
            };
            assert_eq!(image.oriented(&fitted), image.oriented(&turn(angle)));
        }
    }

    // --- Resize ---

    fn pixels(width: Option<u32>, height: Option<u32>) -> Resize {
        Resize {
            target: ResizeTarget::Pixels { width, height },
            method: Resampling::Lanczos3,
        }
    }

    fn percent(p: f32) -> Resize {
        Resize {
            target: ResizeTarget::Percent(p),
            method: Resampling::Lanczos3,
        }
    }

    #[test]
    fn a_percentage_scales_both_sides() {
        assert_eq!(percent(100.0).output_size(1200, 900), (1200, 900));
        assert_eq!(percent(50.0).output_size(1200, 900), (600, 450));
        assert_eq!(percent(200.0).output_size(1200, 900), (2400, 1800));
        assert_eq!(percent(33.3).output_size(1000, 750), (333, 250));
        // Halves round up; a side is never smaller than one pixel.
        assert_eq!(percent(50.0).output_size(101, 11), (51, 6));
        assert_eq!(percent(10.0).output_size(4, 4), (1, 1));
    }

    #[test]
    fn a_percentage_stays_within_the_limits_of_the_card() {
        assert_eq!(percent(1.0).output_size(1000, 500), (100, 50));
        assert_eq!(percent(9000.0).output_size(100, 50), (400, 200));
    }

    #[test]
    fn a_size_in_pixels_with_one_side_keeps_the_shape() {
        assert_eq!(pixels(Some(800), None).output_size(1200, 900), (800, 600));
        assert_eq!(pixels(None, Some(450)).output_size(1200, 900), (600, 450));
        assert_eq!(pixels(Some(1), None).output_size(1200, 900), (1, 1));
        assert_eq!(pixels(Some(1000), None).output_size(3, 1), (1000, 333));
    }

    #[test]
    fn a_size_in_pixels_with_both_sides_is_taken_as_it_is() {
        assert_eq!(
            pixels(Some(800), Some(800)).output_size(1200, 900),
            (800, 800)
        );
        assert_eq!(pixels(None, None).output_size(1200, 900), (1200, 900));
        assert_eq!(pixels(Some(0), Some(0)).output_size(10, 10), (1, 1));
        assert_eq!(
            pixels(Some(99_999), None).output_size(10, 10),
            (MAX_SIDE, MAX_SIDE)
        );
    }

    #[test]
    fn resizing_to_the_size_it_has_is_the_identity_bit_for_bit() {
        let image = numbered(7, 5, 3);
        for method in [
            Resampling::Nearest,
            Resampling::Bilinear,
            Resampling::Bicubic,
            Resampling::Lanczos3,
        ] {
            let resize = Resize {
                target: ResizeTarget::Percent(100.0),
                method,
            };
            assert!(resize.is_neutral(7, 5));
            assert_eq!(image.resized(&resize), image, "{method:?}");
        }
        assert!(pixels(Some(7), Some(5)).is_neutral(7, 5));
        assert!(!percent(101.0).is_neutral(100, 100));
    }

    #[test]
    fn the_resized_image_has_the_size_asked_for_and_the_same_channels() {
        for channels in [1u8, 3] {
            let image = numbered(10, 8, channels);
            let out = image.resized(&percent(150.0));
            assert_eq!((out.width, out.height, out.channels), (15, 12, channels));
            assert_eq!(out.data.len(), 15 * 12 * usize::from(channels));
        }
    }

    #[test]
    fn every_method_keeps_a_constant_image_constant() {
        let image = constant(13, 9, 3, 0.37);
        for method in [
            Resampling::Nearest,
            Resampling::Bilinear,
            Resampling::Bicubic,
            Resampling::Lanczos3,
        ] {
            for p in [37.0, 150.0, 400.0] {
                let out = image.resized(&Resize {
                    target: ResizeTarget::Percent(p),
                    method,
                });
                assert_eq!(
                    (out.width, out.height),
                    (
                        (13.0 * p / 100.0_f32).round() as u32,
                        (9.0 * p / 100.0_f32).round() as u32
                    )
                );
                assert!(
                    out.data.iter().all(|&v| (v - 0.37).abs() < 1e-5),
                    "{method:?} {p} %"
                );
            }
        }
    }

    #[test]
    fn nearest_neighbour_doubles_every_pixel() {
        let out = numbered(2, 1, 1).resized(&Resize {
            target: ResizeTarget::Percent(200.0),
            method: Resampling::Nearest,
        });
        assert_eq!((out.width, out.height), (4, 2));
        assert_eq!(out.data, vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn bilinear_halving_averages_two_by_two_pixels() {
        let mut image = Image::new(2, 2, 1);
        image.data = vec![0.0, 1.0, 0.5, 0.25];
        let out = image.resized(&pixels_resize(1, 1, Resampling::Bilinear));
        assert_eq!((out.width, out.height), (1, 1));
        assert!((out.data[0] - 0.4375).abs() < 1e-6, "{}", out.data[0]);
    }

    fn pixels_resize(width: u32, height: u32, method: Resampling) -> Resize {
        Resize {
            target: ResizeTarget::Pixels {
                width: Some(width),
                height: Some(height),
            },
            method,
        }
    }

    #[test]
    fn resizing_never_leaves_the_range_of_the_samples() {
        // Bicubic and Lanczos overshoot at a hard edge; the samples must stay between 0 and 1.
        let mut image = Image::new(8, 1, 1);
        image.data = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        for method in [Resampling::Bicubic, Resampling::Lanczos3] {
            let out = image.resized(&pixels_resize(37, 3, method));
            assert!(
                out.data.iter().all(|v| (0.0..=1.0).contains(v)),
                "{method:?}"
            );
            // And the edge is still where it was: dark on the left, bright on the right.
            assert!(out.data[0] < 0.01 && out.data[36] > 0.99, "{method:?}");
        }
    }

    // --- Order ---

    #[test]
    fn the_crop_is_set_on_the_oriented_image_and_the_resize_comes_last() {
        // 4 wide, 2 high; turned clockwise it is 2 wide, 4 high: 4 0 / 5 1 / 6 2 / 7 3.
        // Samples are normalised, so the numbers are tenths.
        let mut image = numbered(4, 2, 1);
        image.data.iter_mut().for_each(|v| *v /= 10.0);
        let geometry = Geometry {
            orientation: turn(90.0),
            resize: Some(Resize {
                target: ResizeTarget::Percent(200.0),
                method: Resampling::Nearest,
            }),
        };
        let crop = CropRect {
            x: 0,
            y: 1,
            width: 2,
            height: 2,
        };
        let out = geometry.finish(&image, Some(crop)).unwrap();
        // The part 5 1 / 6 2, then twice as big.
        assert_eq!((out.width, out.height), (4, 4));
        let tenths = [
            5.0, 5.0, 1.0, 1.0, 5.0, 5.0, 1.0, 1.0, 6.0, 6.0, 2.0, 2.0, 6.0, 6.0, 2.0, 2.0,
        ];
        assert_eq!(
            out.data,
            tenths.iter().map(|v| v / 10.0).collect::<Vec<f32>>()
        );
        // The same rectangle cut before turning is another part (the unturned image is only 2 high, so it holds one row of it).
        let crop_first = image.cropped(crop).unwrap().oriented(&turn(90.0));
        assert_eq!((crop_first.width, crop_first.height), (1, 2));
    }

    #[test]
    fn a_crop_outside_the_oriented_image_is_none() {
        let image = numbered(4, 2, 1);
        let geometry = Geometry {
            orientation: turn(90.0),
            resize: None,
        };
        // The turned image is 2 wide: a rectangle that starts at x = 3 is outside it, though it lies inside the original.
        let crop = CropRect {
            x: 3,
            y: 0,
            width: 1,
            height: 1,
        };
        assert_eq!(geometry.finish(&image, Some(crop)), None);
        assert!(Geometry::default().finish(&image, Some(crop)).is_some());
    }

    #[test]
    fn the_default_geometry_changes_nothing() {
        let image = numbered(6, 4, 3);
        assert_eq!(Geometry::default().finish(&image, None), Some(image));
    }
}
