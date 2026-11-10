//! Commands for the Align tab: Set Alignpoints, Align and Limit (`docs/workflow.md`, section 2).

use crate::AppState;
use planets_core::align::{
    AlignError, AlignParams, AlignPoint, Control, FrameAlignment, IntensityMode, LimitMode,
    PointParams, Reference, Roi, align_sequence, apply_threshold, default_threshold,
    find_alignpoints, limit_frames, rank_by_quality,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, State};

/// What the Align tab has computed so far.
#[derive(Default)]
pub struct AlignState {
    /// All points that satisfy the spacing rules, strongest first.
    candidates: Mutex<Vec<AlignPoint>>,
    /// How many of the weakest candidates are dropped (the *Threshold* slider).
    threshold: Mutex<usize>,
    /// Index of the frame the points were set on; it is the reference for *Align*.
    reference_frame: Mutex<u32>,
    results: Mutex<Vec<FrameAlignment>>,
    cancelled: AtomicBool,
    paused: AtomicBool,
}

/// *R.o.Interest*: the rectangle that alignment is based on, in frame pixels.
#[derive(Deserialize)]
pub struct RoiSettings {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointSettings {
    min_distance: u32,
    edge_distance: u32,
    /// *Max Alignpoint movement*; it counts towards the spacing of the points.
    max_movement: u32,
    /// Look for points inside this region only.
    roi: Option<RoiSettings>,
    /// `default`, `area3x3` or `lowest`.
    intensity: String,
    lo: f32,
    hi: f32,
    box_size: u32,
}

#[derive(Serialize)]
pub struct Point {
    x: u32,
    y: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Alignpoints {
    /// The points that are in use after the threshold.
    points: Vec<Point>,
    /// Number of points before the threshold, the upper end of the slider.
    total: usize,
    threshold: usize,
}

fn kept(state: &AlignState) -> Result<Alignpoints, String> {
    let candidates = state.candidates.lock().map_err(|e| e.to_string())?;
    let threshold = *state.threshold.lock().map_err(|e| e.to_string())?;
    Ok(Alignpoints {
        points: apply_threshold(&candidates, threshold)
            .iter()
            .map(|p| Point { x: p.x, y: p.y })
            .collect(),
        total: candidates.len(),
        threshold,
    })
}

/// Sets alignpoints on one frame, with the default threshold.
#[tauri::command]
pub async fn set_alignpoints(
    frame: u32,
    settings: PointSettings,
    recording: State<'_, AppState>,
    state: State<'_, AlignState>,
) -> Result<Alignpoints, String> {
    let image = {
        let mut guard = recording.0.lock().map_err(|e| e.to_string())?;
        let recording = guard.as_mut().ok_or("no recording is open")?;
        recording.read_frame(frame).map_err(|e| e.to_string())?
    };
    let params = PointParams {
        min_distance: settings.min_distance,
        max_movement: settings.max_movement,
        roi: settings.roi.map(Roi::from),
        edge_distance: settings.edge_distance,
        intensity: match settings.intensity.as_str() {
            "area3x3" => IntensityMode::Area3x3,
            "lowest" => IntensityMode::LowestPixel,
            _ => IntensityMode::Default,
        },
        lo: settings.lo,
        hi: settings.hi,
        box_size: settings.box_size,
    };
    let candidates = find_alignpoints(&image, &params);
    *state.threshold.lock().map_err(|e| e.to_string())? = default_threshold(candidates.len());
    *state.candidates.lock().map_err(|e| e.to_string())? = candidates;
    *state.reference_frame.lock().map_err(|e| e.to_string())? = frame;
    state.results.lock().map_err(|e| e.to_string())?.clear();
    kept(&state)
}

/// Moves the *Threshold* slider.
#[tauri::command]
pub fn set_alignpoint_threshold(
    threshold: usize,
    state: State<'_, AlignState>,
) -> Result<Alignpoints, String> {
    *state.threshold.lock().map_err(|e| e.to_string())? = threshold;
    kept(&state)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    done: u32,
    total: u32,
    elapsed_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameRating {
    index: u32,
    quality: f32,
    shift_x: f32,
    shift_y: f32,
    difference: f32,
}

/// Result of *Align*: one rating per frame in frame order, and the frames from best to worst.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Aligned {
    frames: Vec<FrameRating>,
    ranking: Vec<u32>,
}

/// Aligns all frames to the reference frame and rates them. Reports progress through the
/// `align-progress` event and stops on [`cancel_processing`].
#[tauri::command]
pub async fn align(
    settings: AlignSettings,
    app: AppHandle,
    recording: State<'_, AppState>,
    state: State<'_, AlignState>,
) -> Result<Aligned, String> {
    let points: Vec<AlignPoint> = {
        let candidates = state.candidates.lock().map_err(|e| e.to_string())?;
        let threshold = *state.threshold.lock().map_err(|e| e.to_string())?;
        apply_threshold(&candidates, threshold).to_vec()
    };
    if points.is_empty() {
        return Err("set alignpoints first".into());
    }
    let reference_frame = *state.reference_frame.lock().map_err(|e| e.to_string())?;
    let read = |index: u32| -> Result<_, String> {
        let mut guard = recording.0.lock().map_err(|e| e.to_string())?;
        let recording = guard.as_mut().ok_or("no recording is open")?;
        recording.read_frame(index).map_err(|e| e.to_string())
    };
    let total = {
        let guard = recording.0.lock().map_err(|e| e.to_string())?;
        guard
            .as_ref()
            .ok_or("no recording is open")?
            .info()
            .frame_count
    };
    let reference = Reference::with_roi(
        &read(reference_frame)?,
        &points,
        &AlignParams {
            box_size: settings.box_size,
            max_movement: settings.max_movement,
        },
        settings.roi.map(Roi::from),
    );

    state.cancelled.store(false, Ordering::Relaxed);
    state.paused.store(false, Ordering::Relaxed);
    let started = Instant::now();
    let results = align_sequence(&reference, total, read, |done| {
        while state.paused.load(Ordering::Relaxed) && !state.cancelled.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = app.emit(
            "align-progress",
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
    })
    .map_err(|e| match e {
        AlignError::Read(message) => message,
        AlignError::Cancelled => "cancelled".to_owned(),
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
    *state.results.lock().map_err(|e| e.to_string())? = results;
    Ok(aligned)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlignSettings {
    box_size: u32,
    max_movement: u32,
    /// Measure the drift between frames inside this region only; use the one the points were set in.
    roi: Option<RoiSettings>,
}

/// Stops a running *Align*.
#[tauri::command]
pub fn cancel_processing(state: State<'_, AlignState>) {
    state.cancelled.store(true, Ordering::Relaxed);
}

/// Pauses or resumes a running *Align*.
#[tauri::command]
pub fn pause_processing(paused: bool, state: State<'_, AlignState>) {
    state.paused.store(paused, Ordering::Relaxed);
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "mode", content = "value")]
pub enum LimitSettings {
    LowestQuality(f32),
    BestPercent(f32),
    BestCount(usize),
    FramesPerPoint(usize),
}

/// Result of *Limit*.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limited {
    /// The frame indices that stay, best first.
    frames: Vec<u32>,
    /// For every point in use, how far it sits from where the movement of the whole frame puts it, on
    /// average over the frames that stay: the green strokes in the image. `None` if it was never found.
    /// Which shift the original draws is unobserved; this is the local distortion, a few pixels.
    point_shifts: Vec<Option<[f32; 2]>>,
}

/// *Limit*: the frames that stay and the mean shift of every point over them.
#[tauri::command]
pub fn limit(settings: LimitSettings, state: State<'_, AlignState>) -> Result<Limited, String> {
    let results = state.results.lock().map_err(|e| e.to_string())?;
    if results.is_empty() {
        return Err("align first".into());
    }
    let mode = match settings {
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
    Ok(Limited {
        frames: kept.into_iter().map(|i| i as u32).collect(),
        point_shifts,
    })
}
