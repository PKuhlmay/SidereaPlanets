use planets_io::ser::{SerHeader, SerReader};
use serde::Serialize;
use std::path::Path;

/// Summary of an opened recording, as shown in the UI.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingInfo {
    width: u32,
    height: u32,
    bit_depth: u8,
    frame_count: u32,
    color: String,
    observer: String,
    instrument: String,
    telescope: String,
}

impl From<&SerHeader> for RecordingInfo {
    fn from(h: &SerHeader) -> Self {
        Self {
            width: h.width,
            height: h.height,
            bit_depth: h.bit_depth,
            frame_count: h.frame_count,
            color: format!("{:?}", h.color),
            observer: h.observer.clone(),
            instrument: h.instrument.clone(),
            telescope: h.telescope.clone(),
        }
    }
}

#[tauri::command]
fn open_recording(path: String) -> Result<RecordingInfo, String> {
    let reader = SerReader::open(Path::new(&path)).map_err(|e| e.to_string())?;
    Ok(RecordingInfo::from(reader.header()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
