//! The Align tab: Set Alignpoints, Align and Limit (`docs/workflow.md`, section 2).

use crate::{
    Session, SessionError,
    error::lock,
    progress::{JobControl, Operation, ProgressSink},
};
use planets_core::align::{
    AlignError, AlignParams, AlignPoint, FrameAlignment, IntensityMode, LimitMode, PointParams,
    Reference, Roi, align_sequence, apply_threshold, default_threshold, find_alignpoints,
    limit_frames, rank_by_quality,
};
use serde::{Deserialize, Serialize};
use std::{sync::Mutex, time::Instant};

/// What the Align tab has computed so far.
#[derive(Default)]
pub(crate) struct AlignState {
    /// All points that satisfy the spacing rules, strongest first.
    candidates: Mutex<Vec<AlignPoint>>,
    /// How many of the weakest candidates are dropped (the *Threshold* slider).
    threshold: Mutex<usize>,
    /// Index of the frame the points were set on; it is the reference for *Align*.
    reference_frame: Mutex<u32>,
    results: Mutex<Vec<FrameAlignment>>,
    pub(crate) control: JobControl,
}

impl AlignState {
    /// The registration of all frames from the last *Align*, if it fits a request with
    /// `point_count` alignpoints on a recording of `frame_count` frames. Anything else (no *Align*
    /// yet, points checked or unchecked since) gives `None`, and the caller registers again.
    pub(crate) fn results_for(
        &self,
        point_count: usize,
        frame_count: u32,
    ) -> Option<Vec<FrameAlignment>> {
        let results = self.results.lock().ok()?;
        matching(&results, point_count, frame_count).then(|| results.clone())
    }
}

fn matching(results: &[FrameAlignment], point_count: usize, frame_count: u32) -> bool {
    results.len() == frame_count as usize && results.iter().all(|r| r.points.len() == point_count)
}

