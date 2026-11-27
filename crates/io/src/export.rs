//! Writers for result images (*Save image* in the Stack and Wavelet tabs, `docs/formats.md`).
//!
//! The format is chosen explicitly, not derived from the file name, like the file type filter of
//! the reference program decides there. [`save_image`] fixes the extension of the path to match the format.
//! Files are written under a temporary name and renamed at the end, so a failed save never
//! leaves a broken file in place of a good one.

use crate::fits_write::{FitsDepth, write_fits};
use image::{ExtendedColorType, ImageBuffer, ImageEncoder, ImageFormat, Luma, Rgb, codecs::jpeg};
use planets_core::Image;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

/// JPEG quality (1 to 100). The reference program does not say what it uses; this is a visually lossless choice.
const JPEG_QUALITY: u8 = 95;

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Encode(#[from] image::ImageError),
    #[error("cannot export: {0}")]
    InvalidImage(&'static str),
}

/// Output formats of *Save image*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    /// 8 bit per channel.
    Bmp,
    /// 8 bit per channel, lossy.
    Jpeg,
    /// 16 bit per channel, uncompressed.
    Tiff16,
    /// 16 bit per channel.
    Png16,
    /// FITS with 16 bit integers.
    Fits16,
    /// FITS with 32 bit integers.
    Fits32,
    /// FITS with 32 bit floating point samples.
    Fits32Float,
}

impl ExportFormat {
    /// Formats offered after stacking, in the order of the file type list of the reference program.
    pub const STACK: &'static [ExportFormat] = &[
        Self::Bmp,
        Self::Jpeg,
        Self::Fits16,
        Self::Fits32,
        Self::Tiff16,
        Self::Png16,
        Self::Fits32Float,
    ];

    /// Shorter list offered in the Wavelet tab.
    pub const WAVELET: &'static [ExportFormat] = &[
        Self::Bmp,
        Self::Jpeg,
        Self::Fits16,
        Self::Tiff16,
        Self::Png16,
    ];

    /// File extension without the dot, as written.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Bmp => "bmp",
            Self::Jpeg => "jpg",
            Self::Tiff16 => "tif",
            Self::Png16 => "png",
            Self::Fits16 | Self::Fits32 | Self::Fits32Float => "fit",
        }
    }

    /// Name in the file type list.
    pub fn label(self) -> &'static str {
        match self {
            Self::Bmp => "BMP (8 bit)",
            Self::Jpeg => "JPEG (8 bit)",
            Self::Tiff16 => "TIFF (16 bit)",
            Self::Png16 => "PNG (16 bit)",
            Self::Fits16 => "FITS (16 bit)",
            Self::Fits32 => "FITS (32 bit)",
            Self::Fits32Float => "FITS (32 bit float)",
        }
    }

    /// All extensions (lower case, no dot) that count as this format.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Bmp => &["bmp"],
            Self::Jpeg => &["jpg", "jpeg"],
            Self::Tiff16 => &["tif", "tiff"],
            Self::Png16 => &["png"],
            Self::Fits16 | Self::Fits32 | Self::Fits32Float => &["fit", "fits", "fts"],
        }
    }

    /// Returns `path` with an extension that matches the format: an accepted one (any case) is
    /// kept, anything else is replaced. A path without extension gets one.
    pub fn with_extension(self, path: &Path) -> PathBuf {
        let accepted = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .is_some_and(|e| self.extensions().contains(&e.as_str()));
        if accepted {
            path.to_path_buf()
        } else {
            path.with_extension(self.extension())
        }
    }
}

/// Saves `image` in `format`. The extension of `path` is corrected with
/// [`ExportFormat::with_extension`]; the returned path is the file that was written.
pub fn save_image(
    image: &Image,
    path: &Path,
    format: ExportFormat,
) -> Result<PathBuf, ExportError> {
    check(image)?;
    let path = format.with_extension(path);
    write_atomic(&path, |out| write_image(out, image, format))?;
    Ok(path)
}

fn check(image: &Image) -> Result<(), ExportError> {
    if image.width == 0 || image.height == 0 {
        return Err(ExportError::InvalidImage("the image is empty"));
    }
    let expected = image.width as usize * image.height as usize * usize::from(image.channels);
    if image.channels == 0 || image.data.len() != expected {
        return Err(ExportError::InvalidImage(
            "the sample count does not match the image size",
        ));
    }
    Ok(())
}

