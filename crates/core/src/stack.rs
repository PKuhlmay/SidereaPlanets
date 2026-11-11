//! Stacking: the registered frames are added up into one image.
//!
//! Every frame is shifted back onto the reference frame before it is averaged. With *Correct
//! geometry* the shift is not one value per frame but varies over the image: it is blended from the
//! movements of the nearest alignpoints (*Use Nearestby Alignpoints*), so a frame that is distorted
//! by seeing is straightened locally. How the original does this is unobserved, the scheme here is
//! our own guess and is checked against its result (`docs/stacking.md`, `docs/testdaten.md`).
//!
//! Frames are processed in small batches, so that only a few frames are in memory at a time. The sum
//! is kept in `f64`: thousands of 16-bit frames add up without overflow or visible rounding.

use crate::{
    Image,
    align::{AlignPoint, Control, FrameAlignment, Reference},
};
use rayon::prelude::*;

/// Settings of *Stacking options* with the defaults of the reference program (`docs/ui.md`).
#[derive(Debug, Clone, PartialEq)]
pub struct StackParams {
    /// *Use Nearestby Alignpoints*: how many of the nearest points decide the shift of a pixel.
    pub nearest_points: usize,
    /// *Correct geometry*: shift every pixel by its local movement instead of the frame as a whole.
    pub correct_geometry: bool,
    /// *Normalization of frame intensity*: scale each frame to the mean brightness of the reference.
    pub normalize: bool,
    /// *Do not stack n pixels at the frameedge*: this border of every frame is left out.
    pub edge_skip: u32,
}

impl Default for StackParams {
    fn default() -> Self {
        Self {
            nearest_points: 3,
            correct_geometry: true,
            normalize: false,
            edge_skip: 0,
        }
    }
}

/// Spacing in pixels of the grid on which the shift is evaluated; in between it is interpolated.
const FIELD_STEP: usize = 4;

/// Where an alignpoint of the reference was found in a frame.
struct Anchor {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

/// The shift of every pixel of one frame, on a coarse grid. `frame(x + dx, y + dy)` shows what the
/// reference shows at `(x, y)`.
struct Field {
    cols: usize,
    rows: usize,
    data: Vec<(f32, f32)>,
}

impl Field {
    /// The same shift everywhere.
    fn uniform(shift: (f32, f32)) -> Self {
        Self {
            cols: 1,
            rows: 1,
            data: vec![shift],
        }
    }

    /// The shift blended from the `nearest` anchors at every grid node.
    fn from_anchors(width: usize, height: usize, anchors: &[Anchor], nearest: usize) -> Self {
        let cols = (width.saturating_sub(1)) / FIELD_STEP + 2;
        let rows = (height.saturating_sub(1)) / FIELD_STEP + 2;
        let mut data = vec![(0.0, 0.0); cols * rows];
        data.par_chunks_mut(cols).enumerate().for_each(|(j, row)| {
            for (i, node) in row.iter_mut().enumerate() {
                *node = blend(
                    anchors,
                    nearest,
                    (i * FIELD_STEP) as f32,
                    (j * FIELD_STEP) as f32,
                );
            }
        });
        Self { cols, rows, data }
    }