/// *R.o.Interest*: the rectangle that alignment is based on, in frame pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct RoiSettings {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl From<RoiSettings> for Roi {
    fn from(r: RoiSettings) -> Self {
        Roi {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

/// How the pixel value of a candidate point is tested against *Lo* and *Hi*. Anything but
/// `area3x3` and `lowest` means `default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Intensity {
    Area3x3,
    Lowest,
    #[serde(other)]
    Default,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointSettings {
    pub min_distance: u32,
    pub edge_distance: u32,
    /// *Max Alignpoint movement*; it counts towards the spacing of the points.
    pub max_movement: u32,
    /// Look for points inside this region only.
    pub roi: Option<RoiSettings>,
    pub intensity: Intensity,
    pub lo: f32,
    pub hi: f32,
    pub box_size: u32,
}

impl PointSettings {
    fn params(&self) -> PointParams {
        PointParams {
            min_distance: self.min_distance,
            max_movement: self.max_movement,
            roi: self.roi.map(Roi::from),
            edge_distance: self.edge_distance,
            intensity: match self.intensity {
                Intensity::Area3x3 => IntensityMode::Area3x3,
                Intensity::Lowest => IntensityMode::LowestPixel,
                Intensity::Default => IntensityMode::Default,
            },
            lo: self.lo,
            hi: self.hi,
            box_size: self.box_size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alignpoints {
    /// The points that are in use after the threshold.
    pub points: Vec<Point>,
    /// Number of points before the threshold, the upper end of the slider.
    pub total: usize,
    pub threshold: usize,
}

fn kept(state: &AlignState) -> Result<Alignpoints, SessionError> {
    let candidates = lock(&state.candidates)?;
    let threshold = *lock(&state.threshold)?;
    Ok(Alignpoints {
        points: apply_threshold(&candidates, threshold)
            .iter()
            .map(|p| Point { x: p.x, y: p.y })
            .collect(),
        total: candidates.len(),
        threshold,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameRating {
    pub index: u32,
    pub quality: f32,
    pub shift_x: f32,
    pub shift_y: f32,
    pub difference: f32,
}

/// Result of *Align*: one rating per frame in frame order, and the frames from best to worst.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Aligned {
    pub frames: Vec<FrameRating>,
    pub ranking: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignSettings {
    pub box_size: u32,
    pub max_movement: u32,
    /// Measure the drift between frames inside this region only; use the one the points were set in.
    pub roi: Option<RoiSettings>,
    /// *Align by center of gravity*: its *Brightness threshold* (0..=255), absent means off.
    #[serde(default)]
    pub centre_of_gravity: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", tag = "mode", content = "value")]
pub enum LimitSettings {
    LowestQuality(f32),
    BestPercent(f32),
    BestCount(usize),
    FramesPerPoint(usize),
}

/// Result of *Limit*.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limited {
    /// The frame indices that stay, best first.
    pub frames: Vec<u32>,
    /// For every point in use, how far it sits from where the movement of the whole frame puts it, on
    /// average over the frames that stay: the green strokes in the image. `None` if it was never found.
    /// Which shift the original draws is unobserved; this is the local distortion, a few pixels.
    pub point_shifts: Vec<Option<[f32; 2]>>,
}

impl Session {
    /// Sets alignpoints on one frame, with the default threshold. New points start a new alignment:
    /// the old results and everything built on them are gone.
    pub fn set_alignpoints(
        &self,
        frame: u32,
        settings: &PointSettings,
    ) -> Result<Alignpoints, SessionError> {
        let image = self.read(frame)?;
        let params = settings.params();
        let candidates = self.compute.install(|| find_alignpoints(&image, &params));
        let state = &self.align;
        *lock(&state.threshold)? = default_threshold(candidates.len());
        *lock(&state.candidates)? = candidates;
        *lock(&state.reference_frame)? = frame;
        lock(&state.results)?.clear();
        self.discard_results();
        kept(state)
    }

    /// Moves the *Threshold* slider.
    pub fn set_alignpoint_threshold(&self, threshold: usize) -> Result<Alignpoints, SessionError> {
        *lock(&self.align.threshold)? = threshold;
        kept(&self.align)
    }

    /// Aligns all frames to the reference frame and rates them. Reports progress to `progress` and stops on
    /// [`Session::cancel`] with [`Operation::Align`]. Aligning again replaces the alignment, so what was built
    /// on the old one goes at the start: a cancelled run leaves no stack or sharpened image that no longer
    /// matches.
    pub fn align(
        &self,
        settings: &AlignSettings,
        progress: &dyn ProgressSink,
    ) -> Result<Aligned, SessionError> {
        let state = &self.align;
        let points: Vec<AlignPoint> = {
            let candidates = lock(&state.candidates)?;
            let threshold = *lock(&state.threshold)?;
            apply_threshold(&candidates, threshold).to_vec()
        };
        if points.is_empty() {
            return Err(SessionError::NoAlignpoints);
        }
        let reference_frame = *lock(&state.reference_frame)?;
        lock(&state.results)?.clear();
        self.discard_results();
        let total = self.frame_count()?;
        let reference = Reference::with_roi(
            &self.read(reference_frame)?,
            &points,
            &AlignParams {
                box_size: settings.box_size,
                max_movement: settings.max_movement,
                centre_of_gravity: settings.centre_of_gravity,
            },
            settings.roi.map(Roi::from),
        );

        state.control.begin();
        let started = Instant::now();
        let results = self
            .compute
            .install(|| {
                align_sequence(
                    &reference,
                    total,
                    |index| self.read(index),
                    |done| {
                        state.control.wait_while_paused();
                        progress.progress(
                            Operation::Align,
                            done,
                            total,
                            started.elapsed().as_millis() as u64,
                        );
                        state.control.control()
                    },
                )
            })
            .map_err(|e| match e {
                AlignError::Read(error) => error,
                AlignError::Cancelled => SessionError::Cancelled,
            })?;

        let aligned = Aligned {
            frames: results
                .iter()
                .enumerate()
                .map(|(i, r)| FrameRating {
                    index: i as u32,
                    quality: r.quality,
                    shift_x: r.shift.0,
                    shift_y: r.shift.1,
                    difference: r.difference,
                })
                .collect(),
            ranking: rank_by_quality(&results)
                .into_iter()
                .map(|i| i as u32)
                .collect(),
        };
        *lock(&state.results)? = results;
        Ok(aligned)
    }

    /// *Limit*: the frames that stay and the mean shift of every point over them. Another selection of frames
    /// makes an earlier stack obsolete.
    pub fn limit(&self, settings: &LimitSettings) -> Result<Limited, SessionError> {
        let results = lock(&self.align.results)?;
        if results.is_empty() {
            return Err(SessionError::NotAligned);
        }
        let mode = match *settings {
            LimitSettings::LowestQuality(p) => LimitMode::LowestQuality(p),
            LimitSettings::BestPercent(p) => LimitMode::BestPercent(p),
            LimitSettings::BestCount(n) => LimitMode::BestCount(n),
            LimitSettings::FramesPerPoint(n) => LimitMode::FramesPerPoint(n),
        };
        let kept = limit_frames(&results, mode);
        let point_count = results.iter().map(|r| r.points.len()).max().unwrap_or(0);
        let point_shifts = (0..point_count)
            .map(|point| {
                let (mut sum, mut n) = ([0.0f32; 2], 0u32);
                for &frame in &kept {
                    let result = &results[frame];
                    if let Some(Some(m)) = result.points.get(point) {
                        sum[0] += m.dx - result.shift.0;
                        sum[1] += m.dy - result.shift.1;
                        n += 1;
                    }
                }
                (n > 0).then(|| [sum[0] / n as f32, sum[1] / n as f32])
            })
            .collect();
        self.discard_results();
        Ok(Limited {
            frames: kept.into_iter().map(|i| i as u32).collect(),
            point_shifts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rated(points: usize) -> FrameAlignment {
        FrameAlignment {
            shift: (0.0, 0.0),
            points: vec![None; points],
            quality: 1.0,
            difference: 0.0,
        }
    }

    #[test]
    fn results_are_only_reused_for_the_same_frames_and_points() {
        let results = vec![rated(3), rated(3)];
        assert!(matching(&results, 3, 2));
        // Another number of alignpoints than at Align: points were checked or unchecked since.
        assert!(!matching(&results, 2, 2));
        // Another recording length.
        assert!(!matching(&results, 3, 5));
        // No Align yet.
        assert!(!matching(&[], 3, 2));
    }
}