fn write_image(
    out: &mut BufWriter<File>,
    image: &Image,
    format: ExportFormat,
) -> Result<(), ExportError> {
    let (w, h) = (image.width, image.height);
    let colour = image.channels >= 3;
    match format {
        ExportFormat::Bmp => {
            if colour {
                buffer::<Rgb<u8>>(w, h, samples(image, to_u8))?.write_to(out, ImageFormat::Bmp)?;
            } else {
                buffer::<Luma<u8>>(w, h, samples(image, to_u8))?.write_to(out, ImageFormat::Bmp)?;
            }
        }
        ExportFormat::Jpeg => {
            let encoder = jpeg::JpegEncoder::new_with_quality(out, JPEG_QUALITY);
            let (color_type, bytes) = if colour {
                (ExtendedColorType::Rgb8, samples(image, to_u8))
            } else {
                (ExtendedColorType::L8, samples(image, to_u8))
            };
            encoder.write_image(&bytes, w, h, color_type)?;
        }
        ExportFormat::Tiff16 => {
            if colour {
                buffer::<Rgb<u16>>(w, h, samples(image, to_u16))?
                    .write_to(out, ImageFormat::Tiff)?;
            } else {
                buffer::<Luma<u16>>(w, h, samples(image, to_u16))?
                    .write_to(out, ImageFormat::Tiff)?;
            }
        }
        ExportFormat::Png16 => {
            if colour {
                buffer::<Rgb<u16>>(w, h, samples(image, to_u16))?
                    .write_to(out, ImageFormat::Png)?;
            } else {
                buffer::<Luma<u16>>(w, h, samples(image, to_u16))?
                    .write_to(out, ImageFormat::Png)?;
            }
        }
        ExportFormat::Fits16 => write_fits(out, image, FitsDepth::Int16)?,
        ExportFormat::Fits32 => write_fits(out, image, FitsDepth::Int32)?,
        ExportFormat::Fits32Float => write_fits(out, image, FitsDepth::Float32)?,
    }
    Ok(())
}

/// Wraps raw samples into an image buffer of pixel type `P`.
fn buffer<P: image::Pixel>(
    width: u32,
    height: u32,
    samples: Vec<P::Subpixel>,
) -> Result<ImageBuffer<P, Vec<P::Subpixel>>, ExportError> {
    ImageBuffer::from_raw(width, height, samples).ok_or(ExportError::InvalidImage(
        "the sample buffer has the wrong size",
    ))
}

/// Converts to interleaved samples of the output type: one channel for mono (and anything with
/// fewer than three channels), three for colour. Further channels such as alpha are dropped.
fn samples<T>(image: &Image, convert: impl Fn(f32) -> T) -> Vec<T> {
    let channels = usize::from(image.channels).max(1);
    if channels >= 3 {
        image
            .data
            .chunks_exact(channels)
            .flat_map(|px| [convert(px[0]), convert(px[1]), convert(px[2])])
            .collect()
    } else {
        image
            .data
            .chunks_exact(channels)
            .map(|px| convert(px[0]))
            .collect()
    }
}

// `as` casts of floats saturate and map NaN to 0, so no value can overflow here.
fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Full range: 1.0 becomes 65535. (The reference program scales its 8 bit result by 256, see `docs/formats.md`.)
fn to_u16(v: f32) -> u16 {
    (v.clamp(0.0, 1.0) * 65535.0).round() as u16
}

/// The sibling name a file is written under before it is renamed into place.
pub(crate) fn part_path(path: &Path) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(".part");
    PathBuf::from(name)
}

