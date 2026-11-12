//! The colour and tone panels of the Wavelet tab (*Functions*, *Contrast/Brightness*,
//! `docs/design.md`, section Colour and tone panels).
//!
//! All values cross the boundary in the units the panels show: percent for balance and contrast,
//! pixels for shifts, 0..255 for the histogram points, brightness on the -512..512 slider scale.

use crate::{Session, SessionError, error::lock, image::RgbaImage, wavelet::SourceInfo};
use planets_core::{
    Image,
    color::{Adjustments, ChannelMix, RgbBalance, RgbShifts, Shift, adjust, estimate_rgb_shifts},
    tone::{HISTOGRAM_BINS, Levels, auto_levels, histogram},
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// The image the panels work on and the last result.
#[derive(Default)]
pub(crate) struct ColorState {
    /// The summed or sharpened image the adjustments start from.
    source: Mutex<Option<Image>>,
    /// The source with the last adjustments applied, kept for saving.
    result: Mutex<Option<Image>>,
}

impl ColorState {
    /// Forgets the source and the result: they were made from an alignment that is gone.
    pub(crate) fn clear(&self) {
        if let Ok(mut source) = self.source.lock() {
            *source = None;
        }
        if let Ok(mut result) = self.result.lock() {
            *result = None;
        }
    }

    /// Makes `image` the source of the panels and drops the previous result.
    pub(crate) fn set_source(&self, image: Image) -> Result<(), SessionError> {
        *lock(&self.source)? = Some(image);
        *lock(&self.result)? = None;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct ChannelShift {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct ChannelShifts {
    pub red: ChannelShift,
    pub green: ChannelShift,
    pub blue: ChannelShift,
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

/// Range of one weight of *Color Mixing*, in percent.
pub const MIX_LIMIT: f32 = 200.0;

/// *Color Mixing* in the units of the card: percent, `matrix[3 * output + input]` with the order red, green,
/// blue. The default is the identity (100 on the diagonal), which leaves the image alone.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ColorMix {
    pub matrix: [f32; 9],
    pub preserve_brightness: bool,
}

impl Default for ColorMix {
    fn default() -> Self {
        Self {
            matrix: [100.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 100.0],
            preserve_brightness: false,
        }
    }
}

impl From<ColorMix> for ChannelMix {
    fn from(m: ColorMix) -> Self {
        let weight = |i: usize| m.matrix[i].clamp(-MIX_LIMIT, MIX_LIMIT) / 100.0;
        Self {
            matrix: std::array::from_fn(|row| std::array::from_fn(|col| weight(row * 3 + col))),
            preserve_brightness: m.preserve_brightness,
        }
    }
}

/// The state of all panels at once; the result is always computed from the source.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorSettings {
    /// *RGB Align*, in pixels.
    pub shifts: ChannelShifts,
    /// *Color Mixing*; a settings object without it (older callers, project files) means the identity.
    #[serde(default)]
    pub mix: ColorMix,
    /// *RGB Balance*, in percent (100 = unchanged).
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    /// *Histogram*: black and white point on a 0..255 scale.
    pub black: f32,
    pub white: f32,
    /// *Gamma* (1.0 = unchanged).
    pub gamma: f32,
    /// *Contrast* in percent (100 = unchanged) and *Brightness* on the slider scale -512..512.
    pub contrast: f32,
    pub brightness: f32,
}

impl From<ColorSettings> for Adjustments {
    fn from(s: ColorSettings) -> Self {
        Adjustments {
            shifts: s.shifts.into(),
            mix: s.mix.into(),
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

/// 8-bit RGBA, as the window holds its images, to normalised RGB. Alpha is dropped.
fn image_from_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Image, SessionError> {
    if rgba.len() != width as usize * height as usize * 4 {
        return Err(SessionError::BadImageSize {
            width,
            height,
            bytes: rgba.len(),
        });
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistogramData {
    /// Red, green and blue (one entry for a mono image), each with [`HISTOGRAM_BINS`] counts.
    pub channels: Vec<Vec<u32>>,
    pub luma: Vec<u32>,
    pub bins: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HistogramPoints {
    pub black: f32,
    pub white: f32,
}

impl Session {
    /// Takes the image the panels work on from the window: the sharpened image, or the stack before
    /// Wavelet has run, as raw RGBA bytes of `width` x `height`.
    pub fn set_color_source(
        &self,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<SourceInfo, SessionError> {
        let image = image_from_rgba(width, height, rgba)?;
        let info = SourceInfo::of(&image);
        self.color.set_source(image)?;
        Ok(info)
    }

    fn with_color_source<T>(
        &self,
        f: impl FnOnce(&Image) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        let guard = lock(&self.color.source)?;
        f(guard.as_ref().ok_or(SessionError::NoImageToAdjust)?)
    }

    /// *RGB Align*, automatic: how far red and blue have to move to lie on green, within `max_shift`
    /// pixels. Does not change the image; the panel puts the result into its fields and calls
    /// [`Session::apply_colors`].
    pub fn estimate_rgb_align(&self, max_shift: u32) -> Result<ChannelShifts, SessionError> {
        // The search grows with the square of the range; 20 px is far beyond any dispersion.
        let max_shift = max_shift.clamp(1, 20);
        self.with_color_source(|image| {
            Ok(self
                .compute
                .install(|| estimate_rgb_shifts(image, max_shift))
                .into())
        })
    }

    /// Applies all panel settings to the source and keeps the result for saving.
    pub fn apply_colors(&self, settings: &ColorSettings) -> Result<RgbaImage, SessionError> {
        let adjustments: Adjustments = (*settings).into();
        let result = self
            .with_color_source(|image| Ok(self.compute.install(|| adjust(image, &adjustments))))?;
        let rgba = RgbaImage::from_image(&result);
        *lock(&self.color.result)? = Some(result);
        Ok(rgba)
    }

    /// The histogram of the image as the *Histogram* panel shows it. With `after` it is the final
    /// result, otherwise the image just before the stretch, which is the one the black and white
    /// point are placed on.
    pub fn color_histogram(
        &self,
        settings: &ColorSettings,
        after: bool,
    ) -> Result<HistogramData, SessionError> {
        let adjustments: Adjustments = (*settings).into();
        let adjustments = if after {
            adjustments
        } else {
            adjustments.before_levels()
        };
        let h = self.with_color_source(|image| {
            Ok(self
                .compute
                .install(|| histogram(&adjust(image, &adjustments))))
        })?;
        Ok(HistogramData {
            channels: h.channels,
            luma: h.luma,
            bins: HISTOGRAM_BINS,
        })
    }

    /// *Histogram*, automatic: black and white point that cut off the darkest and brightest `clip`
    /// percent of the pixels (0..49), on the 0..255 scale. Measured on the image before the stretch.
    pub fn auto_histogram_points(
        &self,
        settings: &ColorSettings,
        clip: f32,
    ) -> Result<HistogramPoints, SessionError> {
        let adjustments = Adjustments::from(*settings).before_levels();
        let levels = self.with_color_source(|image| {
            Ok(self
                .compute
                .install(|| auto_levels(&adjust(image, &adjustments), clip / 100.0)))
        })?;
        Ok(HistogramPoints {
            black: levels.black * 255.0,
            white: levels.white * 255.0,
        })
    }
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
            mix: ColorMix::default(),
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

    #[test]
    fn mixing_percent_becomes_factors_within_the_limit() {
        let mut mix = ColorMix::default();
        mix.matrix[1] = 50.0;
        mix.matrix[5] = -999.0;
        mix.preserve_brightness = true;
        let engine: ChannelMix = mix.into();
        assert_eq!(engine.matrix[0], [1.0, 0.5, 0.0]);
        assert_eq!(engine.matrix[1][2], -2.0);
        assert!(engine.preserve_brightness);
        assert!(ChannelMix::from(ColorMix::default()).is_identity());
    }

    fn neutral_settings(mix: Option<serde_json::Value>) -> ColorSettings {
        let mut value = serde_json::json!({
            "shifts": { "red": { "x": 0, "y": 0 }, "green": { "x": 0, "y": 0 }, "blue": { "x": 0, "y": 0 } },
            "red": 100, "green": 100, "blue": 100, "black": 0, "white": 255,
            "gamma": 1, "contrast": 100, "brightness": 0
        });
        if let Some(mix) = mix {
            value["mix"] = mix;
        }
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn settings_without_a_mix_mean_the_identity() {
        assert_eq!(neutral_settings(None).mix, ColorMix::default());
        let with = neutral_settings(Some(serde_json::json!({
            "matrix": [0, 100, 0, 100, 0, 0, 0, 0, 100], "preserveBrightness": true
        })));
        assert_eq!(with.mix.matrix[1], 100.0);
        assert!(with.mix.preserve_brightness);
    }

    #[test]
    fn applying_the_colours_mixes_the_source() {
        let session = Session::new().unwrap();
        session
            .set_color_source(1, 1, &[200, 100, 50, 255])
            .unwrap();
        let mut settings = neutral_settings(None);
        // Red and blue swap places.
        settings.mix.matrix = [0.0, 0.0, 100.0, 0.0, 100.0, 0.0, 100.0, 0.0, 0.0];
        let out = session.apply_colors(&settings).unwrap();
        assert_eq!(&out.into_rgba()[..3], &[50, 100, 200]);
    }
}
