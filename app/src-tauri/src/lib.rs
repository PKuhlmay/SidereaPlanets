mod align_commands;
mod stack_commands;
mod wavelet_commands;

use planets_io::recording::{Recording, RecordingInfo as IoInfo};
use serde::Serialize;
use std::{path::PathBuf, sync::Mutex};
use tauri::{State, ipc::Response};

/// Summary of an opened recording, as shown in the UI.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingInfo {
    format: &'static str,
    codec: Option<String>,
    width: u32,
    height: u32,
    bit_depth: u8,
    frame_count: u32,
    color: String,
    fps: Option<f64>,
    observer: Option<String>,
    instrument: Option<String>,
    telescope: Option<String>,
}

impl From<IoInfo> for RecordingInfo {
    fn from(i: IoInfo) -> Self {
        Self {
            format: i.format,
            codec: i.codec,
            width: i.width,
            height: i.height,
            bit_depth: i.bit_depth,
            frame_count: i.frame_count,
            color: i.color,
            fps: i.fps,
            observer: i.observer,
            instrument: i.instrument,
            telescope: i.telescope,
        }
    }
}

/// The recording that is currently open.
#[derive(Default)]
struct AppState(Mutex<Option<Recording>>);

/// Opens the files picked in the dialog: one video, or one or more still frames as a sequence.
#[tauri::command]
async fn open_recording(
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<RecordingInfo, String> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let recording = Recording::open_files(&paths).map_err(|e| e.to_string())?;
    let info = recording.info().into();
    *state.0.lock().map_err(|e| e.to_string())? = Some(recording);
    Ok(info)
}

/// Returns one frame as binary: width (u32 LE), height (u32 LE), then width * height * 4 bytes RGBA.
#[tauri::command]
async fn read_frame(index: u32, state: State<'_, AppState>) -> Result<Response, String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    let recording = guard.as_mut().ok_or("no recording is open")?;
    let image = recording.read_frame(index).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(8 + image.width as usize * image.height as usize * 4);
    out.extend(image.width.to_le_bytes());
    out.extend(image.height.to_le_bytes());
    out.extend(image.to_rgba8());
    Ok(Response::new(out))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .manage(align_commands::AlignState::default())
        .manage(stack_commands::StackState::default())
        .manage(wavelet_commands::WaveletState::default())
        .invoke_handler(tauri::generate_handler![
            open_recording,
            read_frame,
            align_commands::set_alignpoints,
            align_commands::set_alignpoint_threshold,
            align_commands::align,
            align_commands::cancel_processing,
            align_commands::pause_processing,
            align_commands::limit,
            stack_commands::stack,
            stack_commands::stacked_image,
            stack_commands::cancel_stacking,
            stack_commands::pause_stacking,
            wavelet_commands::wavelet_default_settings,
            wavelet_commands::wavelet_set_source_frame,
            wavelet_commands::wavelet_preview,
            wavelet_commands::wavelet_do_all,
            wavelet_commands::save_wavelet_scheme,
            wavelet_commands::load_wavelet_scheme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
