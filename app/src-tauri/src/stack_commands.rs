//! Commands for the Stack tab: Stack and fetching the stacked image (`docs/workflow.md`, section 3).

use crate::{CommandResult, Events, pack};
use planets_session::{
    DerotationInfo, DerotationSettings, DiscShape, Operation, Session, StackSettings, Stacked,
};
use serde::Serialize;
use tauri::{AppHandle, State, ipc::Response};

/// The measured disc for the overlay of the Stack step, in image pixels (see [`DiscShape`]).
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiscView {
    x: f64,
    y: f64,
    semi_major: f64,
    semi_minor: f64,
    /// Degrees clockwise on the screen from pointing right.
    angle: f64,
    /// Outer edge of Saturn's ring as `[semiMajor, semiMinor]`.
    ring: Option<[f64; 2]>,
    /// Unit vector towards the north pole, `[x, y]` with y down.
    north: [f64; 2],
}

impl From<DiscShape> for DiscView {
    fn from(disc: DiscShape) -> Self {
        Self {
            x: disc.x,
            y: disc.y,
            semi_major: disc.semi_major,
            semi_minor: disc.semi_minor,
            angle: disc.angle,
            ring: disc.ring.map(|(major, minor)| [major, minor]),
            north: [disc.north.0, disc.north.1],
        }
    }
}

/// What *De-rotate images* shows in the Stack panel: span, rotation, the measured disc, or the problem in words.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DerotationView {
    span_seconds: f64,
    rotation_degrees: f64,
    disc: Option<DiscView>,
    problem: Option<String>,
}

impl From<DerotationInfo> for DerotationView {
    fn from(info: DerotationInfo) -> Self {
        Self {
            span_seconds: info.span_seconds,
            rotation_degrees: info.rotation_degrees,
            disc: info.disc.map(DiscView::from),
            problem: info.problem,
        }
    }
}

/// Span, rotation and measured disc for *De-rotate images*, for the kept `frames` and the frame the points were set on.
#[tauri::command]
pub async fn derotation_info(
    settings: DerotationSettings,
    reference_frame: u32,
    frames: Vec<u32>,
    session: State<'_, Session>,
) -> CommandResult<DerotationView> {
    Ok(session
        .derotation_info(&settings, reference_frame, &frames)?
        .into())
}

/// Registers the selected frames against the reference frame and stacks them. Reports progress
/// through the `stack-progress` event and stops on [`cancel_stacking`].
#[tauri::command]
pub async fn stack(
    settings: StackSettings,
    app: AppHandle,
    session: State<'_, Session>,
) -> CommandResult<Stacked> {
    Ok(session.stack(&settings, &Events(app))?)
}

/// Returns the stacked image as binary, like `read_frame`.
#[tauri::command]
pub fn stacked_image(session: State<'_, Session>) -> CommandResult<Response> {
    Ok(pack(session.stacked_image()?))
}

/// Stops a running *Stack* (and *Save Aligned* / *Save Integrated*, which share its flags).
#[tauri::command]
pub fn cancel_stacking(session: State<'_, Session>) {
    session.cancel(Operation::Stack);
}

/// Pauses or resumes a running *Stack*.
#[tauri::command]
pub fn pause_stacking(paused: bool, session: State<'_, Session>) {
    session.pause(Operation::Stack, paused);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_derotation_info_reaches_the_window_in_its_shape() {
        let info = DerotationInfo {
            span_seconds: 192.0,
            rotation_degrees: 5.8,
            sub_earth_latitude: 1.0,
            pole_angle: 2.0,
            disc: Some(DiscShape {
                x: 100.0,
                y: 80.0,
                semi_major: 40.0,
                semi_minor: 37.5,
                angle: -12.0,
                ring: Some((90.0, 20.0)),
                north: (0.2, -0.98),
            }),
            problem: None,
        };
        let json = serde_json::to_value(DerotationView::from(info)).expect("serialisable");
        assert_eq!(
            json,
            serde_json::json!({
                "spanSeconds": 192.0,
                "rotationDegrees": 5.8,
                "disc": {
                    "x": 100.0, "y": 80.0, "semiMajor": 40.0, "semiMinor": 37.5, "angle": -12.0,
                    "ring": [90.0, 20.0], "north": [0.2, -0.98]
                },
                "problem": null
            })
        );
    }
}
