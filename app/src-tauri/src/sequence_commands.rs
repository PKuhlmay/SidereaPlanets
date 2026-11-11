//! Commands for *Create AVI* in the Stack tab: *Save Aligned* and *Save Integrated*
//! (`docs/formats.md`, section "Create AVI").
//!
//! The frames that stay after *Limit* are moved onto the reference frame like *Stack* does and
//! streamed into an uncompressed AVI. *Save Integrated* writes the moving average over the aligned
//! frames instead. Cancel and Pause use the flags of *Stack*.

use crate::{
    AppState,
    align_commands::AlignState,
    export_commands::{pick_save, start_dialog},
    stack_commands::{StackSettings, StackState},
};
use planets_core::{
    Image,
    align::{Control, FrameAlignment, Reference},
    sequence::{MovingAverage, SequenceError, aligned_frames},
    stack::StackParams,
};
use planets_io::avi_write::AviWriter;
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Instant};
use tauri::{AppHandle, Emitter, State};

/// What *Save Aligned* and *Save Integrated* need: the Stack tab as for *Stack*, plus *Create AVI*.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceSettings {
    stack: StackSettings,
    /// *Framerate* of the video.
    framerate: u32,
    /// *Sort On Quality* (*Save Aligned* only): the best frame first.
    sort_on_quality: bool,
    /// *Average of n Frames* (*Save Integrated* only).
    average_frames: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Progress {
    done: u32,
    total: u32,
    elapsed_ms: u64,
}

/// What the status bar says after a video was written.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedVideo {
    path: String,
    /// Frames in the video.
    frames: u32,
}

/// Opens the save dialog for a video. `None` if the user cancels. The file dialog comes before the
/// job so that the time used does not include the time spent choosing a name.
#[tauri::command]
pub async fn choose_video_path(
    app: AppHandle,
    suggested_name: Option<String>,
    directory: Option<String>,
) -> Result<Option<String>, String> {
    let name = suggested_name.map(|n| with_avi_extension(Path::new(&n)));
    let dialog = start_dialog(&app, name.as_deref(), directory.as_deref())
        .set_title("Save video")
        .add_filter("AVI video", &["avi"]);
    Ok(pick_save(dialog)?.map(|p| with_avi_extension(&p).to_string_lossy().into_owned()))
}

fn with_avi_extension(path: &Path) -> std::path::PathBuf {
    match path.extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case("avi") => path.to_path_buf(),
        _ => {
            let mut name = path.as_os_str().to_owned();
            name.push(".avi");
            name.into()
        }
    }
}

/// *Save Aligned*: every frame that stays, moved onto the reference frame, as one video frame each.
#[tauri::command]
pub async fn save_aligned(
    path: String,
    settings: SequenceSettings,
    app: AppHandle,
    recording: State<'_, AppState>,
    align: State<'_, AlignState>,
    state: State<'_, StackState>,
) -> Result<SavedVideo, String> {
    write(
        Mode::Aligned,
        &path,
        &settings,
        &app,
        &recording,
        &align,
        &state,
    )
}

/// *Save Integrated*: the moving average over *Average of n Frames* aligned frames.
#[tauri::command]
pub async fn save_integrated(
    path: String,
    settings: SequenceSettings,
    app: AppHandle,
    recording: State<'_, AppState>,
    align: State<'_, AlignState>,
    state: State<'_, StackState>,
) -> Result<SavedVideo, String> {
    write(
        Mode::Integrated,
        &path,
        &settings,
        &app,
        &recording,
        &align,
        &state,
    )
}

#[derive(Clone, Copy)]
enum Mode {
    Aligned,
    Integrated,
}

fn write(
    mode: Mode,
    path: &str,
    settings: &SequenceSettings,
    app: &AppHandle,
    recording: &State<'_, AppState>,
    align: &AlignState,
    state: &StackState,
) -> Result<SavedVideo, String> {
    let stack = &settings.stack;
    if stack.point_count() == 0 {
        return Err("set alignpoints first".into());
    }
    let read = |index: u32| -> Result<Image, String> {
        let mut guard = recording.0.lock().map_err(|e| e.to_string())?;
        let recording = guard.as_mut().ok_or("no recording is open")?;
        recording.read_frame(index).map_err(|e| e.to_string())
    };
    let frame_count = {
        let guard = recording.0.lock().map_err(|e| e.to_string())?;
        guard
            .as_ref()
            .ok_or("no recording is open")?
            .info()
            .frame_count
    };
    let reference_frame = read(stack.reference_frame)?;
    let reference = stack.reference(&reference_frame);
    let known = align.results_for(stack.point_count(), frame_count);

    let mut selection = stack.frames.clone();
    if matches!(mode, Mode::Aligned) && settings.sort_on_quality {
        let results = known
            .as_deref()
            .ok_or("Sort On Quality needs the result of Align")?;
        sort_best_first(&mut selection, results);
    }

    state.begin();
    let total = selection.len() as u32;
    let started = Instant::now();
    let frames = write_video(
        Path::new(path),
        &VideoJob {
            fps: settings.framerate,
            average: matches!(mode, Mode::Integrated).then_some(settings.average_frames),
            reference: &reference,
            reference_frame: &reference_frame,
            params: &stack.params(),
            selection: &selection,
            known: known.as_deref(),
        },
        read,
        |done| {
            let control = state.checkpoint();
            let _ = app.emit(
                "sequence-progress",
                Progress {
                    done,
                    total,
                    elapsed_ms: started.elapsed().as_millis() as u64,
                },
            );
            control
        },
    )?;
    Ok(SavedVideo {
        path: path.to_owned(),
        frames,
    })
}

