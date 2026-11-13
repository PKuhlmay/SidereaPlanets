//! Commands for the colour and tone panels of the Wavelet tab (*Functions*, *Contrast/Brightness*,
//! `docs/design.md`, section Colour and tone panels). The units are those of the panels, see
//! `planets_session::ColorSettings`.

use crate::{CommandResult, pack};
use planets_core::refine::RefineSettings;
use planets_session::{
    ChannelShifts, ColorSettings, HistogramData, HistogramPoints, Session, SourceInfo,
};
use tauri::{State, ipc::Response};

/// Takes the image the panels work on from the window: the sharpened image, or the stack before
/// Wavelet has run. The body is the raw RGBA bytes, the headers `width` and `height` give the size.
#[tauri::command]
pub fn set_color_source(
    request: tauri::ipc::Request<'_>,
    session: State<'_, Session>,
) -> CommandResult<SourceInfo> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected the RGBA bytes as raw body".to_owned().into());
    };
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .ok_or_else(|| format!("missing or invalid header {name}"))
    };
    Ok(session.set_color_source(header("width")?, header("height")?, bytes)?)
}

/// *RGB Align*, automatic: how far red and blue have to move to lie on green, within `max_shift`
/// pixels. Does not change the image; the panel puts the result into its fields and calls
/// [`apply_colors`].
#[tauri::command]
pub async fn estimate_rgb_align(
    max_shift: u32,
    session: State<'_, Session>,
) -> CommandResult<ChannelShifts> {
    Ok(session.estimate_rgb_align(max_shift)?)
}

/// Applies all panel settings to the source and returns the result as binary, like `read_frame`.
#[tauri::command]
pub async fn apply_colors(
    settings: ColorSettings,
    session: State<'_, Session>,
) -> CommandResult<Response> {
    Ok(pack(session.apply_colors(&settings)?))
}

/// The histogram of the image as the *Histogram* panel shows it. With `after` it is the final
/// result, otherwise the image just before the stretch.
#[tauri::command]
pub async fn color_histogram(
    settings: ColorSettings,
    after: bool,
    session: State<'_, Session>,
) -> CommandResult<HistogramData> {
    Ok(session.color_histogram(&settings, after)?)
}

/// *Histogram*, automatic: black and white point that cut off the darkest and brightest `clip`
/// percent of the pixels (0..49), on the 0..255 scale.
#[tauri::command]
pub async fn auto_histogram_points(
    settings: ColorSettings,
    clip: f32,
    session: State<'_, Session>,
) -> CommandResult<HistogramPoints> {
    Ok(session.auto_histogram_points(&settings, clip)?)
}

/// *Masking*, *Show mask*: the sharpened image with the part that gets the sharpening tinted, as binary like `read_frame`. Only a
/// view: it is not stored and does not go into *Save Image*.
#[tauri::command]
pub async fn refine_mask_overlay(
    settings: RefineSettings,
    session: State<'_, Session>,
) -> CommandResult<Response> {
    Ok(pack(session.refine_mask_overlay(&settings)?))
}
