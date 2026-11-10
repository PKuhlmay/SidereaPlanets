use planets_io::recording::{Recording, RecordingInfo as IoInfo};
use serde::Serialize;
use std::{path::Path, sync::Mutex};
use tauri::{State, ipc::Response};

/// Summary of an opened recording, as shown in the UI.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingInfo {
    format: &'static str,
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

#[tauri::command]
async fn open_recording(path: String, state: State<'_, AppState>) -> Result<RecordingInfo, String> {
    let recording = Recording::open(Path::new(&path)).map_err(|e| e.to_string())?;
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
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![open_recording, read_frame])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
