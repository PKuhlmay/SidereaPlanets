//! Commands for the Stack tab: Stack and fetching the stacked image (`docs/workflow.md`, section 3).
//!
//! The stack is kept as an [`Image`] in [`StackState`], so that the Wavelet tab and the export can
//! take it from there without going through the UI.

use crate::{
    AppState, align_commands::RoiSettings, export_commands::ExportState,
    wavelet_commands::WaveletState,
};
use planets_core::{
    Image,
    align::{AlignParams, Control, Reference, Roi},
    stack::{StackError, StackParams, points_from_positions, stack_sequence},
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, State, ipc::Response};

/// What the Stack tab has computed.
#[derive(Default)]
pub struct StackState {
    result: Mutex<Option<Image>>,
    cancelled: AtomicBool,
    paused: AtomicBool,
}

impl StackState {
    /// A copy of the last stacked image, `None` before the first *Stack*.
    pub fn image(&self) -> Option<Image> {
        self.result.lock().ok().and_then(|guard| guard.clone())
    }
}

#[derive(Deserialize)]
pub struct PointPosition {
    x: u32,
    y: u32,
}

/// Everything *Stack* needs from the Align tab and from *Stacking options* (`docs/ui.md`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackSettings {
    /// The frame the alignpoints were set on.
    reference_frame: u32,
    /// The alignpoints in use (checked in the *Alignmentpoints* list).
    points: Vec<PointPosition>,
    /// The frames that stay after *Limit* (checked in the frame list).
    frames: Vec<u32>,
    /// *Alignmentbox size* and *Max Alignpoint movement* of the Align tab.
    box_size: u32,
    max_movement: u32,
    /// *Use Nearestby Alignpoints*.
    nearest_points: usize,
    /// *Correct geometry*.
    correct_geometry: bool,
    /// *Normalization of frame intensity*.
    normalize: bool,
    /// *Do not stack n pixels at the frameedge*.
    edge_skip: u32,
    /// The *R.o.Interest* the alignpoints were set in, if any.
    roi: Option<RoiSettings>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    done: u32,
    total: u32,
    elapsed_ms: u64,
}

/// What the UI shows after *Stack*: "Stack size (n=10) - (1024x680)".
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stacked {
    width: u32,
    height: u32,
    channels: u8,
    /// How many frames went into the stack.
    frames: u32,
    elapsed_ms: u64,
}

/// Registers the selected frames against the reference frame and stacks them. Reports progress
/// through the `stack-progress` event and stops on [`cancel_stacking`].
#[tauri::command]
pub async fn stack(
    settings: StackSettings,
    app: AppHandle,
    recording: State<'_, AppState>,
    state: State<'_, StackState>,
) -> Result<Stacked, String> {
    if settings.points.is_empty() {
        return Err("set alignpoints first".into());
    }
    let read = |index: u32| -> Result<Image, String> {
        let mut guard = recording.0.lock().map_err(|e| e.to_string())?;
        let recording = guard.as_mut().ok_or("no recording is open")?;
        recording.read_frame(index).map_err(|e| e.to_string())
    };
    let reference_frame = read(settings.reference_frame)?;
    let points = points_from_positions(settings.points.iter().map(|p| (p.x, p.y)));
    let reference = Reference::with_roi(
        &reference_frame,
        &points,
        &AlignParams {
            box_size: settings.box_size,
            max_movement: settings.max_movement,
        },
        settings.roi.map(Roi::from),
    );
    let params = StackParams {
        nearest_points: settings.nearest_points,
        correct_geometry: settings.correct_geometry,
        normalize: settings.normalize,
        edge_skip: settings.edge_skip,
    };

    state.cancelled.store(false, Ordering::Relaxed);
    state.paused.store(false, Ordering::Relaxed);
    let total = settings.frames.len() as u32;
    let started = Instant::now();
    let stacked = stack_sequence(
        &reference,
        &reference_frame,
        &params,
        &settings.frames,
        read,
        |done| {
            while state.paused.load(Ordering::Relaxed) && !state.cancelled.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(50));
            }
            let _ = app.emit(
                "stack-progress",
                Progress {
                    done,
                    total,
                    elapsed_ms: started.elapsed().as_millis() as u64,
                },
            );
            if state.cancelled.load(Ordering::Relaxed) {
                Control::Cancel
            } else {
                Control::Continue
            }
        },
    )
    .map_err(|e| match e {
        StackError::NoFrames => "no frames selected".to_owned(),
        StackError::Read(message) => message,
        StackError::Cancelled => "cancelled".to_owned(),
    })?;

    let summary = Stacked {
        width: stacked.image.width,
        height: stacked.image.height,
        channels: stacked.image.channels,
        frames: stacked.frames,
        elapsed_ms: started.elapsed().as_millis() as u64,
    };
    // The Wavelet tab sharpens the stacked image and *Save image* writes it.
    app.state::<WaveletState>()
        .set_source(stacked.image.clone())?;
    app.state::<ExportState>().set_stack(stacked.image.clone());
    *state.result.lock().map_err(|e| e.to_string())? = Some(stacked.image);
    Ok(summary)
}

/// Returns the stacked image as binary, like `read_frame`: width (u32 LE), height (u32 LE), then
/// width * height * 4 bytes RGBA.
#[tauri::command]
pub fn stacked_image(state: State<'_, StackState>) -> Result<Response, String> {
    let image = state.image().ok_or("stack first")?;
    let mut out = Vec::with_capacity(8 + image.width as usize * image.height as usize * 4);
    out.extend(image.width.to_le_bytes());
    out.extend(image.height.to_le_bytes());
    out.extend(image.to_rgba8());
    Ok(Response::new(out))
}

/// Stops a running *Stack*.
#[tauri::command]
pub fn cancel_stacking(state: State<'_, StackState>) {
    state.cancelled.store(true, Ordering::Relaxed);
}

/// Pauses or resumes a running *Stack*.
#[tauri::command]
pub fn pause_stacking(paused: bool, state: State<'_, StackState>) {
    state.paused.store(paused, Ordering::Relaxed);
}