    /// The shift at a pixel, bilinear between the four surrounding nodes.
    fn at(&self, x: f32, y: f32) -> (f32, f32) {
        let locate = |v: f32, n: usize| {
            let g = v / FIELD_STEP as f32;
            let i0 = (g.floor().max(0.0) as usize).min(n - 1);
            (i0, (i0 + 1).min(n - 1), (g - i0 as f32).clamp(0.0, 1.0))
        };
        let (x0, x1, tx) = locate(x, self.cols);
        let (y0, y1, ty) = locate(y, self.rows);
        let node = |i: usize, j: usize| self.data[j * self.cols + i];
        let mix =
            |a: (f32, f32), b: (f32, f32), t: f32| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        mix(
            mix(node(x0, y0), node(x1, y0), tx),
            mix(node(x0, y1), node(x1, y1), tx),
            ty,
        )
    }
}

/// The movement at `(x, y)`: a weighted mean of the `nearest` anchors closest to it. The weights
/// fall to zero at the distance of the next anchor in line, so the result does not jump when the
/// set of nearest anchors changes. Without anchors nothing moves.
fn blend(anchors: &[Anchor], nearest: usize, x: f32, y: f32) -> (f32, f32) {
    let nearest = nearest.max(1);
    // Anchors ordered by squared distance; one more than needed, to know where the weights end.
    let mut near: Vec<(f32, &Anchor)> = Vec::with_capacity(nearest + 1);
    for a in anchors {
        let d2 = (a.x - x).powi(2) + (a.y - y).powi(2);
        if near.len() == nearest + 1 && d2 >= near[nearest].0 {
            continue;
        }
        let at = near.partition_point(|&(other, _)| other <= d2);
        near.insert(at, (d2, a));
        near.truncate(nearest + 1);
    }
    let Some(&(closest, a)) = near.first() else {
        return (0.0, 0.0);
    };
    if closest < 1e-6 {
        return (a.dx, a.dy);
    }
    let radius = (near.len() > nearest).then(|| near[nearest].0.sqrt());
    let weight = |d2: f32| {
        let d = d2.sqrt();
        match radius {
            Some(r) => ((r - d).max(0.0) / (r * d)).powi(2),
            None => 1.0 / d2,
        }
    };
    let mut weights: Vec<f32> = near
        .iter()
        .take(nearest)
        .map(|&(d2, _)| weight(d2))
        .collect();
    // Anchors at exactly the same distance as the next in line all get weight zero.
    if weights.iter().sum::<f32>() <= 0.0 {
        weights = near.iter().take(nearest).map(|&(d2, _)| 1.0 / d2).collect();
    }
    let total: f32 = weights.iter().sum();
    let (mut dx, mut dy) = (0.0, 0.0);
    for (&w, &(_, a)) in weights.iter().zip(&near) {
        dx += w * a.dx;
        dy += w * a.dy;
    }
    (dx / total, dy / total)
}

/// Bilinear sample of `frame` at `(x, y)` into `out` (one value per channel). Returns `false`, and
/// leaves `out` alone, if the position is closer than `margin` pixels to the edge or outside.
fn sample(frame: &Image, x: f32, y: f32, margin: f32, out: &mut [f32]) -> bool {
    let (w, h) = (frame.width as usize, frame.height as usize);
    let channels = usize::from(frame.channels);
    if w == 0 || h == 0 {
        return false;
    }
    if !(x >= margin && y >= margin && x <= (w - 1) as f32 - margin && y <= (h - 1) as f32 - margin)
    {
        return false;
    }
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (tx, ty) = (x - x0 as f32, y - y0 as f32);
    for (c, value) in out.iter_mut().enumerate().take(channels) {
        let at = |xx: usize, yy: usize| frame.data[(yy * w + xx) * channels + c];
        let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * tx;
        let bottom = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * tx;
        *value = top + (bottom - top) * ty;
    }
    true
}

/// The running sum of a stack, which receives batches of registered frames.
pub struct Stacker<'a> {
    reference: &'a Reference,
    params: StackParams,
    width: usize,
    height: usize,
    channels: usize,
    /// Mean sample value of the reference frame, the target of the intensity normalisation.
    target_mean: f32,
    /// Sum of the samples of all frames that cover a pixel.
    sum: Vec<f64>,
    /// Number of frames that cover each pixel; a frame leaves out what lies beyond its edge.
    count: Vec<u32>,
    frames: u32,
}

