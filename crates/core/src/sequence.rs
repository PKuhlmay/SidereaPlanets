//! Sequences of aligned frames for *Save Aligned* and *Save Integrated* (`docs/formats.md`, section
//! "Create AVI").
//!
//! [`aligned_frames`] moves the chosen frames onto the reference one after the other and hands each to
//! the caller, so thousands of frames never have to fit in memory. [`MovingAverage`] turns that stream
//! into the integrated sequence. The definition of *Save Integrated* used here is our own design.

use crate::{
    Image,
    align::{Control, FrameAlignment, Reference},
    stack::{StackParams, Stacker},
};
use std::collections::VecDeque;

/// Why [`aligned_frames`] stopped early.
#[derive(Debug, PartialEq, Eq)]
pub enum SequenceError<E> {
    /// There was no frame to write.
    NoFrames,
    /// Reading a frame failed.
    Read(E),
    /// Handing a finished frame to the caller failed.
    Write(E),
    /// The progress callback asked to stop.
    Cancelled,
}

/// Registers the frames `selection` against `reference` (or takes the registration from `known`,
/// indexed by frame, when it has one for the frame), moves each onto the reference like the stack
/// does and passes the result to `sink` in the order of `selection`. Frames of another size or
/// channel count than the reference frame are left out. Returns how many frames reached `sink`.
/// Frames go one after the other (the work inside a frame is parallel); `progress` receives the number of
/// finished frames after each frame, and a cancel takes effect within one frame.
#[allow(clippy::too_many_arguments)]
pub fn aligned_frames<E>(
    reference: &Reference,
    reference_frame: &Image,
    params: &StackParams,
    selection: &[u32],
    known: Option<&[FrameAlignment]>,
    mut read: impl FnMut(u32) -> Result<Image, E>,
    mut sink: impl FnMut(Image) -> Result<(), E>,
    mut progress: impl FnMut(u32) -> Control,
) -> Result<u32, SequenceError<E>> {
    if selection.is_empty() {
        return Err(SequenceError::NoFrames);
    }
    let stacker = Stacker::new(reference, reference_frame, params.clone());
    let mut written = 0u32;
    for (done, &index) in selection.iter().enumerate() {
        let frame = read(index).map_err(SequenceError::Read)?;
        let alignment = known
            .and_then(|k| k.get(index as usize))
            .cloned()
            .unwrap_or_else(|| reference.align_frame(&frame));
        if let Some(image) = stacker.warp(&frame, &alignment) {
            sink(image).map_err(SequenceError::Write)?;
            written += 1;
        }
        if progress(done as u32 + 1) == Control::Cancel {
            return Err(SequenceError::Cancelled);
        }
    }
    if written == 0 {
        return Err(SequenceError::NoFrames);
    }
    Ok(written)
}

/// Mean of the last `size` frames, one result per new frame once the window is full. Fewer frames
/// than `size` give one result at [`MovingAverage::finish`]: the mean of what there was.
pub struct MovingAverage {
    size: usize,
    window: VecDeque<Image>,
    sum: Vec<f64>,
    emitted: bool,
}

impl MovingAverage {
    /// `size` is *Average of n Frames*; below one it counts as one.
    pub fn new(size: usize) -> Self {
        Self {
            size: size.max(1),
            window: VecDeque::new(),
            sum: Vec::new(),
            emitted: false,
        }
    }

    /// Adds a frame; the result is the mean of the window once it holds `size` frames. All frames
    /// must have the same size and channel count.
    pub fn push(&mut self, image: Image) -> Option<Image> {
        if self.sum.is_empty() {
            self.sum = vec![0.0; image.data.len()];
        }
        for (total, &v) in self.sum.iter_mut().zip(&image.data) {
            *total += f64::from(v);
        }
        self.window.push_back(image);
        if self.window.len() > self.size
            && let Some(old) = self.window.pop_front()
        {
            for (total, &v) in self.sum.iter_mut().zip(&old.data) {
                *total -= f64::from(v);
            }
        }
        if self.window.len() < self.size {
            return None;
        }
        self.emitted = true;
        Some(self.mean())
    }

    /// The mean of a window that never filled up, if no result has been returned yet.
    pub fn finish(&mut self) -> Option<Image> {
        if self.emitted || self.window.is_empty() {
            return None;
        }
        self.emitted = true;
        Some(self.mean())
    }

