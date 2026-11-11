//! Commands for the Wavelet tab: preview, Do All and schemes (`docs/wavelets.md`).

use crate::{AppState, export_commands::ExportState};
use planets_core::{
    Image,
    wavelet::{self, ColourMode, Previewer, Region, WaveletSettings, rgba_region, scheme},
};
use serde::Serialize;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Manager, State, ipc::Response};

/// What the Wavelet tab works on and what it has produced.
#[derive(Default)]
pub struct WaveletState {
    previewer: Mutex<Option<Previewer>>,
    processed: Mutex<Option<Arc<Image>>>,
}

impl WaveletState {
    /// Sets the image the tab sharpens (the stacked image) and forgets the previous result.
    /// Called by the `stack` command after *Stack*.
    pub fn set_source(&self, image: Image) -> Result<(), String> {
        *self.previewer.lock().map_err(|e| e.to_string())? = Some(Previewer::new(Arc::new(image)));
        *self.processed.lock().map_err(|e| e.to_string())? = None;
        Ok(())
    }

    /// Forgets the image and the result: they were made from an alignment that is gone.
    pub fn clear(&self) {
        if let Ok(mut previewer) = self.previewer.lock() {
            *previewer = None;
        }
        if let Ok(mut processed) = self.processed.lock() {
            *processed = None;
        }
    }

    /// The result of the last *Do All*, for *Save image* (RK-009).
    #[allow(dead_code)] // first caller is the export command of RK-009
    pub fn processed(&self) -> Option<Arc<Image>> {
        self.processed.lock().ok().and_then(|p| p.clone())
    }
}

/// Size of the image in the Wavelet tab.
#[derive(Serialize)]
pub struct SourceInfo {
    width: u32,
    height: u32,
    channels: u8,
}

fn info(image: &Image) -> SourceInfo {
    SourceInfo {
        width: image.width,
        height: image.height,
        channels: image.channels,
    }
}

/// The settings *Reset Wavelets* restores.
#[tauri::command]
pub fn wavelet_default_settings() -> WaveletSettings {
    WaveletSettings::default()
}

/// Uses a frame of the open recording as the image of the tab. A stand-in until *Stack* sets the
/// real source through [`WaveletState::set_source`].
#[tauri::command]
pub async fn wavelet_set_source_frame(
    index: u32,
    recording: State<'_, AppState>,
    state: State<'_, WaveletState>,
) -> Result<SourceInfo, String> {
    let image = {
        let mut guard = recording.0.lock().map_err(|e| e.to_string())?;
        let recording = guard.as_mut().ok_or("no recording is open")?;
        recording.read_frame(index).map_err(|e| e.to_string())?
    };
    let source = info(&image);
    state.set_source(image)?;
    Ok(source)
}

/// Renders the preview of one area, with the tone curve and colour treatment *Do All* uses. `settings` of `null` gives
/// the unprocessed image (*Before*).
///
/// Returns binary: x, y, width, height of the area that was rendered (4 x u32 LE; the requested area cut to
/// the image), then width * height * 4 bytes RGBA.
#[tauri::command]
pub async fn wavelet_preview(
    settings: Option<WaveletSettings>,
    area: Region,
    mode: ColourMode,
    state: State<'_, WaveletState>,
) -> Result<Response, String> {
    let mut guard = state.previewer.lock().map_err(|e| e.to_string())?;
    let previewer = guard.as_mut().ok_or("no image to sharpen")?;
    let preview = match settings {
        Some(settings) => previewer.render(&settings, mode, area),
        None => previewer.before(area),
    };
    let r = preview.region;
    let mut out = Vec::with_capacity(16 + preview.rgba.len());
    for v in [r.x, r.y, r.width, r.height] {
        out.extend(v.to_le_bytes());
    }
    out.extend(preview.rgba);
    Ok(Response::new(out))
}

/// *Do All*: applies the settings to the whole image (tone curve and colour treatment included) and keeps the result
/// for saving.
///
/// Returns binary like `read_frame`: width (u32 LE), height (u32 LE), then width * height * 4 bytes RGBA.
#[tauri::command]
pub async fn wavelet_do_all(
    settings: WaveletSettings,
    mode: ColourMode,
    app: AppHandle,
    state: State<'_, WaveletState>,
) -> Result<Response, String> {
    let source = {
        let guard = state.previewer.lock().map_err(|e| e.to_string())?;
        guard
            .as_ref()
            .ok_or("no image to sharpen")?
            .source()
            .clone()
    };
    let result = Arc::new(wavelet::do_all(&source, &settings, mode));
    let rgba = rgba_region(&result, Region::full(result.width, result.height));
    let mut out = Vec::with_capacity(8 + rgba.len());
    out.extend(result.width.to_le_bytes());
    out.extend(result.height.to_le_bytes());
    out.extend(rgba);
    app.state::<ExportState>().set_wavelet((*result).clone());
    *state.processed.lock().map_err(|e| e.to_string())? = Some(result);
    Ok(Response::new(out))
}

/// *Save Scheme*: writes the settings to a `.rwv` file.
#[tauri::command]
pub fn save_wavelet_scheme(path: String, settings: WaveletSettings) -> Result<(), String> {
    fs::write(&path, scheme::to_rwv(&settings)).map_err(|e| format!("{path}: {e}"))
}

/// *Load Scheme*: reads the settings from a `.rwv` file.
#[tauri::command]
pub fn load_wavelet_scheme(path: String) -> Result<WaveletSettings, String> {
    // The original writes ANSI text; the file only holds ASCII, so a lossy conversion is enough.
    let bytes = fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
    scheme::from_rwv(&String::from_utf8_lossy(&bytes)).map_err(|e| format!("{path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image() -> Image {
        Image::new(4, 4, 1)
    }

    #[test]
    fn clearing_forgets_the_source_and_the_result() {
        let state = WaveletState::default();
        state.set_source(image()).unwrap();
        *state.processed.lock().unwrap() = Some(Arc::new(image()));
        state.clear();
        assert!(state.previewer.lock().unwrap().is_none());
        assert!(state.processed().is_none());
    }
}