impl<'a> Stacker<'a> {
    /// `reference_frame` is the frame that `reference` was made from; it fixes size and channels.
    pub fn new(reference: &'a Reference, reference_frame: &Image, params: StackParams) -> Self {
        let (width, height) = (
            reference_frame.width as usize,
            reference_frame.height as usize,
        );
        let channels = usize::from(reference_frame.channels).max(1);
        let target_mean = mean(&reference_frame.data);
        Self {
            reference,
            params,
            width,
            height,
            channels,
            target_mean,
            sum: vec![0.0; width * height * channels],
            count: vec![0; width * height],
            frames: 0,
        }
    }

    /// How many frames have been added. Frames of another size or channel count are left out.
    pub fn frames(&self) -> u32 {
        self.frames
    }

    /// Adds frames together with the result of registering them against the reference. The frames
    /// are added in parallel by image row.
    pub fn add_aligned(&mut self, batch: &[(&Image, &FrameAlignment)]) {
        let usable: Vec<&(&Image, &FrameAlignment)> = batch
            .iter()
            .filter(|(f, _)| {
                f.width as usize == self.width
                    && f.height as usize == self.height
                    && usize::from(f.channels).max(1) == self.channels
            })
            .collect();
        if usable.is_empty() {
            return;
        }
        let fields: Vec<Field> = usable
            .par_iter()
            .map(|(_, alignment)| self.field(alignment))
            .collect();
        let gains: Vec<f32> = usable.iter().map(|(frame, _)| self.gain(frame)).collect();

        let (width, channels) = (self.width, self.channels);
        let margin = self.params.edge_skip as f32;
        self.sum
            .par_chunks_mut(width * channels)
            .zip(self.count.par_chunks_mut(width))
            .enumerate()
            .for_each(|(y, (sum_row, count_row))| {
                let mut px = vec![0.0f32; channels];
                for (&&(frame, _), (field, &gain)) in usable.iter().zip(fields.iter().zip(&gains)) {
                    for x in 0..width {
                        let (dx, dy) = field.at(x as f32, y as f32);
                        if !sample(frame, x as f32 + dx, y as f32 + dy, margin, &mut px) {
                            continue;
                        }
                        for (total, &v) in sum_row[x * channels..(x + 1) * channels]
                            .iter_mut()
                            .zip(&px)
                        {
                            *total += f64::from(v * gain);
                        }
                        count_row[x] += 1;
                    }
                }
            });
        self.frames += usable.len() as u32;
    }

    /// The shift of each pixel of a frame.
    fn field(&self, alignment: &FrameAlignment) -> Field {
        if !self.params.correct_geometry {
            return Field::uniform(alignment.shift);
        }
        let anchors: Vec<Anchor> = self
            .reference
            .points()
            .iter()
            .zip(&alignment.points)
            .filter_map(|(p, shift)| {
                shift.map(|s| Anchor {
                    x: p.x as f32,
                    y: p.y as f32,
                    dx: s.dx,
                    dy: s.dy,
                })
            })
            .collect();
        Field::from_anchors(
            self.width,
            self.height,
            &anchors,
            self.params.nearest_points,
        )
    }

    /// The factor that brings a frame to the brightness of the reference, if that is asked for.
    fn gain(&self, frame: &Image) -> f32 {
        let frame_mean = mean(&frame.data);
        if self.params.normalize && frame_mean > 0.0 {
            self.target_mean / frame_mean
        } else {
            1.0
        }
    }

    /// The mean of all frames. A pixel that no frame covers stays black.
    pub fn finish(self) -> Image {
        let channels = self.channels;
        let data = self
            .sum
            .chunks_exact(channels)
            .zip(&self.count)
            .flat_map(|(px, &n)| {
                px.iter().map(move |&s| {
                    if n == 0 {
                        0.0
                    } else {
                        (s / f64::from(n)) as f32
                    }
                })
            })
            .collect();
        Image {
            width: self.width as u32,
            height: self.height as u32,
            channels: channels as u8,
            data,
        }
    }
}

fn mean(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    (data.iter().map(|&v| f64::from(v)).sum::<f64>() / data.len() as f64) as f32
}

