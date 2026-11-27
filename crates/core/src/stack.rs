//! Stacking: the registered frames are added up into one image.
//!
//! Every frame is shifted back onto the reference frame before it is averaged. With *Correct
//! geometry* the shift is not one value per frame but varies over the image: it is blended from the
//! movements of the nearest alignpoints (*Use Nearestby Alignpoints*), so a frame that is distorted
//! by seeing is straightened locally. The scheme is our own design (`docs/stacking.md`) and is checked by a regression test
//! against stored reference results.
//!
//! Frames are processed in batches of about one frame per thread, so that only a few frames are in memory at a time. The sum
//! is kept in `f64`: thousands of 16-bit frames add up without overflow or visible rounding.

use crate::{
    Image,
    align::{AlignPoint, Control, FrameAlignment, Reference},
    batch::{Stop, batch_size, for_each_batch},
    derotate::Derotator,
};
use rayon::prelude::*;

/// Settings of *Stacking options* with its defaults (`docs/ui.md`).
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
#[derive(Clone, Copy)]
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
    fn from_anchors(
        width: usize,
        height: usize,
        anchors: &[Anchor],
        nearest: usize,
        base: (f32, f32),
        trust: f32,
    ) -> Self {
        let cols = (width.saturating_sub(1)) / FIELD_STEP + 2;
        let rows = (height.saturating_sub(1)) / FIELD_STEP + 2;
        let mut data = vec![(0.0, 0.0); cols * rows];
        data.par_chunks_mut(cols).enumerate().for_each(|(j, row)| {
            for (i, node) in row.iter_mut().enumerate() {
                let local = blend(
                    anchors,
                    nearest,
                    (i * FIELD_STEP) as f32,
                    (j * FIELD_STEP) as f32,
                );
                *node = (
                    base.0 + trust * (local.0 - base.0),
                    base.1 + trust * (local.1 - base.1),
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

/// How far the movements of the alignpoints can be trusted to differ from the movement of the frame
/// as a whole, `0.0..=1.0`. Every point is predicted from the other points (blended as for a pixel)
/// and from the frame shift alone; the share by which the first prediction is the better one is the
/// trust. Where the points move coherently (a real distortion) it is close to 1; where their
/// deviations are scatter of the matching (flat, blurred structure) it is 0 and the frame moves as a whole.
fn coherence(anchors: &[Anchor], nearest: usize, base: (f32, f32)) -> f32 {
    if anchors.len() < 2 {
        return 0.0;
    }
    let (mut local, mut global) = (0.0f64, 0.0f64);
    for (i, a) in anchors.iter().enumerate() {
        let others: Vec<Anchor> = anchors
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(_, o)| *o)
            .collect();
        let (px, py) = blend(&others, nearest, a.x, a.y);
        local += f64::from((px - a.dx).powi(2) + (py - a.dy).powi(2));
        global += f64::from((base.0 - a.dx).powi(2) + (base.1 - a.dy).powi(2));
    }
    if global <= 0.0 {
        return 0.0;
    }
    (1.0 - local / global).clamp(0.0, 1.0) as f32
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

    /// One frame moved onto the reference, the way [`Stacker::add_aligned`] would add it, but as an
    /// image of its own (*Save Aligned*). Pixels the frame does not cover stay black. `None` if the
    /// frame has another size or channel count than the reference frame.
    pub fn warp(&self, frame: &Image, alignment: &FrameAlignment) -> Option<Image> {
        let channels = usize::from(frame.channels).max(1);
        if frame.width as usize != self.width
            || frame.height as usize != self.height
            || channels != self.channels
        {
            return None;
        }
        let field = self.field(alignment);
        let gain = self.gain(frame);
        let width = self.width;
        let mut data = vec![0.0f32; width * self.height * channels];
        data.par_chunks_mut(width * channels)
            .enumerate()
            .for_each(|(y, row)| {
                let mut px = vec![0.0f32; channels];
                for x in 0..width {
                    let (dx, dy) = field.at(x as f32, y as f32);
                    if sample(frame, x as f32 + dx, y as f32 + dy, 0.0, &mut px) {
                        for (out, &v) in row[x * channels..(x + 1) * channels].iter_mut().zip(&px) {
                            *out = v * gain;
                        }
                    }
                }
            });
        Some(Image {
            width: frame.width,
            height: frame.height,
            channels: channels as u8,
            data,
        })
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
                // A match on the border of the search range only says that the movement is at least
                // that large, so it is not a position to follow.
                shift.filter(|s| !s.at_limit).map(|s| Anchor {
                    x: p.x as f32,
                    y: p.y as f32,
                    dx: s.dx,
                    dy: s.dy,
                })
            })
            .collect();
        let nearest = self.params.nearest_points;
        let trust = coherence(&anchors, nearest, alignment.shift);
        if trust <= 0.0 {
            return Field::uniform(alignment.shift);
        }
        Field::from_anchors(
            self.width,
            self.height,
            &anchors,
            nearest,
            alignment.shift,
            trust,
        )
    }

    /// The factor that brings a frame to the brightness of the reference, if that is asked for.
    fn gain(&self, frame: &Image) -> f32 {
        if !self.params.normalize {
            return 1.0;
        }
        let frame_mean = mean(&frame.data);
        if frame_mean > 0.0 {
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

/// What de-rotation needs while stacking: the measured planet and the time of every frame of the selection.
pub struct Derotation<'a> {
    pub derotator: &'a Derotator,
    /// For the frame at position `i` of the selection: its time minus the reference time, in seconds.
    pub offsets: &'a [f64],
}

/// Registers the frames `selection` (indices as `read` takes them) against `reference` and stacks
/// them. The frames are read in the order of `selection`, one batch of about one frame per thread of the
/// current `rayon` pool at a time (`read` may borrow a file; the next batch is read while the current one
/// is worked on, so memory holds two batches at most). The frames of a batch are registered in parallel and
/// added to the sum in the order of `selection`, so the stack is the one of adding frame after frame, bit
/// for bit, and does not depend on the number of threads. `progress` receives 1, 2, 3, ... after each frame
/// (once a batch is done) and a cancel takes effect after that batch.
pub fn stack_sequence<E: Send>(
    reference: &Reference,
    reference_frame: &Image,
    params: &StackParams,
    selection: &[u32],
    read: impl FnMut(u32) -> Result<Image, E> + Send,
    progress: impl FnMut(u32) -> Control + Send,
) -> Result<StackResult, StackError<E>> {
    stack_sequence_derotated(
        reference,
        reference_frame,
        params,
        None,
        selection,
        read,
        progress,
    )
}

/// [`stack_sequence`] with de-rotation: every frame of a batch is turned back to the reference time first, in parallel,
/// and then registered and added. `reference` and `reference_frame` must be the reference frame already turned to that time, or
/// the alignpoints would follow features that moved.
pub fn stack_sequence_derotated<E: Send>(
    reference: &Reference,
    reference_frame: &Image,
    params: &StackParams,
    derotation: Option<&Derotation>,
    selection: &[u32],
    read: impl FnMut(u32) -> Result<Image, E> + Send,
    mut progress: impl FnMut(u32) -> Control + Send,
) -> Result<StackResult, StackError<E>> {
    if selection.is_empty() {
        return Err(StackError::NoFrames);
    }
    let mut stacker = Stacker::new(reference, reference_frame, params.clone());
    let frame_bytes = reference_frame.data.len() * std::mem::size_of::<f32>();
    let mut done = 0u32;
    for_each_batch(selection, batch_size(frame_bytes), read, |frames| {
        let first = done as usize;
        let turned: Option<Vec<Image>> = derotation.map(|d| {
            frames
                .par_iter()
                .enumerate()
                .map(|(i, frame)| {
                    let offset = d.offsets.get(first + i).copied().unwrap_or(0.0);
                    d.derotator.derotate(frame, offset)
                })
                .collect()
        });
        let source: Vec<&Image> = match &turned {
            Some(images) => images.iter().collect(),
            None => frames.iter().collect(),
        };
        let alignments: Vec<FrameAlignment> = source
            .par_iter()
            .map(|frame| reference.align_frame(frame))
            .collect();
        let batch: Vec<(&Image, &FrameAlignment)> =
            source.iter().copied().zip(&alignments).collect();
        stacker.add_aligned(&batch);
        for _ in frames {
            done += 1;
            if progress(done) == Control::Cancel {
                return Control::Cancel;
            }
        }
        Control::Continue
    })
    .map_err(|stop| match stop {
        Stop::Read(error) => StackError::Read(error),
        Stop::Cancelled => StackError::Cancelled,
    })?;
    let frames = stacker.frames();
    Ok(StackResult {
        image: stacker.finish(),
        frames,
    })
}

/// Peak signal to noise ratio in dB between two images of the same size, for samples in
/// `0.0..=1.0` (the peak is 1.0, i.e. 65535 in 16 bit). `None` if the sizes differ; infinite if the
/// images are identical. It is the measure used by the regression tests against stored reference results.
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

    /// The test picture at `(px, py)`; see [`scene`].
    fn pattern(px: f32, py: f32) -> f32 {
        0.5 + 0.2 * (px * 0.35).sin() + 0.2 * (py * 0.27).cos() + 0.1 * ((px + py) * 0.15).sin()
    }

    #[test]
    fn local_correction_follows_a_coherent_distortion() {
        // The content is stretched about the middle: moved by `slope * (x - 40)`, from -2 px at the
        // left edge to +2 px at the right one. The frame shift (the median) is zero; only a shift
        // that varies over the picture straightens it.
        let (w, h) = (80u32, 40u32);
        let slope = 0.05f32;
        let frame = scene(w, h, (0.0, 0.0));
        let mut distorted = frame.clone();
        for y in 0..h {
            for x in 0..w {
                let moved = slope * (x as f32 - 40.0);
                distorted.data[(y * w + x) as usize] = pattern(x as f32 - moved, y as f32);
            }
        }
        let points = grid_points(w, h, 4);
        let reference = reference_of(&frame, &points);
        let alignment = FrameAlignment {
            shift: (0.0, 0.0),
            points: points
                .iter()
                .map(|p| {
                    // `frame(x + d)` shows the reference at `x`: d = slope * (x + d - 40).
                    Some(PointShift {
                        dx: slope * (p.x as f32 - 40.0) / (1.0 - slope),
                        dy: 0.0,
                        score: 1.0,
                        at_limit: false,
                        quality: 0.0,
                    })
                })
                .collect(),
            quality: 1.0,
            difference: 1.0,
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
        // Compared away from the edges: a frame that moved right does not reach the left edge.
        let inner_db = |img: &Image| {
            let mse = (0..h as usize)
                .flat_map(|y| (6..w as usize - 6).map(move |x| y * w as usize + x))
                .map(|i| f64::from(img.data[i] - frame.data[i]).powi(2))
                .sum::<f64>()
                / (h as usize * (w as usize - 12)) as f64;
            10.0 * (1.0 / mse).log10()
        };
        let (local_db, global_db) = (inner_db(&local), inner_db(&global));
        assert!(
            local_db > global_db + 10.0,
            "{local_db} dB against {global_db} dB"
        );
    }

    #[test]
    fn scatter_of_the_points_does_not_distort_the_frame() {
        // The picture did not move at all, but the matches of the points scatter by a few pixels, as
        // they do on flat blurred structure. Following them would smear the frame; the stack must
        // fall back to the movement of the frame as a whole.
        let frame = scene(80, 40, (0.0, 0.0));
        let points = grid_points(80, 40, 8);
        let reference = reference_of(&frame, &points);
        let mut alignment = uniform_alignment(points.len(), (0.0, 0.0));
        for (i, p) in alignment.points.iter_mut().enumerate() {
            let scatter = |k: usize| ((i * k) % 13) as f32 / 2.0 - 3.0;
            *p = Some(PointShift {
                dx: scatter(7),
                dy: scatter(5),
                score: 0.9,
                at_limit: false,
                quality: 0.0,
            });
        }
        let stack = stack_of(
            &reference,
            &frame,
            StackParams::default(),
            &[(frame.clone(), alignment)],
        );
        assert!(psnr(&stack, &frame).unwrap() > 60.0);
    }

    #[test]
    fn matches_at_the_limit_of_the_search_range_are_not_followed() {
        // One point sits on the border of its search range with a movement of 5 px; the frame did not
        // move. The point is left out, the stack equals the frame.
        let frame = scene(60, 40, (0.0, 0.0));
        let points = grid_points(60, 40, 10);
        let reference = reference_of(&frame, &points);
        let mut alignment = uniform_alignment(points.len(), (0.0, 0.0));
        alignment.points[3] = Some(PointShift {
            dx: 5.0,
            dy: 0.0,
            score: 0.9,
            at_limit: true,
            quality: 0.0,
        });
        let stack = stack_of(
            &reference,
            &frame,
            StackParams::default(),
            &[(frame.clone(), alignment)],
        );
        assert!(psnr(&stack, &frame).unwrap() > 100.0);
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

    fn pool(threads: usize) -> rayon::ThreadPool {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
    }

    #[test]
    fn stack_sequence_reports_every_frame_in_order_and_stops_after_a_cancel() {
        let frame = scene(60, 50, (0.0, 0.0));
        let points = grid_points(60, 50, 12);
        let reference = reference_of(&frame, &points);
        let pool = pool(8);
        let mut reported = Vec::new();
        pool.install(|| {
            stack_sequence(
                &reference,
                &frame,
                &StackParams::default(),
                &[0, 0, 0, 0, 0, 0],
                |_| Ok::<_, ()>(frame.clone()),
                |done| {
                    reported.push(done);
                    Control::Continue
                },
            )
            .unwrap()
        });
        assert_eq!(reported, vec![1, 2, 3, 4, 5, 6]);

        // A cancel at the first report: at most the batch that was worked on and the one read next
        // (8 frames each) have been read, of 100.
        let reads = std::sync::atomic::AtomicUsize::new(0);
        let cancelled = pool.install(|| {
            stack_sequence(
                &reference,
                &frame,
                &StackParams::default(),
                &[0; 100],
                |_| {
                    reads.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    Ok::<_, ()>(frame.clone())
                },
                |_| Control::Cancel,
            )
        });
        assert_eq!(cancelled.err(), Some(StackError::Cancelled));
        assert!(reads.load(std::sync::atomic::Ordering::Relaxed) <= 16);
    }

    #[test]
    fn stack_sequence_reads_only_a_bounded_number_of_frames_ahead() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        let frame = scene(60, 50, (0.0, 0.0));
        let reference = reference_of(&frame, &grid_points(60, 50, 12));
        let reads = Arc::new(AtomicUsize::new(0));
        let counted = reads.clone();
        let mut worst_lead = 0;
        let source = &frame;
        pool(4).install(|| {
            stack_sequence(
                &reference,
                &frame,
                &StackParams::default(),
                &[0; 60],
                move |_| {
                    counted.fetch_add(1, Ordering::Relaxed);
                    Ok::<_, ()>(source.clone())
                },
                |done| {
                    worst_lead = worst_lead.max(reads.load(Ordering::Relaxed) - done as usize);
                    Control::Continue
                },
            )
            .unwrap()
        });
        // The batch that was worked on is done; one more is read or being read: 4 frames.
        assert!(worst_lead <= 8, "{worst_lead} frames were read ahead");
    }

    /// A colour frame with a distortion that differs from frame to frame, and its own brightness.
    fn distorted(index: u32, width: u32, height: u32) -> Image {
        let i = index as f32;
        let mut img = Image::new(width, height, 3);
        for y in 0..height {
            for x in 0..width {
                let px = x as f32 - 1.5 * (i * 0.7).sin() - 0.8 * (y as f32 * 0.05 + i).sin();
                let py = y as f32 - 1.2 * (i * 0.4).cos() - 0.6 * (x as f32 * 0.04 + i).cos();
                for c in 0..3 {
                    let v = 0.4
                        + 0.2 * (px * 0.35 + c as f32).sin()
                        + 0.2 * (py * 0.27).cos()
                        + 0.1 * ((px + py) * 0.15).sin();
                    img.data[((y * width + x) * 3 + c) as usize] =
                        v * (0.8 + 0.05 * (index % 5) as f32);
                }
            }
        }
        img
    }

    fn bits(image: &Image) -> Vec<u32> {
        image.data.iter().map(|v| v.to_bits()).collect()
    }

    /// The stack the way it was made before the frames went in batches: one frame after the other.
    fn stack_frame_by_frame(
        reference: &Reference,
        reference_frame: &Image,
        params: &StackParams,
        frames: &[Image],
    ) -> StackResult {
        let mut stacker = Stacker::new(reference, reference_frame, params.clone());
        for frame in frames {
            let alignment = reference.align_frame(frame);
            stacker.add_aligned(&[(frame, &alignment)]);
        }
        let frames = stacker.frames();
        StackResult {
            image: stacker.finish(),
            frames,
        }
    }

    #[test]
    fn stack_sequence_is_bit_identical_to_stacking_frame_by_frame_on_any_number_of_threads() {
        let (w, h) = (96, 80);
        let first = distorted(0, w, h);
        let reference = reference_of(&first, &grid_points(w, h, 20));
        let frames: Vec<Image> = (0..23).map(|i| distorted(i, w, h)).collect();
        // The mean must not hide an order: use the options that change the sum the most.
        for (geometry, normalize, edge_skip) in
            [(true, true, 3), (true, false, 0), (false, false, 0)]
        {
            let params = StackParams {
                correct_geometry: geometry,
                normalize,
                edge_skip,
                ..StackParams::default()
            };
            let expected = stack_frame_by_frame(&reference, &first, &params, &frames);
            for threads in [1, 2, 3, 8] {
                let selection: Vec<u32> = (0..frames.len() as u32).collect();
                let result = pool(threads).install(|| {
                    stack_sequence(
                        &reference,
                        &first,
                        &params,
                        &selection,
                        |i| Ok::<_, ()>(frames[i as usize].clone()),
                        |_| Control::Continue,
                    )
                    .unwrap()
                });
                assert_eq!(result.frames, expected.frames, "{threads} threads");
                assert!(
                    bits(&result.image) == bits(&expected.image),
                    "stack differs from frame by frame on {threads} threads \
                     (geometry {geometry}, normalize {normalize}, edge skip {edge_skip})"
                );
            }
        }
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

    /// Three frames of a rotating, synthetic Jupiter five minutes apart, the middle one as the reference.
    fn rotating_planet() -> (Vec<Image>, Vec<AlignPoint>) {
        use crate::synthetic::synthetic_jupiter;
        let frames: Vec<Image> = [0.0, 300.0, 600.0]
            .iter()
            .map(|&t| synthetic_jupiter(t).0)
            .collect();
        // Alignpoints on a grid over the disc (centre 160, 120; radius 100).
        let points = points_from_positions(
            (60..=180)
                .step_by(30)
                .flat_map(|y| (80..=240).step_by(30).map(move |x| (x, y))),
        );
        (frames, points)
    }

    #[test]
    fn de_rotated_frames_stack_to_the_reference_time_not_a_smear() {
        use crate::{derotate::Derotator, synthetic::synthetic_jupiter};
        let (frames, points) = rotating_planet();
        let view = synthetic_jupiter(0.0).1;
        let middle = &frames[1];
        let derotator = Derotator::new(view, middle).unwrap();
        let offsets = [-300.0, 0.0, 300.0];
        let reference = reference_of(middle, &points);
        let params = StackParams {
            correct_geometry: false,
            ..StackParams::default()
        };
        let run = |derotation: Option<&Derotation>| {
            stack_sequence_derotated(
                &reference,
                middle,
                &params,
                derotation,
                &[0, 1, 2],
                |i| Ok::<_, ()>(frames[i as usize].clone()),
                |_| Control::Continue,
            )
            .unwrap()
            .image
        };
        let plain = run(None);
        let turned = run(Some(&Derotation {
            derotator: &derotator,
            offsets: &offsets,
        }));
        // Only the disc counts: the dark sky around it is the same in all of them.
        let inside = |img: &Image| -> Image {
            let mut out = img.clone();
            for y in 0..240usize {
                for x in 0..320usize {
                    if ((x as f64 - 160.3).hypot(y as f64 - 120.6)) > 97.0 {
                        out.data[y * 320 + x] = 0.0;
                    }
                }
            }
            out
        };
        let target = inside(middle);
        let (with, without) = (
            psnr(&inside(&turned), &target).unwrap(),
            psnr(&inside(&plain), &target).unwrap(),
        );
        eprintln!("stack of 3 frames over 10 min: with {with:.2} dB, without {without:.2} dB");
        assert!(with > without + 4.0, "with {with} dB, without {without} dB");
        assert!(with > 45.0, "with {with} dB");
    }

    #[test]
    fn without_time_differences_de_rotation_changes_nothing() {
        use crate::derotate::Derotator;
        let (frames, points) = rotating_planet();
        let view = crate::synthetic::synthetic_jupiter(0.0).1;
        let derotator = Derotator::new(view, &frames[1]).unwrap();
        let reference = reference_of(&frames[1], &points);
        let run = |derotation: Option<&Derotation>| {
            stack_sequence_derotated(
                &reference,
                &frames[1],
                &StackParams::default(),
                derotation,
                &[0, 1, 2],
                |i| Ok::<_, ()>(frames[i as usize].clone()),
                |_| Control::Continue,
            )
            .unwrap()
        };
        let none = run(None);
        let zero = run(Some(&Derotation {
            derotator: &derotator,
            offsets: &[0.0, 0.0, 0.0],
        }));
        assert_eq!(none, zero);
    }
}
