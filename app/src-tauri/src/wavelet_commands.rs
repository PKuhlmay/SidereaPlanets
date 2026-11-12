//! Commands for the Wavelet tab: preview, Do All and schemes (`docs/wavelets.md`).

use crate::{CommandResult, Events, pack};
use planets_core::wavelet::{ColourMode, Region, WaveletSettings};
use planets_session::{Operation, Session, SourceInfo};
use std::path::Path;
use tauri::{AppHandle, State, ipc::Response};

/// The settings *Reset Wavelets* restores.
#[tauri::command]
pub fn wavelet_default_settings() -> WaveletSettings {
    planets_session::wavelet_default_settings()
}

/// Uses a frame of the open recording as the image of the tab. A stand-in until *Stack* sets the
/// real source.
#[tauri::command]
pub async fn wavelet_set_source_frame(
    index: u32,
    session: State<'_, Session>,
) -> CommandResult<SourceInfo> {
    Ok(session.wavelet_set_source_frame(index)?)
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
    session: State<'_, Session>,
) -> CommandResult<Response> {
    let preview = session.wavelet_preview(settings.as_ref(), area, mode)?;
    let r = preview.region();
    let mut out = Vec::with_capacity(16 + preview.rgba().len());
    for v in [r.x, r.y, r.width, r.height] {
        out.extend(v.to_le_bytes());
    }
    out.extend(preview.into_rgba());
    Ok(Response::new(out))
}

/// *Do All*: applies the settings to the whole image (tone curve and colour treatment included) and keeps the result
/// for saving.
///
/// Returns binary like `read_frame`.
#[tauri::command]
pub async fn wavelet_do_all(
    settings: WaveletSettings,
    mode: ColourMode,
    app: AppHandle,
    session: State<'_, Session>,
) -> CommandResult<Response> {
    Ok(pack(session.wavelet_do_all(
        &settings,
        mode,
        &Events(app),
    )?))
}

/// Stops a running *Do All*; it then returns the error `cancelled` and leaves the earlier result untouched.
#[tauri::command]
pub fn cancel_wavelet(session: State<'_, Session>) {
    session.cancel(Operation::Wavelet);
}

/// Pauses or resumes a running *Do All* between two layers.
#[tauri::command]
pub fn pause_wavelet(paused: bool, session: State<'_, Session>) {
    session.pause(Operation::Wavelet, paused);
}

/// *Save Scheme*: writes the settings to a `.rwv` file.
#[tauri::command]
pub fn save_wavelet_scheme(path: String, settings: WaveletSettings) -> CommandResult<()> {
    Ok(planets_session::save_wavelet_scheme(
        Path::new(&path),
        &settings,
    )?)
}

/// *Load Scheme*: reads the settings from a `.rwv` file.
#[tauri::command]
pub fn load_wavelet_scheme(path: String) -> CommandResult<WaveletSettings> {
    Ok(planets_session::load_wavelet_scheme(Path::new(&path))?)
}