/// Why [`stack_sequence`] stopped early.
#[derive(Debug, PartialEq, Eq)]
pub enum StackError<E> {
    /// There was no frame to stack.
    NoFrames,
    /// Reading a frame failed.
    Read(E),
    /// The progress callback asked to stop.
    Cancelled,
}

/// The finished stack.
#[derive(Debug, Clone, PartialEq)]
pub struct StackResult {
    /// The mean of the frames, same size and channels as the reference frame, samples in `0.0..=1.0`
    /// for input in that range.
    pub image: Image,
    /// How many frames went into it.
    pub frames: u32,
}

/// Registers the frames `selection` (indices as `read` takes them) against `reference` and stacks
/// them. Frames are read one after the other (`read` may borrow a file), registered and added in
/// parallel, a batch at a time. `progress` receives the number of finished frames after each batch.
pub fn stack_sequence<E>(
    reference: &Reference,
    reference_frame: &Image,
    params: &StackParams,
    selection: &[u32],
    mut read: impl FnMut(u32) -> Result<Image, E>,
    mut progress: impl FnMut(u32) -> Control,
) -> Result<StackResult, StackError<E>> {
    if selection.is_empty() {
        return Err(StackError::NoFrames);
    }
    let mut stacker = Stacker::new(reference, reference_frame, params.clone());
    let batch = (rayon::current_num_threads() * 2).max(1);
    let mut done = 0;
    for indices in selection.chunks(batch) {
        let frames = indices
            .iter()
            .map(|&i| read(i))
            .collect::<Result<Vec<_>, _>>()
            .map_err(StackError::Read)?;
        let alignments: Vec<FrameAlignment> = frames
            .par_iter()
            .map(|f| reference.align_frame(f))
            .collect();
        let pairs: Vec<(&Image, &FrameAlignment)> = frames.iter().zip(&alignments).collect();
        stacker.add_aligned(&pairs);
        done += indices.len() as u32;
        if progress(done) == Control::Cancel {
            return Err(StackError::Cancelled);
        }
    }
    let frames = stacker.frames();
    Ok(StackResult {
        image: stacker.finish(),
        frames,
    })
}

/// Peak signal to noise ratio in dB between two images of the same size, for samples in
/// `0.0..=1.0` (the peak is 1.0, i.e. 65535 in 16 bit). `None` if the sizes differ; infinite if the
/// images are identical. This is the measure of `docs/testdaten.md` for comparisons with the original.
pub fn psnr(a: &Image, b: &Image) -> Option<f64> {
    if (a.width, a.height, a.channels) != (b.width, b.height, b.channels) || a.data.is_empty() {
        return None;
    }
    let mse = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(&x, &y)| (f64::from(x) - f64::from(y)).powi(2))
        .sum::<f64>()
        / a.data.len() as f64;
    Some(10.0 * (1.0 / mse).log10())
}