    fn mean(&self) -> Image {
        let template = &self.window[0];
        let n = self.window.len() as f64;
        Image {
            width: template.width,
            height: template.height,
            channels: template.channels,
            data: self.sum.iter().map(|&s| (s / n) as f32).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::align::AlignParams;

    fn flat(value: f32) -> Image {
        Image {
            width: 2,
            height: 2,
            channels: 1,
            data: vec![value; 4],
        }
    }

    #[test]
    fn the_window_slides_one_frame_at_a_time() {
        let mut average = MovingAverage::new(2);
        let results: Vec<Option<f32>> = [0.0, 1.0, 0.5, 0.0]
            .into_iter()
            .map(|v| average.push(flat(v)).map(|i| i.data[0]))
            .collect();
        assert_eq!(results, vec![None, Some(0.5), Some(0.75), Some(0.25)]);
        assert!(average.finish().is_none());
    }

    #[test]
    fn fewer_frames_than_the_window_give_their_mean() {
        let mut average = MovingAverage::new(5);
        assert!(average.push(flat(0.2)).is_none());
        assert!(average.push(flat(0.6)).is_none());
        let mean = average.finish().expect("a mean of the two frames");
        assert!((mean.data[0] - 0.4).abs() < 1e-6);
        assert!(average.finish().is_none());
    }

    /// A frame with one bright pixel at `(x, y)`.
    fn dot(x: usize, y: usize) -> Image {
        let mut image = Image::new(24, 24, 1);
        image.data[y * 24 + x] = 1.0;
        image
    }

    fn uniform(shift: (f32, f32)) -> FrameAlignment {
        FrameAlignment {
            shift,
            points: vec![],
            quality: 1.0,
            difference: 0.0,
        }
    }

    fn bright_pixel(image: &Image) -> (usize, usize) {
        let at = image
            .data
            .iter()
            .position(|&v| v > 0.5)
            .expect("a bright pixel");
        (at % 24, at / 24)
    }

    #[test]
    fn frames_come_out_moved_onto_the_reference_in_the_order_asked_for() {
        let base = dot(10, 10);
        let reference = Reference::new(&base, &[], &AlignParams::default());
        // `frame(p + shift)` shows what the reference shows at `p`: the dot moved by -shift.
        let frames = [dot(12, 9), dot(10, 10), dot(7, 13)];
        let alignments = [
            uniform((2.0, -1.0)),
            uniform((0.0, 0.0)),
            uniform((-3.0, 3.0)),
        ];
        let params = StackParams {
            correct_geometry: false,
            ..StackParams::default()
        };
        let mut seen = Vec::new();
        let mut reads = Vec::new();
        let count = aligned_frames::<()>(
            &reference,
            &base,
            &params,
            &[2, 0, 1],
            Some(&alignments),
            |i| {
                reads.push(i);
                Ok(frames[i as usize].clone())
            },
            |image| {
                seen.push(bright_pixel(&image));
                Ok(())
            },
            |_| Control::Continue,
        )
        .unwrap();
        assert_eq!(count, 3);
        assert_eq!(reads, vec![2, 0, 1]);
        assert_eq!(seen, vec![(10, 10); 3]);
    }

    #[test]
    fn an_empty_selection_is_an_error_and_cancel_stops() {
        let base = dot(10, 10);
        let reference = Reference::new(&base, &[], &AlignParams::default());
        let params = StackParams::default();
        let none = aligned_frames::<()>(
            &reference,
            &base,
            &params,
            &[],
            None,
            |_| Ok(base.clone()),
            |_| Ok(()),
            |_| Control::Continue,
        );
        assert_eq!(none, Err(SequenceError::NoFrames));
        let cancelled = aligned_frames::<()>(
            &reference,
            &base,
            &params,
            &[0, 0],
            Some(&[uniform((0.0, 0.0))]),
            |_| Ok(base.clone()),
            |_| Ok(()),
            |_| Control::Cancel,
        );
        assert_eq!(cancelled, Err(SequenceError::Cancelled));
    }

    #[test]
    fn progress_comes_after_every_frame_and_a_cancel_stops_at_once() {
        let base = dot(10, 10);
        let reference = Reference::new(&base, &[], &AlignParams::default());
        let params = StackParams::default();
        let alignments = vec![uniform((0.0, 0.0)); 6];
        let selection = [0, 1, 2, 3, 4, 5];
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .unwrap();
        let mut reported = Vec::new();
        pool.install(|| {
            aligned_frames::<()>(
                &reference,
                &base,
                &params,
                &selection,
                Some(&alignments),
                |_| Ok(base.clone()),
                |_| Ok(()),
                |done| {
                    reported.push(done);
                    Control::Continue
                },
            )
            .unwrap()
        });
        assert_eq!(reported, vec![1, 2, 3, 4, 5, 6]);
        let mut reads = 0;
        let cancelled = pool.install(|| {
            aligned_frames::<()>(
                &reference,
                &base,
                &params,
                &selection,
                Some(&alignments),
                |_| {
                    reads += 1;
                    Ok(base.clone())
                },
                |_| Ok(()),
                |_| Control::Cancel,
            )
        });
        assert_eq!(cancelled, Err(SequenceError::Cancelled));
        assert_eq!(reads, 1);
    }

    #[test]
    fn a_failing_sink_is_reported_as_a_write_error() {
        let base = dot(10, 10);
        let reference = Reference::new(&base, &[], &AlignParams::default());
        let result = aligned_frames::<&str>(
            &reference,
            &base,
            &StackParams::default(),
            &[0],
            Some(&[uniform((0.0, 0.0))]),
            |_| Ok(base.clone()),
            |_| Err("disk full"),
            |_| Control::Continue,
        );
        assert_eq!(result, Err(SequenceError::Write("disk full")));
    }
}