/// Writes through `write` into a temporary file next to `path` and renames it on success.
pub(crate) fn write_atomic<E: From<std::io::Error>>(
    path: &Path,
    write: impl FnOnce(&mut BufWriter<File>) -> Result<(), E>,
) -> Result<(), E> {
    let part = part_path(path);
    let result = (|| {
        let mut out = BufWriter::new(File::create(&part)?);
        write(&mut out)?;
        out.flush()?;
        Ok(())
    })();
    let result = result.and_then(|()| fs::rename(&part, path).map_err(E::from));
    if result.is_err() {
        // Best effort: the temporary file is useless after a failure.
        let _ = fs::remove_file(&part);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fits;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("planets-export-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A small gradient with distinct values per channel and row, so mix-ups show up.
    fn rgb_image() -> Image {
        let (w, h) = (5u32, 3u32);
        let mut data = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let t = (y * w + x) as f32 / (w * h) as f32;
                data.extend([t, 1.0 - t, (x as f32 / (w - 1) as f32) * 0.5]);
            }
        }
        Image {
            width: w,
            height: h,
            channels: 3,
            data,
        }
    }

    fn mono_image() -> Image {
        Image {
            width: 4,
            height: 2,
            channels: 1,
            data: vec![0.0, 0.25, 0.5, 1.0, 1.0, 0.75, 0.125, 0.0],
        }
    }

    fn assert_close(a: &Image, b: &Image, tolerance: f32) {
        assert_eq!(
            (a.width, a.height, a.channels),
            (b.width, b.height, b.channels)
        );
        for (i, (x, y)) in a.data.iter().zip(&b.data).enumerate() {
            assert!((x - y).abs() <= tolerance, "sample {i}: {x} vs {y}");
        }
    }

    #[test]
    fn extension_is_fixed_to_match_the_format() {
        let f = ExportFormat::Tiff16;
        assert_eq!(
            f.with_extension(Path::new("a/jupiter.bmp")),
            Path::new("a/jupiter.tif")
        );
        assert_eq!(
            f.with_extension(Path::new("jupiter")),
            Path::new("jupiter.tif")
        );
        assert_eq!(
            f.with_extension(Path::new("jupiter.TIFF")),
            Path::new("jupiter.TIFF")
        );
        assert_eq!(
            ExportFormat::Fits32Float.with_extension(Path::new("x.fits")),
            Path::new("x.fits")
        );
    }

    #[test]
    fn tiff16_roundtrip_is_exact_to_one_step() {
        let dir = temp_dir("tiff");
        let image = rgb_image();
        let path = save_image(&image, &dir.join("out.tif"), ExportFormat::Tiff16).unwrap();
        let back = image::open(&path).unwrap().to_rgb16();
        assert_eq!(back.dimensions(), (5, 3));
        let expected: Vec<u16> = image
            .data
            .iter()
            .map(|v| (v * 65535.0).round() as u16)
            .collect();
        assert_eq!(back.as_raw(), &expected);
    }

    #[test]
    fn png16_roundtrip_keeps_16_bits() {
        let dir = temp_dir("png");
        let image = rgb_image();
        let path = save_image(&image, &dir.join("out.png"), ExportFormat::Png16).unwrap();
        let decoded = image::open(&path).unwrap();
        assert_eq!(decoded.color(), image::ColorType::Rgb16);
        let back = decoded.to_rgb16();
        let expected: Vec<u16> = image
            .data
            .iter()
            .map(|v| (v * 65535.0).round() as u16)
            .collect();
        assert_eq!(back.as_raw(), &expected);
    }

    #[test]
    fn mono_images_stay_single_channel() {
        let dir = temp_dir("mono");
        let image = mono_image();
        let tif = save_image(&image, &dir.join("m.tif"), ExportFormat::Tiff16).unwrap();
        assert_eq!(image::open(&tif).unwrap().color(), image::ColorType::L16);
        let png = save_image(&image, &dir.join("m.png"), ExportFormat::Png16).unwrap();
        assert_eq!(image::open(&png).unwrap().color(), image::ColorType::L16);
        let bmp = save_image(&image, &dir.join("m.bmp"), ExportFormat::Bmp).unwrap();
        assert_eq!(
            image::open(&bmp).unwrap().to_luma8().get_pixel(1, 0).0[0],
            64
        );
    }

    #[test]
    fn bmp_is_8_bit_rgb() {
        let dir = temp_dir("bmp");
        let image = rgb_image();
        let path = save_image(&image, &dir.join("o.bmp"), ExportFormat::Bmp).unwrap();
        let back = image::open(&path).unwrap().to_rgb8();
        let expected: Vec<u8> = image
            .data
            .iter()
            .map(|v| (v * 255.0).round() as u8)
            .collect();
        assert_eq!(back.as_raw(), &expected);
    }

    #[test]
    fn jpeg_is_close_to_the_source() {
        let dir = temp_dir("jpg");
        // A flat colour survives JPEG almost exactly.
        let image = Image {
            width: 16,
            height: 16,
            channels: 3,
            data: [0.8, 0.4, 0.2].repeat(256),
        };
        let path = save_image(&image, &dir.join("o.jpg"), ExportFormat::Jpeg).unwrap();
        let back = image::open(&path).unwrap().to_rgb8();
        for px in back.pixels() {
            for (got, want) in px.0.iter().zip([204u8, 102, 51]) {
                assert!(got.abs_diff(want) <= 3, "{got} vs {want}");
            }
        }
    }

    #[test]
    fn fits_roundtrip_rgb_and_mono() {
        let dir = temp_dir("fits");
        for (name, image) in [("rgb", rgb_image()), ("mono", mono_image())] {
            for (format, tolerance) in [
                (ExportFormat::Fits16, 0.5 / 65535.0),
                (ExportFormat::Fits32, 1e-6),
                (ExportFormat::Fits32Float, 0.0),
            ] {
                let path = save_image(&image, &dir.join(format!("{name}.fit")), format).unwrap();
                assert_close(&fits::read_image(&path).unwrap(), &image, tolerance + 1e-7);
            }
        }
    }

    #[test]
    fn fits_file_has_whole_blocks_and_flipped_rows() {
        let dir = temp_dir("fits-layout");
        let image = mono_image();
        let path = save_image(&image, &dir.join("l.fit"), ExportFormat::Fits16).unwrap();
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes.len() % 2880, 0);
        assert!(bytes.starts_with(b"SIMPLE  =                    T"));
        // First stored row is the bottom row of the image (0.0 is -32768 in the unsigned convention).
        let data = &bytes[2880..];
        assert_eq!(i16::from_be_bytes([data[0], data[1]]), 32767); // 1.0 -> 65535 - 32768
        assert_eq!(i16::from_be_bytes([data[6], data[7]]), -32768); // 0.0 of the bottom row
    }

    #[test]
    fn out_of_range_and_nan_samples_are_clamped() {
        let dir = temp_dir("clamp");
        let image = Image {
            width: 3,
            height: 1,
            channels: 1,
            data: vec![-1.0, 2.0, f32::NAN],
        };
        let path = save_image(&image, &dir.join("c.fit"), ExportFormat::Fits16).unwrap();
        assert_eq!(fits::read_image(&path).unwrap().data, vec![0.0, 1.0, 0.0]);
        let tif = save_image(&image, &dir.join("c.tif"), ExportFormat::Tiff16).unwrap();
        let back = image::open(&tif).unwrap().to_luma16();
        assert_eq!(back.as_raw(), &vec![0u16, 65535, 0]);
    }

    #[test]
    fn extra_channels_are_dropped() {
        let dir = temp_dir("rgba");
        let image = Image {
            width: 1,
            height: 1,
            channels: 4,
            data: vec![1.0, 0.0, 0.5, 0.2],
        };
        let path = save_image(&image, &dir.join("a.png"), ExportFormat::Png16).unwrap();
        assert_eq!(image::open(&path).unwrap().color(), image::ColorType::Rgb16);
    }

    #[test]
    fn inconsistent_images_are_rejected_and_leave_no_file() {
        let dir = temp_dir("bad");
        let image = Image {
            width: 2,
            height: 2,
            channels: 1,
            data: vec![0.0; 3],
        };
        let path = dir.join("bad.tif");
        assert!(matches!(
            save_image(&image, &path, ExportFormat::Tiff16),
            Err(ExportError::InvalidImage(_))
        ));
        assert!(!path.exists());
        assert!(!part_path(&path).exists());
    }

    #[test]
    fn failed_write_keeps_the_old_file() {
        let dir = temp_dir("keep");
        let path = dir.join("keep.tif");
        fs::write(&path, b"old").unwrap();
        let result: Result<(), ExportError> =
            write_atomic(&path, |_| Err(ExportError::InvalidImage("boom")));
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"old");
        assert!(!part_path(&path).exists());
    }

    #[test]
    fn existing_file_is_replaced() {
        let dir = temp_dir("replace");
        let path = dir.join("r.png");
        fs::write(&path, b"old").unwrap();
        save_image(&mono_image(), &path, ExportFormat::Png16).unwrap();
        assert!(image::open(&path).is_ok());
    }
}