/// The points a stack was registered with, for callers that only have positions (the UI).
pub fn points_from_positions(positions: impl IntoIterator<Item = (u32, u32)>) -> Vec<AlignPoint> {
    positions
        .into_iter()
        .map(|(x, y)| AlignPoint {
            x,
            y,
            strength: 0.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::align::{AlignParams, PointShift};

    /// A smooth, structured test picture: a sum of sines in both directions, sampled at `(x, y)`
    /// shifted by `shift`, so `scene(.., shift)` shows the same content moved by `shift` pixels.
    fn scene(width: u32, height: u32, shift: (f32, f32)) -> Image {
        let mut img = Image::new(width, height, 1);
        for y in 0..height {
            for x in 0..width {
                let (px, py) = (x as f32 - shift.0, y as f32 - shift.1);
                img.data[(y * width + x) as usize] = 0.5
                    + 0.2 * (px * 0.35).sin()
                    + 0.2 * (py * 0.27).cos()
                    + 0.1 * ((px + py) * 0.15).sin();
            }
        }
        img
    }

    fn grid_points(width: u32, height: u32, step: u32) -> Vec<AlignPoint> {
        points_from_positions((step..height - step).step_by(step as usize).flat_map(|y| {
            (step..width - step)
                .step_by(step as usize)
                .map(move |x| (x, y))
        }))
    }

    /// A registration result with every point moved by `shift`.
    fn uniform_alignment(points: usize, shift: (f32, f32)) -> FrameAlignment {
        FrameAlignment {
            shift,
            points: vec![
                Some(PointShift {
                    dx: shift.0,
                    dy: shift.1,
                    score: 1.0,
                    at_limit: false,
                    quality: 0.0,
                });
                points
            ],
            quality: 1.0,
            difference: 0.0,
        }
    }

    fn reference_of(frame: &Image, points: &[AlignPoint]) -> Reference {
        Reference::new(frame, points, &AlignParams::default())
    }

    fn stack_of(
        reference: &Reference,
        reference_frame: &Image,
        params: StackParams,
        frames: &[(Image, FrameAlignment)],
    ) -> Image {
        let mut stacker = Stacker::new(reference, reference_frame, params);
        let pairs: Vec<(&Image, &FrameAlignment)> = frames.iter().map(|(f, a)| (f, a)).collect();
        stacker.add_aligned(&pairs);
        stacker.finish()
    }

    #[test]
    fn identical_frames_give_the_frame_back() {
        let frame = scene(40, 30, (0.0, 0.0));
        let points = grid_points(40, 30, 10);
        let reference = reference_of(&frame, &points);
        let frames: Vec<_> = (0..3)
            .map(|_| (frame.clone(), uniform_alignment(points.len(), (0.0, 0.0))))
            .collect();
        let stack = stack_of(&reference, &frame, StackParams::default(), &frames);
        assert!(psnr(&stack, &frame).unwrap() > 100.0);
    }

    #[test]
    fn shifted_frames_are_moved_back_before_they_are_averaged() {
        let frame = scene(60, 50, (0.0, 0.0));
        let points = grid_points(60, 50, 10);
        let reference = reference_of(&frame, &points);
        // Content that moved by `s` shows the reference position `p` at `p + s`.
        let shifts = [(2.0, -1.0), (-3.0, 2.0), (0.0, 0.0)];
        let frames: Vec<_> = shifts
            .iter()
            .map(|&s| (scene(60, 50, s), uniform_alignment(points.len(), s)))
            .collect();
        let stack = stack_of(&reference, &frame, StackParams::default(), &frames);
        // Integer shifts are resampled exactly; only the borders that not every frame covers differ.
        let inner = |img: &Image| -> Vec<f32> {
            (10..40)
                .flat_map(|y| (10..50).map(move |x| (x, y)))
                .map(|(x, y)| img.data[y * 60 + x])
                .collect()
        };
        for (a, b) in inner(&stack).iter().zip(inner(&frame)) {
            assert!((a - b).abs() < 1e-5, "{a} against {b}");
        }
        // Without the registration the average would be blurred.
        let naive = {
            let mut sum = Image::new(60, 50, 1);
            for (f, _) in &frames {
                sum.data
                    .iter_mut()
                    .zip(&f.data)
                    .for_each(|(s, v)| *s += v / 3.0);
            }
            sum
        };
        assert!(psnr(&stack, &frame).unwrap() > psnr(&naive, &frame).unwrap() + 10.0);
    }

    #[test]
    fn local_correction_follows_the_nearest_points() {
        // Left half of the picture moved by +2, right half by -2: only a local shift can undo both.
        let (w, h) = (80u32, 40u32);
        let frame = scene(w, h, (0.0, 0.0));
        let left = scene(w, h, (2.0, 0.0));
        let right = scene(w, h, (-2.0, 0.0));
        let mut distorted = frame.clone();
        for y in 0..h as usize {
            for x in 0..w as usize {
                let src = if x < 40 { &left } else { &right };
                distorted.data[y * w as usize + x] = src.data[y * w as usize + x];
            }
        }
        let points = grid_points(w, h, 8);
        let reference = reference_of(&frame, &points);
        let alignment = FrameAlignment {
            shift: (0.0, 0.0),
            points: points
                .iter()
                .map(|p| {
                    Some(PointShift {
                        dx: if p.x < 40 { 2.0 } else { -2.0 },
                        dy: 0.0,
                        score: 1.0,
                        at_limit: false,
                        quality: 0.0,
                    })
                })
                .collect(),
            quality: 1.0,
            difference: 2.0,
        };
        let frames = [(distorted, alignment)];
        let local = stack_of(&reference, &frame, StackParams::default(), &frames);
        let global = stack_of(
            &reference,
            &frame,
            StackParams {
                correct_geometry: false,
                ..StackParams::default()
            },
            &frames,
        );
        // Far from the seam and the edges the local correction restores the picture exactly.
        for &(x, y) in &[(12usize, 20usize), (25, 10), (60, 20), (70, 30)] {
            let i = y * w as usize + x;
            assert!((local.data[i] - frame.data[i]).abs() < 1e-4, "at {x},{y}");
        }
        assert!(psnr(&local, &frame).unwrap() > psnr(&global, &frame).unwrap() + 5.0);
    }

    #[test]
    fn without_geometry_correction_the_frame_shift_is_used() {
        let frame = scene(50, 40, (0.0, 0.0));
        let points = grid_points(50, 40, 10);
        let reference = reference_of(&frame, &points);
        // Points disagree with the frame shift; with the correction off only the shift counts.
        let mut alignment = uniform_alignment(points.len(), (0.0, 0.0));
        alignment.points[0] = Some(PointShift {
            dx: 3.0,
            dy: 3.0,
            score: 1.0,
            at_limit: false,
            quality: 0.0,
        });
        let stack = stack_of(
            &reference,
            &frame,
            StackParams {
                correct_geometry: false,
                ..StackParams::default()
            },
            &[(frame.clone(), alignment)],
        );
        assert!(psnr(&stack, &frame).unwrap() > 100.0);
    }

    #[test]
    fn untracked_points_are_ignored() {
        let frame = scene(50, 40, (0.0, 0.0));
        let points = grid_points(50, 40, 10);
        let reference = reference_of(&frame, &points);
        let mut alignment = uniform_alignment(points.len(), (0.0, 0.0));
        alignment.points[1] = None;
        alignment.points[2] = None;
        let stack = stack_of(
            &reference,
            &frame,
            StackParams::default(),
            &[(frame.clone(), alignment)],
        );
        assert!(psnr(&stack, &frame).unwrap() > 100.0);
    }

    #[test]
    fn blend_is_exact_at_a_point_and_continuous_around_it() {
        let anchors = [
            Anchor {
                x: 0.0,
                y: 0.0,
                dx: 1.0,
                dy: 0.0,
            },
            Anchor {
                x: 10.0,
                y: 0.0,
                dx: 3.0,
                dy: 0.0,
            },
            Anchor {
                x: 0.0,
                y: 10.0,
                dx: 5.0,
                dy: 0.0,
            },
            Anchor {
                x: 10.0,
                y: 10.0,
                dx: 7.0,
                dy: 0.0,
            },
        ];
        assert_eq!(blend(&anchors, 3, 10.0, 0.0), (3.0, 0.0));
        // One step further the value moves by a small amount, not by a jump.
        let (near, _) = blend(&anchors, 3, 9.9, 0.0);
        assert!((near - 3.0).abs() < 0.2, "{near}");
        // Equally far from all four, the weights cancel the choice of the three nearest.
        let (centre, _) = blend(&anchors, 3, 5.0, 5.0);
        assert!((1.0..=7.0).contains(&centre));
        // No anchors, no movement.
        assert_eq!(blend(&[], 3, 5.0, 5.0), (0.0, 0.0));
    }

    #[test]
    fn a_single_anchor_moves_everything_alike() {
        let anchors = [Anchor {
            x: 3.0,
            y: 4.0,
            dx: -1.5,
            dy: 2.0,
        }];
        assert_eq!(blend(&anchors, 3, 50.0, 50.0), (-1.5, 2.0));
    }

    #[test]
    fn the_border_of_the_frame_can_be_left_out() {
        let frame = scene(30, 30, (0.0, 0.0));
        let points = grid_points(30, 30, 10);
        let reference = reference_of(&frame, &points);
        let mut stacker = Stacker::new(
            &reference,
            &frame,
            StackParams {
                edge_skip: 5,
                ..StackParams::default()
            },
        );
        let alignment = uniform_alignment(points.len(), (0.0, 0.0));
        stacker.add_aligned(&[(&frame, &alignment)]);
        // Nothing covers the border, the pixel stays black; inside, the frame is reproduced.
        assert_eq!(stacker.count[0], 0);
        assert_eq!(stacker.count[15 * 30 + 15], 1);
        let stack = stacker.finish();
        assert_eq!(stack.data[0], 0.0);
        assert!((stack.data[15 * 30 + 15] - frame.data[15 * 30 + 15]).abs() < 1e-6);
    }

    #[test]
    fn pixels_beyond_the_edge_use_only_the_frames_that_cover_them() {
        let frame = scene(40, 30, (0.0, 0.0));
        let points = grid_points(40, 30, 10);
        let reference = reference_of(&frame, &points);
        // The second frame shows the content 5 pixels to the right, so its last 5 columns lie beyond it.
        let frames = [
            (frame.clone(), uniform_alignment(points.len(), (0.0, 0.0))),
            (
                scene(40, 30, (5.0, 0.0)),
                uniform_alignment(points.len(), (5.0, 0.0)),
            ),
        ];
        let stack = stack_of(&reference, &frame, StackParams::default(), &frames);
        // There the mean is that of the first frame alone, not half of it.
        let i = 10 * 40 + 38;
        assert!((stack.data[i] - frame.data[i]).abs() < 1e-5);
    }

    #[test]
    fn normalisation_matches_the_brightness_of_the_reference() {
        let frame = scene(30, 30, (0.0, 0.0));
        let points = grid_points(30, 30, 10);
        let reference = reference_of(&frame, &points);
        let mut dim = frame.clone();
        dim.data.iter_mut().for_each(|v| *v *= 0.5);
        let alignment = uniform_alignment(points.len(), (0.0, 0.0));
        let stack = stack_of(
            &reference,
            &frame,
            StackParams {
                normalize: true,
                ..StackParams::default()
            },
            &[(dim, alignment)],
        );
        assert!(psnr(&stack, &frame).unwrap() > 100.0);
    }

    #[test]
    fn thousands_of_saturated_frames_do_not_overflow() {
        let mut one = Image::new(1, 1, 1);
        one.data[0] = 1.0;
        let points = [AlignPoint {
            x: 0,
            y: 0,
            strength: 0.0,
        }];
        let reference = reference_of(&one, &points);
        let alignment = uniform_alignment(1, (0.0, 0.0));
        let mut stacker = Stacker::new(&reference, &one, StackParams::default());
        // 70 000 frames of full scale: more than a 16 bit sum could hold.
        for _ in 0..70_000 {
            stacker.add_aligned(&[(&one, &alignment)]);
        }
        assert_eq!(stacker.frames(), 70_000);
        assert_eq!(stacker.finish().data[0], 1.0);
    }

    #[test]
    fn frames_of_another_size_are_left_out() {
        let frame = scene(30, 30, (0.0, 0.0));
        let points = grid_points(30, 30, 10);
        let reference = reference_of(&frame, &points);
        let other = scene(20, 20, (0.0, 0.0));
        let alignment = uniform_alignment(points.len(), (0.0, 0.0));
        let mut stacker = Stacker::new(&reference, &frame, StackParams::default());
        stacker.add_aligned(&[(&other, &alignment), (&frame, &alignment)]);
        assert_eq!(stacker.frames(), 1);
    }

    #[test]
    fn colour_channels_are_stacked_separately() {
        let mut frame = Image::new(20, 20, 3);
        const COLOUR: [f32; 3] = [0.2, 0.5, 0.8];
        for (i, v) in frame.data.iter_mut().enumerate() {
            *v = COLOUR[i % 3];
        }
        let points = grid_points(20, 20, 5);
        let reference = reference_of(&frame, &points);
        let alignment = uniform_alignment(points.len(), (0.0, 0.0));
        let stack = stack_of(
            &reference,
            &frame,
            StackParams::default(),
            &[(frame.clone(), alignment)],
        );
        assert_eq!(stack.channels, 3);
        assert!(
            stack
                .data
                .iter()
                .enumerate()
                .all(|(i, v)| (v - COLOUR[i % 3]).abs() < 1e-6)
        );
    }

    #[test]
    fn stack_sequence_registers_the_frames_itself_and_reports_progress() {
        let (w, h) = (96u32, 80u32);
        let frame = scene(w, h, (0.0, 0.0));
        let points = grid_points(w, h, 24);
        let reference = reference_of(&frame, &points);
        let shifts = [(0.0, 0.0), (2.0, 1.0), (-2.0, 2.0), (1.0, -2.0)];
        let mut reported = Vec::new();
        let result = stack_sequence(
            &reference,
            &frame,
            &StackParams::default(),
            &[0, 1, 2, 3],
            |i| Ok::<_, ()>(scene(w, h, shifts[i as usize])),
            |done| {
                reported.push(done);
                Control::Continue
            },
        )
        .unwrap();
        assert_eq!(result.frames, 4);
        assert_eq!(reported.last(), Some(&4));
        assert!(
            psnr(&result.image, &frame).unwrap() > 30.0,
            "{}",
            psnr(&result.image, &frame).unwrap()
        );
    }

    #[test]
    fn stack_sequence_stops_on_cancel_and_on_read_errors() {
        let frame = scene(40, 40, (0.0, 0.0));
        let points = grid_points(40, 40, 10);
        let reference = reference_of(&frame, &points);
        let params = StackParams::default();
        let cancelled = stack_sequence(
            &reference,
            &frame,
            &params,
            &[0, 1],
            |_| Ok::<_, ()>(frame.clone()),
            |_| Control::Cancel,
        );
        assert_eq!(cancelled.err(), Some(StackError::Cancelled));
        let failed = stack_sequence(
            &reference,
            &frame,
            &params,
            &[0],
            |_| Err("disk"),
            |_| Control::Continue,
        );
        assert_eq!(failed.err(), Some(StackError::Read("disk")));
        let empty = stack_sequence(
            &reference,
            &frame,
            &params,
            &[],
            |_| Ok::<_, ()>(frame.clone()),
            |_| Control::Continue,
        );
        assert_eq!(empty.err(), Some(StackError::NoFrames));
    }

    #[test]
    fn psnr_of_identical_images_is_infinite_and_of_different_sizes_is_none() {
        let a = scene(10, 10, (0.0, 0.0));
        assert_eq!(psnr(&a, &a), Some(f64::INFINITY));
        assert_eq!(psnr(&a, &scene(10, 11, (0.0, 0.0))), None);
        let mut b = a.clone();
        b.data.iter_mut().for_each(|v| *v += 0.01);
        // MSE 1e-4 at peak 1.0 is 40 dB.
        assert!((psnr(&a, &b).unwrap() - 40.0).abs() < 0.01);
    }
}
