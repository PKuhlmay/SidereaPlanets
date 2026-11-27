//! De-rotation for the Stack step: which planet, when each frame was taken, and the numbers the Stack panel shows
//! (span of the recording, how far the planet turns, the measured disc).

use crate::{Session, SessionError, error::lock};
use planets_core::{
    Image,
    derotate::{Derotator, DiscShape, View},
    planet::Planet,
};
use serde::{Deserialize, Serialize};

fn yes() -> bool {
    true
}

/// *De-rotate images* in the *Stacking options*: absent from a request means off.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerotationSettings {
    pub planet: Planet,
    /// How the camera sat, degrees counter-clockwise on the screen; 0 means celestial north is up.
    #[serde(default)]
    pub camera_angle_deg: f64,
    /// The image is a mirror image (east to the right with north up).
    #[serde(default)]
    pub mirrored: bool,
    /// When the first frame was taken, seconds since 1970-01-01 00:00:00 UTC. Only needed if the frames carry no time stamps of
    /// their own (or [`DerotationSettings::use_file_timestamps`] is off).
    #[serde(default)]
    pub start_time_utc: Option<f64>,
    /// Use the time stamps of the file (SER trailer) when it has them.
    #[serde(default = "yes")]
    pub use_file_timestamps: bool,
}

/// What the Stack panel says about de-rotation, before and without stacking.
#[derive(Debug, Clone, PartialEq)]
pub struct DerotationInfo {
    /// Seconds from the first to the last of the selected frames.
    pub span_seconds: f64,
    /// How far the planet turns in that time, degrees.
    pub rotation_degrees: f64,
    /// `D_E` and `P` of the axis at the reference time, degrees.
    pub sub_earth_latitude: f64,
    pub pole_angle: f64,
    /// The planet as measured in the reference frame, for the overlay.
    pub disc: Option<DiscShape>,
    /// Why de-rotation cannot work (no start time, no planet in the frame), in the words the window shows.
    pub problem: Option<String>,
}

/// Everything *Stack* needs to turn the frames: the measured planet and the time of every frame.
pub(crate) struct Plan {
    pub(crate) derotator: Derotator,
    /// Time of each selected frame minus the reference time, seconds, in the order of the selection.
    pub(crate) offsets: Vec<f64>,
    /// The same for the reference frame the alignpoints were set on.
    pub(crate) reference_offset: f64,
    span_seconds: f64,
}

impl Session {
    /// The time of the frames `indices` in Unix seconds: from the time stamps of the file if it has them and the settings say so,
    /// else the start time plus the index over the frame rate.
    fn frame_times(
        &self,
        settings: &DerotationSettings,
        indices: &[u32],
    ) -> Result<Vec<f64>, SessionError> {
        let guard = lock(&self.recording)?;
        let recording = guard.as_ref().ok_or(SessionError::NoRecording)?;
        if let (true, Some(times)) = (settings.use_file_timestamps, recording.frame_timestamps()) {
            return indices
                .iter()
                .map(|&i| {
                    times
                        .get(i as usize)
                        .copied()
                        .ok_or_else(|| SessionError::failed(format!("frame {i} has no time stamp")))
                })
                .collect();
        }
        let start = settings.start_time_utc.ok_or_else(|| {
            SessionError::failed("De-rotation needs the start time of the recording")
        })?;
        let fps = recording.info().fps.ok_or_else(|| {
            SessionError::failed(
                "De-rotation needs time stamps or a frame rate, and this recording has neither",
            )
        })?;
        Ok(indices
            .iter()
            .map(|&i| start + f64::from(i) / fps)
            .collect())
    }

    /// Measures the planet in `reference` (read from `reference_frame`) for the time in the middle of `frames`.
    fn derotation_plan(
        &self,
        settings: &DerotationSettings,
        reference_frame: u32,
        frames: &[u32],
        reference: &Image,
    ) -> Result<(Plan, View), SessionError> {
        if frames.is_empty() {
            return Err(SessionError::NoFramesSelected);
        }
        let mut wanted = frames.to_vec();
        wanted.push(reference_frame);
        let times = self.frame_times(settings, &wanted)?;
        let (selected, reference_time_of_frame) = times.split_at(frames.len());
        let first = selected.iter().copied().fold(f64::MAX, f64::min);
        let last = selected.iter().copied().fold(f64::MIN, f64::max);
        let reference_time = (first + last) / 2.0;

        let planet = settings.planet;
        let geometry = planet.pole_geometry(reference_time);
        let view = View {
            planet,
            sub_earth_latitude: geometry.sub_earth_latitude,
            pole_angle: geometry.pole_angle,
            camera_angle: settings.camera_angle_deg,
            mirrored: settings.mirrored,
        };
        let derotator = Derotator::new(view, reference).ok_or_else(|| {
            SessionError::failed("De-rotation could not find the planet in the reference frame")
        })?;
        Ok((
            Plan {
                derotator,
                offsets: selected.iter().map(|t| t - reference_time).collect(),
                reference_offset: reference_time_of_frame[0] - reference_time,
                span_seconds: last - first,
            },
            view,
        ))
    }

    /// What the Stack panel shows for the frames that stay after *Limit*: the span, how far the planet turns, and the disc measured in
    /// the reference frame. A problem the user can fix (no start time) is part of the answer, not an error.
    pub fn derotation_info(
        &self,
        settings: &DerotationSettings,
        reference_frame: u32,
        frames: &[u32],
    ) -> Result<DerotationInfo, SessionError> {
        let reference = self.read(reference_frame)?;
        match self.derotation_plan(settings, reference_frame, frames, &reference) {
            Ok((plan, view)) => Ok(DerotationInfo {
                span_seconds: plan.span_seconds,
                rotation_degrees: settings.planet.rotation_degrees(plan.span_seconds),
                sub_earth_latitude: view.sub_earth_latitude,
                pole_angle: view.pole_angle,
                disc: Some(view.shape(&plan.derotator.disc())),
                problem: None,
            }),
            Err(SessionError::Failed(problem)) => Ok(DerotationInfo {
                span_seconds: 0.0,
                rotation_degrees: 0.0,
                sub_earth_latitude: 0.0,
                pole_angle: 0.0,
                disc: None,
                problem: Some(problem),
            }),
            Err(error) => Err(error),
        }
    }

    /// The plan for stacking and the reference frame turned to the reference time.
    pub(crate) fn derotation_for_stack(
        &self,
        settings: &DerotationSettings,
        reference_frame: u32,
        frames: &[u32],
        reference: &Image,
    ) -> Result<(Plan, Image), SessionError> {
        let (plan, _) = self.derotation_plan(settings, reference_frame, frames, reference)?;
        let turned = plan.derotator.derotate(reference, plan.reference_offset);
        Ok((plan, turned))
    }
}
