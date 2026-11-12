//! The Stack tab: Stack and the stacked image (`docs/workflow.md`, section 3).
//!
//! The stack is kept as an [`Image`] in [`StackState`], so that the Wavelet tab and the export can
//! take it from there without going through the UI.

use crate::{
    Session, SessionError,
    align::RoiSettings,
    derotate::DerotationSettings,
    error::lock,
    image::RgbaImage,
    progress::{JobControl, Operation, ProgressSink},
};
use planets_core::{
    Image,
    align::{AlignParams, Reference, Roi},
    stack::{Derotation, StackError, StackParams, points_from_positions, stack_sequence_derotated},
};
use serde::{Deserialize, Serialize};
use std::{sync::Mutex, time::Instant};

/// What the Stack tab has computed. Its Cancel and Pause flags are also those of *Save Aligned* and
/// *Save Integrated*.
#[derive(Default)]
pub(crate) struct StackState {
    result: Mutex<Option<Image>>,
    pub(crate) control: JobControl,
}

impl StackState {
    /// A copy of the last stacked image, `None` before the first *Stack*.
    pub(crate) fn image(&self) -> Option<Image> {
        self.result.lock().ok().and_then(|guard| guard.clone())
    }

    /// Forgets the stacked image: it was made from an alignment that is gone.
    pub(crate) fn clear(&self) {
        if let Ok(mut guard) = self.result.lock() {
            *guard = None;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PointPosition {
    pub x: u32,
    pub y: u32,
}

/// Everything *Stack* needs from the Align tab and from *Stacking options* (`docs/ui.md`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackSettings {
    /// The frame the alignpoints were set on.
    pub reference_frame: u32,
    /// The alignpoints in use (checked in the *Alignmentpoints* list).
    pub points: Vec<PointPosition>,
    /// The frames that stay after *Limit* (checked in the frame list).
    pub frames: Vec<u32>,
    /// *Alignmentbox size* and *Max Alignpoint movement* of the Align tab.
    pub box_size: u32,
    pub max_movement: u32,
    /// *Use Nearestby Alignpoints*.
    pub nearest_points: usize,
    /// *Correct geometry*.
    pub correct_geometry: bool,
    /// *Normalization of frame intensity*.
    pub normalize: bool,
    /// *Do not stack n pixels at the frameedge*.
    pub edge_skip: u32,
    /// The *R.o.Interest* the alignpoints were set in, if any.
    pub roi: Option<RoiSettings>,
    /// *De-rotate images*: turn the frames back to a common time before they are stacked. Absent means off.
    #[serde(default)]
    pub derotation: Option<DerotationSettings>,
    /// *Align by center of gravity* of the Align step: its *Brightness threshold* (0..=255), absent means off.
    #[serde(default)]
    pub centre_of_gravity: Option<f32>,
}

impl StackSettings {
    /// The reference for registering frames: the alignpoints of the request on `reference_frame`.
    pub(crate) fn reference(&self, reference_frame: &Image) -> Reference {
        let points = points_from_positions(self.points.iter().map(|p| (p.x, p.y)));
        Reference::with_roi(
            reference_frame,
            &points,
            &AlignParams {
                box_size: self.box_size,
                max_movement: self.max_movement,
                centre_of_gravity: self.centre_of_gravity,
            },
            self.roi.map(Roi::from),
        )
    }

    /// The *Stacking options* of the request.
    pub(crate) fn params(&self) -> StackParams {
        StackParams {
            nearest_points: self.nearest_points,
            correct_geometry: self.correct_geometry,
            normalize: self.normalize,
            edge_skip: self.edge_skip,
        }
    }
}

/// What the UI shows after *Stack*: "Stack size (n=10) - (1024x680)".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stacked {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    /// How many frames went into the stack.
    pub frames: u32,
    pub elapsed_ms: u64,
}

impl Session {
    /// Registers the selected frames against the reference frame and stacks them. Reports progress to
    /// `progress` and stops on [`Session::cancel`] with [`Operation::Stack`]. The stack becomes the image
    /// of the Wavelet tab and the one *Save image* writes in the Stack tab.
    pub fn stack(
        &self,
        settings: &StackSettings,
        progress: &dyn ProgressSink,
    ) -> Result<Stacked, SessionError> {
        if settings.points.is_empty() {
            return Err(SessionError::NoAlignpoints);
        }
        let reference_frame = self.read(settings.reference_frame)?;
        // With de-rotation the reference frame is turned to the middle of the recording too, so that the alignpoints see the same
        // features in every frame.
        let plan = match &settings.derotation {
            Some(derotation) => Some(self.derotation_for_stack(
                derotation,
                settings.reference_frame,
                &settings.frames,
                &reference_frame,
            )?),
            None => None,
        };
        let reference_frame = plan
            .as_ref()
            .map_or(reference_frame, |(_, turned)| turned.clone());
        let reference = settings.reference(&reference_frame);
        let params = settings.params();
        let derotation = plan.as_ref().map(|(plan, _)| Derotation {
            derotator: &plan.derotator,
            offsets: &plan.offsets,
        });

        self.stack.control.begin();
        let total = settings.frames.len() as u32;
        let started = Instant::now();
        let stacked = self
            .compute
            .install(|| {
                stack_sequence_derotated(
                    &reference,
                    &reference_frame,
                    &params,
                    derotation.as_ref(),
                    &settings.frames,
                    |index| self.read(index),
                    |done| {
                        let control = self.stack.control.checkpoint();
                        progress.progress(
                            Operation::Stack,
                            done,
                            total,
                            started.elapsed().as_millis() as u64,
                        );
                        control
                    },
                )
            })
            .map_err(|e| match e {
                StackError::NoFrames => SessionError::NoFramesSelected,
                StackError::Read(error) => error,
                StackError::Cancelled => SessionError::Cancelled,
            })?;

        let summary = Stacked {
            width: stacked.image.width,
            height: stacked.image.height,
            channels: stacked.image.channels,
            frames: stacked.frames,
            elapsed_ms: started.elapsed().as_millis() as u64,
        };
        // The Wavelet tab sharpens the stacked image and *Save image* writes it.
        self.wavelet.set_source(stacked.image.clone())?;
        self.export.set_stack(stacked.image.clone());
        *lock(&self.stack.result)? = Some(stacked.image);
        Ok(summary)
    }

    /// The stacked image.
    pub fn stacked_image(&self) -> Result<RgbaImage, SessionError> {
        let image = self.stack.image().ok_or(SessionError::NotStacked)?;
        Ok(RgbaImage::from_image(&image))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearing_forgets_the_stacked_image() {
        let state = StackState::default();
        *state.result.lock().unwrap() = Some(Image::new(2, 2, 3));
        assert!(state.image().is_some());
        state.clear();
        assert!(state.image().is_none());
    }
}
