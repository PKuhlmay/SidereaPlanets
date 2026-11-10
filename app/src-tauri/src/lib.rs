use planets_io::recording::{Recording, RecordingInfo as IoInfo};
use serde::Serialize;
use std::path::Path;

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

#[tauri::command]
fn open_recording(path: String) -> Result<RecordingInfo, String> {
    let recording = Recording::open(Path::new(&path)).map_err(|e| e.to_string())?;
    Ok(recording.info().into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