/// Puts the frames of `selection` in the order of their quality, best first.
fn sort_best_first(selection: &mut [u32], results: &[FrameAlignment]) {
    let quality = |frame: &u32| results.get(*frame as usize).map_or(0.0, |r| r.quality);
    selection.sort_by(|a, b| quality(b).total_cmp(&quality(a)));
}

/// Everything [`write_video`] needs apart from the file, the reader and the progress callback.
struct VideoJob<'a> {
    fps: u32,
    /// `None` writes every aligned frame, `Some(n)` the moving average over `n` of them.
    average: Option<usize>,
    reference: &'a Reference,
    reference_frame: &'a Image,
    params: &'a StackParams,
    selection: &'a [u32],
    known: Option<&'a [FrameAlignment]>,
}

/// Writes the video; the result is the number of frames in it. A job that stops early leaves no
/// file behind.
fn write_video(
    path: &Path,
    job: &VideoJob,
    read: impl FnMut(u32) -> Result<Image, String>,
    progress: impl FnMut(u32) -> Control,
) -> Result<u32, String> {
    let mut writer = AviWriter::create(
        path,
        job.reference_frame.width,
        job.reference_frame.height,
        job.fps,
    )
    .map_err(|e| e.to_string())?;
    let mut average = job.average.map(MovingAverage::new);
    let mut written = 0u32;
    let mut put = |image: &Image| -> Result<(), String> {
        writer.write_frame(image).map_err(|e| e.to_string())?;
        written += 1;
        Ok(())
    };
    aligned_frames(
        job.reference,
        job.reference_frame,
        job.params,
        job.selection,
        job.known,
        read,
        |frame| match average.as_mut() {
            None => put(&frame),
            Some(window) => window.push(frame).map_or(Ok(()), |mean| put(&mean)),
        },
        progress,
    )
    .map_err(|e| match e {
        SequenceError::NoFrames => "no frames selected".to_owned(),
        SequenceError::Read(message) | SequenceError::Write(message) => message,
        SequenceError::Cancelled => "cancelled".to_owned(),
    })?;
    if let Some(mean) = average.as_mut().and_then(MovingAverage::finish) {
        put(&mean)?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use planets_core::align::{AlignParams, AlignPoint};
    use planets_io::recording::Recording;
    use std::path::PathBuf;

    fn frame(level: f32) -> Image {
        Image {
            width: 8,
            height: 6,
            channels: 1,
            data: vec![level; 48],
        }
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("planets-seq-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    /// Writes four flat frames with the levels 0, 64, 128, 192 and reads the video back.
    fn run(average: Option<usize>, name: &str) -> Vec<u8> {
        let base = frame(0.0);
        let reference = Reference::new(&base, &[] as &[AlignPoint], &AlignParams::default());
        let params = StackParams {
            correct_geometry: false,
            ..StackParams::default()
        };
        let still = FrameAlignment {
            shift: (0.0, 0.0),
            points: vec![],
            quality: 1.0,
            difference: 0.0,
        };
        let known = vec![still; 4];
        let path = temp(name);
        let frames = write_video(
            &path,
            &VideoJob {
                fps: 25,
                average,
                reference: &reference,
                reference_frame: &base,
                params: &params,
                selection: &[0, 1, 2, 3],
                known: Some(&known),
            },
            |i| Ok(frame(f32::from(i as u8) * 64.0 / 255.0)),
            |_| Control::Continue,
        )
        .unwrap();
        let mut video = Recording::open_files(std::slice::from_ref(&path)).unwrap();
        assert_eq!(video.info().frame_count, frames);
        (0..frames)
            .map(|i| (video.read_frame(i).unwrap().data[0] * 255.0).round() as u8)
            .collect()
    }

    #[test]
    fn save_aligned_writes_one_video_frame_per_selected_frame() {
        assert_eq!(run(None, "aligned.avi"), vec![0, 64, 128, 192]);
    }

    #[test]
    fn save_integrated_writes_the_moving_average() {
        // Windows of two: (0, 64), (64, 128), (128, 192).
        assert_eq!(run(Some(2), "integrated.avi"), vec![32, 96, 160]);
        // A window larger than the selection gives the mean of all four.
        assert_eq!(run(Some(9), "integrated-all.avi"), vec![96]);
    }

    #[test]
    fn a_cancelled_job_leaves_no_file() {
        let base = frame(0.0);
        let reference = Reference::new(&base, &[] as &[AlignPoint], &AlignParams::default());
        let path = temp("cancelled.avi");
        let result = write_video(
            &path,
            &VideoJob {
                fps: 25,
                average: None,
                reference: &reference,
                reference_frame: &base,
                params: &StackParams::default(),
                selection: &[0, 0],
                known: None,
            },
            |_| Ok(frame(0.5)),
            |_| Control::Cancel,
        );
        assert_eq!(result, Err("cancelled".to_owned()));
        assert!(!path.exists());
    }

    #[test]
    fn frames_are_sorted_best_quality_first() {
        let rated = |quality| FrameAlignment {
            shift: (0.0, 0.0),
            points: vec![],
            quality,
            difference: 0.0,
        };
        let results = [rated(1.0), rated(3.0), rated(2.0)];
        let mut selection = [0, 2, 1];
        sort_best_first(&mut selection, &results);
        assert_eq!(selection, [1, 2, 0]);
    }

    #[test]
    fn the_file_name_ends_in_avi() {
        assert_eq!(
            with_avi_extension(Path::new("a.AVI")),
            PathBuf::from("a.AVI")
        );
        assert_eq!(
            with_avi_extension(Path::new("a.mp4")),
            PathBuf::from("a.mp4.avi")
        );
        assert_eq!(with_avi_extension(Path::new("a")), PathBuf::from("a.avi"));
    }
}
