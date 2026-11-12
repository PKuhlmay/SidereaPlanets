//! Commands for the colour and tone panels of the Wavelet tab (*Functions*, *Contrast/Brightness*,
//! `docs/design.md`, section Colour and tone panels).
//!
//! All values cross the boundary in the units the panels show: percent for balance and contrast,
//! pixels for shifts, 0..255 for the histogram points, brightness on the -512..512 slider scale.

use crate::compute::ComputeState;
use planets_core::{
    Image,
    color::{Adjustments, RgbBalance, RgbShifts, Shift, adjust, estimate_rgb_shifts},
    tone::{HISTOGRAM_BINS, Levels, auto_levels, histogram},
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{State, ipc::Response};

/// The image the panels work on and the last result.
#[derive(Default)]
pub struct ColorState {
    /// The summed or sharpened image the adjustments start from.
    source: Mutex<Option<Image>>,
    /// The source with the last adjustments applied, kept for saving.
    result: Mutex<Option<Image>>,
}

impl ColorState {
    /// Forgets the source and the result: they were made from an alignment that is gone.
    pub fn clear(&self) {
        if let Ok(mut source) = self.source.lock() {
            *source = None;
        }
        if let Ok(mut result) = self.result.lock() {
            *result = None;
        }
    }

    /// Makes `image` the source of the panels and drops the previous result. The Stack and
    /// Wavelet steps call this when they have a new image.
    pub fn set_source(&self, image: Image) -> Result<(), String> {
        *self.source.lock().map_err(|e| e.to_string())? = Some(image);
        *self.result.lock().map_err(|e| e.to_string())? = None;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct ChannelShift {
    x: f32,
    y: f32,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct ChannelShifts {
    red: ChannelShift,
    green: ChannelShift,
    blue: ChannelShift,
}

impl From<ChannelShifts> for RgbShifts {
    fn from(s: ChannelShifts) -> Self {
        let shift = |c: ChannelShift| Shift { x: c.x, y: c.y };
        RgbShifts {
            red: shift(s.red),
            green: shift(s.green),
            blue: shift(s.blue),
        }
    }
}

impl From<RgbShifts> for ChannelShifts {
    fn from(s: RgbShifts) -> Self {
        let shift = |c: Shift| ChannelShift { x: c.x, y: c.y };
        ChannelShifts {
            red: shift(s.red),
            green: shift(s.green),
            blue: shift(s.blue),
        }
    }
}

/// The state of all panels at once; the result is always computed from the source.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorSettings {
    /// *RGB Align*, in pixels.
    shifts: ChannelShifts,
    /// *RGB Balance*, in percent (100 = unchanged).
    red: f32,
    green: f32,
    blue: f32,
    /// *Histogram*: black and white point on a 0..255 scale.
    black: f32,
    white: f32,
    /// *Gamma* (1.0 = unchanged).
    gamma: f32,
    /// *Contrast* in percent (100 = unchanged) and *Brightness* on the slider scale -512..512.
    contrast: f32,
    brightness: f32,
}

impl From<ColorSettings> for Adjustments {
    fn from(s: ColorSettings) -> Self {
        Adjustments {
            shifts: s.shifts.into(),
            balance: RgbBalance {
                red: s.red / 100.0,
                green: s.green / 100.0,
                blue: s.blue / 100.0,
            },
            levels: Levels {
                black: s.black / 255.0,
                white: s.white / 255.0,
            },
            gamma: s.gamma,
            contrast: s.contrast / 100.0,
            brightness: s.brightness / 512.0,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    width: u32,
    height: u32,
    /// 1 for a mono image: the colour panels do nothing then.
    channels: u8,
}

/// 8-bit RGBA, as the window holds its images, to normalised RGB. Alpha is dropped.
fn image_from_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Image, String> {
    if rgba.len() != width as usize * height as usize * 4 {
        return Err(format!(
            "{} bytes do not fit an image of {width}x{height}",
            rgba.len()
        ));
    }
    let mut image = Image::new(width, height, 3);
    let (out_pixels, _) = image.data.as_chunks_mut::<3>();
    let (in_pixels, _) = rgba.as_chunks::<4>();
    for (out, px) in out_pixels.iter_mut().zip(in_pixels) {
        for (o, &v) in out.iter_mut().zip(px) {
            *o = f32::from(v) / 255.0;
        }
    }
    Ok(image)
}

/// Takes the image the panels work on from the window: the sharpened image, or the stack before
/// Wavelet has run. The body is the raw RGBA bytes, the headers `width` and `height` give the size.
/// Stack and Wavelet may instead hand over a full-precision image with [`ColorState::set_source`].
#[tauri::command]
pub fn set_color_source(
    request: tauri::ipc::Request<'_>,
    state: State<'_, ColorState>,
) -> Result<SourceInfo, String> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected the RGBA bytes as raw body".into());
    };
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok())
            .ok_or_else(|| format!("missing or invalid header {name}"))
    };
    let image = image_from_rgba(header("width")?, header("height")?, bytes)?;
    let info = SourceInfo {
        width: image.width,
        height: image.height,
        channels: image.channels,
    };
    state.set_source(image)?;
    Ok(info)
}

fn with_source<T>(
    state: &ColorState,
    f: impl FnOnce(&Image) -> Result<T, String>,
) -> Result<T, String> {
    let guard = state.source.lock().map_err(|e| e.to_string())?;
    f(guard.as_ref().ok_or("no image to adjust")?)
}

/// *RGB Align*, automatic: how far red and blue have to move to lie on green, within `max_shift`
/// pixels. Does not change the image; the panel puts the result into its fields and calls
/// [`apply_colors`].
#[tauri::command]
pub async fn estimate_rgb_align(
    max_shift: u32,
    state: State<'_, ColorState>,
    compute: State<'_, ComputeState>,
) -> Result<ChannelShifts, String> {
    // The search grows with the square of the range; 20 px is far beyond any dispersion.
    let max_shift = max_shift.clamp(1, 20);
    with_source(&state, |image| {
        Ok(compute
            .install(|| estimate_rgb_shifts(image, max_shift))
            .into())
    })
}

/// Applies all panel settings to the source and returns the result as binary: width (u32 LE),
/// height (u32 LE), then width * height * 4 bytes RGBA, like `read_frame`.
#[tauri::command]
pub async fn apply_colors(
    settings: ColorSettings,
    state: State<'_, ColorState>,
    compute: State<'_, ComputeState>,
) -> Result<Response, String> {
    let adjustments: Adjustments = settings.into();
    let result = with_source(&state, |image| {
        Ok(compute.install(|| adjust(image, &adjustments)))
    })?;
    let mut out = Vec::with_capacity(8 + result.width as usize * result.height as usize * 4);
    out.extend(result.width.to_le_bytes());
    out.extend(result.height.to_le_bytes());
    out.extend(result.to_rgba8());
    *state.result.lock().map_err(|e| e.to_string())? = Some(result);
    Ok(Response::new(out))
}

#[derive(Serialize)]
pub struct HistogramData {
    /// Red, green and blue (one entry for a mono image), each with [`HISTOGRAM_BINS`] counts.
    channels: Vec<Vec<u32>>,
    luma: Vec<u32>,
    bins: usize,
}

/// The histogram of the image as the *Histogram* panel shows it. With `after` it is the final
/// result, otherwise the image just before the stretch, which is the one the black and white
/// point are placed on.
#[tauri::command]
pub async fn color_histogram(
    settings: ColorSettings,
    after: bool,
    state: State<'_, ColorState>,
    compute: State<'_, ComputeState>,
) -> Result<HistogramData, String> {
    let adjustments: Adjustments = settings.into();
    let adjustments = if after {
        adjustments
    } else {
        adjustments.before_levels()
    };
    let h = with_source(&state, |image| {
        Ok(compute.install(|| histogram(&adjust(image, &adjustments))))
    })?;
    Ok(HistogramData {
        channels: h.channels,
        luma: h.luma,
        bins: HISTOGRAM_BINS,
    })
}

#[derive(Serialize)]
pub struct HistogramPoints {
    black: f32,
    white: f32,
}

/// *Histogram*, automatic: black and white point that cut off the darkest and brightest `clip`
/// percent of the pixels (0..49), on the 0..255 scale. Measured on the image before the stretch.
#[tauri::command]
pub async fn auto_histogram_points(
    settings: ColorSettings,
    clip: f32,
    state: State<'_, ColorState>,
    compute: State<'_, ComputeState>,
) -> Result<HistogramPoints, String> {
    let adjustments = Adjustments::from(settings).before_levels();
    let levels = with_source(&state, |image| {
        Ok(compute.install(|| auto_levels(&adjust(image, &adjustments), clip / 100.0)))
    })?;
    Ok(HistogramPoints {
        black: levels.black * 255.0,
        white: levels.white * 255.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_becomes_normalised_rgb() {
        let image = image_from_rgba(2, 1, &[255, 0, 51, 7, 0, 255, 0, 255]).unwrap();
        assert_eq!((image.width, image.height, image.channels), (2, 1, 3));
        assert_eq!(image.data, vec![1.0, 0.0, 0.2, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn clearing_forgets_the_source_and_the_result() {
        let state = ColorState::default();
        state.set_source(Image::new(2, 2, 3)).unwrap();
        *state.result.lock().unwrap() = Some(Image::new(2, 2, 3));
        state.clear();
        assert!(state.source.lock().unwrap().is_none());
        assert!(state.result.lock().unwrap().is_none());
    }

    #[test]
    fn a_wrong_byte_count_is_refused() {
        assert!(image_from_rgba(2, 2, &[0; 15]).is_err());
    }

    #[test]
    fn panel_units_become_engine_units() {
        let adjustments: Adjustments = ColorSettings {
            shifts: ChannelShifts {
                red: ChannelShift { x: 1.0, y: -1.0 },
                green: ChannelShift { x: 0.0, y: 0.0 },
                blue: ChannelShift { x: 0.0, y: 0.0 },
            },
            red: 50.0,
            green: 100.0,
            blue: 200.0,
            black: 51.0,
            white: 255.0,
            gamma: 2.0,
            contrast: 150.0,
            brightness: 256.0,
        }
        .into();
        assert_eq!(adjustments.shifts.red, Shift { x: 1.0, y: -1.0 });
        assert_eq!(adjustments.balance.red, 0.5);
        assert_eq!(adjustments.balance.blue, 2.0);
        assert_eq!(adjustments.levels.black, 0.2);
        assert_eq!(adjustments.levels.white, 1.0);
        assert_eq!(adjustments.contrast, 1.5);
        assert_eq!(adjustments.brightness, 0.5);
    }
}
