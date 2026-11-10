//! Still-image series (BMP, PNG, TIFF, JPEG and FITS): every file is one frame.

use crate::fits::{self, FitsError};
use image::{ColorType, DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use planets_core::Image;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum ImageSeqError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Decode(#[from] image::ImageError),
    #[error(transparent)]
    Fits(#[from] FitsError),
    #[error("no files selected")]
    NoFiles,
    #[error("'{0}' is not an image or FITS file")]
    NotAnImage(String),
    #[error("frame {index} out of range ({count} frames)")]
    FrameOutOfRange { index: u32, count: u32 },
    #[error("frame {index} is {found}, but the first frame is {expected}")]
    FrameMismatch {
        index: u32,
        expected: String,
        found: String,
    },
}

/// Summary of an image series, taken from the first file.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageSequenceInfo {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub mono: bool,
    pub frame_count: u32,
}

/// Extensions (lower case) that [`ImageSequenceReader`] accepts.
pub const EXTENSIONS: &[&str] = &[
    "bmp", "png", "tif", "tiff", "jpg", "jpeg", "fit", "fits", "fts",
];

/// True if the path has an extension of a supported image or FITS format.
pub fn is_image_file(path: &Path) -> bool {
    extension(path).is_some_and(|e| EXTENSIONS.contains(&e.as_str()))
}

fn is_fits(path: &Path) -> bool {
    extension(path).is_some_and(|e| matches!(e.as_str(), "fit" | "fits" | "fts"))
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
}

/// Sorts paths so that `frame2` comes before `frame10`.
pub fn natural_sort(paths: &mut [PathBuf]) {
    paths.sort_by(|a, b| natural_cmp(&a.to_string_lossy(), &b.to_string_lossy()));
}

/// Splits into runs of digits and non-digits; digit runs compare by value, text ignores case.
fn natural_cmp(a: &str, b: &str) -> Ordering {
    fn chunks(s: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut start = 0;
        let mut prev_digit = None;
        for (i, c) in s.char_indices() {
            let digit = c.is_ascii_digit();
            if prev_digit.is_some_and(|p| p != digit) {
                out.push(&s[start..i]);
                start = i;
            }
            prev_digit = Some(digit);
        }
        if start < s.len() {
            out.push(&s[start..]);
        }
        out
    }
    let (ca, cb) = (chunks(a), chunks(b));
    for (x, y) in ca.iter().zip(&cb) {
        let is_number = |s: &str| s.starts_with(|c: char| c.is_ascii_digit());
        let ord = if is_number(x) && is_number(y) {
            let (xt, yt) = (x.trim_start_matches('0'), y.trim_start_matches('0'));
            // Same value: the run with fewer leading zeros comes first.
            xt.len()
                .cmp(&yt.len())
                .then_with(|| xt.cmp(yt))
                .then_with(|| x.len().cmp(&y.len()))
        } else {
            x.to_lowercase().cmp(&y.to_lowercase())
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    ca.len().cmp(&cb.len()).then_with(|| a.cmp(b))
}

fn describe(width: u32, height: u32, mono: bool) -> String {
    let colour = if mono { "mono" } else { "RGB" };
    format!("{width}x{height} {colour}")
}

fn format_name(format: Option<ImageFormat>) -> &'static str {
    match format {
        Some(ImageFormat::Bmp) => "BMP",
        Some(ImageFormat::Png) => "PNG",
        Some(ImageFormat::Tiff) => "TIFF",
        Some(ImageFormat::Jpeg) => "JPEG",
        _ => "Image",
    }
}

fn bits_per_channel(color: ColorType) -> u8 {
    u8::try_from(color.bits_per_pixel() / u16::from(color.channel_count())).unwrap_or(u8::MAX)
}

/// Reads shape and bit depth of one file without decoding the pixels.
fn probe(path: &Path) -> Result<ImageSequenceInfo, ImageSeqError> {
    let (format, width, height, bit_depth, mono) = if is_fits(path) {
        let i = fits::read_info(path)?;
        ("FITS", i.width, i.height, i.bit_depth, i.channels == 1)
    } else {
        let reader = ImageReader::open(path)?.with_guessed_format()?;
        let format = format_name(reader.format());
        let decoder = reader.into_decoder()?;
        let (width, height) = decoder.dimensions();
        let color = decoder.color_type();
        (
            format,
            width,
            height,
            bits_per_channel(color),
            !color.has_color(),
        )
    };
    Ok(ImageSequenceInfo {
        format,
        width,
        height,
        bit_depth,
        mono,
        frame_count: 0,
    })
}

/// Converts a decoded picture to normalised samples. Alpha is dropped; depths above 8 bit
/// (including float) are quantised to 16 bit.
fn convert(decoded: DynamicImage) -> Image {
    let (width, height) = (decoded.width(), decoded.height());
    let color = decoded.color();
    let normalise = |max: f32| move |v: u16| f32::from(v) / max;
    let (channels, data): (u8, Vec<f32>) = match (color.has_color(), bits_per_channel(color) > 8) {
        (false, false) => (
            1,
            decoded
                .into_luma8()
                .into_raw()
                .into_iter()
                .map(|v| f32::from(v) / 255.0)
                .collect(),
        ),
        (false, true) => (
            1,
            decoded
                .into_luma16()
                .into_raw()
                .into_iter()
                .map(normalise(65535.0))
                .collect(),
        ),
        (true, false) => (
            3,
            decoded
                .into_rgb8()
                .into_raw()
                .into_iter()
                .map(|v| f32::from(v) / 255.0)
                .collect(),
        ),
        (true, true) => (
            3,
            decoded
                .into_rgb16()
                .into_raw()
                .into_iter()
                .map(normalise(65535.0))
                .collect(),
        ),
    };
    Image {
        width,
        height,
        channels,
        data,
    }
}

/// A series of still images or FITS files opened for random frame access. Only the first file
/// is inspected on open; every other file is decoded on demand in [`Self::read_frame`] and must
/// match the first one in size and colour layout (mono or RGB).
pub struct ImageSequenceReader {
    paths: Vec<PathBuf>,
    info: ImageSequenceInfo,
}

impl ImageSequenceReader {
    /// Takes the frames in the given order; use [`natural_sort`] first for file-dialog selections.
    pub fn open(paths: Vec<PathBuf>) -> Result<Self, ImageSeqError> {
        if let Some(bad) = paths.iter().find(|p| !is_image_file(p)) {
            return Err(ImageSeqError::NotAnImage(bad.display().to_string()));
        }
        let first = paths.first().ok_or(ImageSeqError::NoFiles)?;
        let mut info = probe(first)?;
        info.frame_count = u32::try_from(paths.len()).unwrap_or(u32::MAX);
        Ok(Self { paths, info })
    }

    pub fn info(&self) -> &ImageSequenceInfo {
        &self.info
    }

    pub fn read_frame(&self, index: u32) -> Result<Image, ImageSeqError> {
        let path = self
            .paths
            .get(index as usize)
            .ok_or(ImageSeqError::FrameOutOfRange {
                index,
                count: self.info.frame_count,
            })?;
        let frame = if is_fits(path) {
            fits::read_image(path)?
        } else {
            convert(ImageReader::open(path)?.with_guessed_format()?.decode()?)
        };
        let (width, height, mono) = (frame.width, frame.height, frame.channels == 1);
        let expected = &self.info;
        if (width, height, mono) != (expected.width, expected.height, expected.mono) {
            return Err(ImageSeqError::FrameMismatch {
                index,
                expected: describe(expected.width, expected.height, expected.mono),
                found: describe(width, height, mono),
            });
        }
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, ImageFormat, Luma, Rgb, Rgba};
    use std::sync::atomic::{AtomicU32, Ordering};

    fn temp_path(name: &str) -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "planets-img-test-{}-{n}-{name}",
            std::process::id()
        ))
    }

    fn gray16(path: &Path, w: u32, h: u32, data: Vec<u16>) {
        ImageBuffer::<Luma<u16>, _>::from_raw(w, h, data)
            .unwrap()
            .save(path)
            .unwrap();
    }

    fn norm16(v: u16) -> f32 {
        f32::from(v) / 65535.0
    }

    #[test]
    fn sixteen_bit_png_keeps_values_and_row_order() {
        let path = temp_path("g16.png");
        let values: Vec<u16> = (0..12).map(|i| i * 5000).collect();
        gray16(&path, 4, 3, values.clone());
        let reader = ImageSequenceReader::open(vec![path]).unwrap();
        let img = reader.read_frame(0).unwrap();
        assert_eq!((img.width, img.height, img.channels), (4, 3, 1));
        let expect: Vec<f32> = values.into_iter().map(norm16).collect();
        assert_eq!(img.data, expect);
    }

    #[test]
    fn rgb8_png_keeps_channel_order() {
        let path = temp_path("rgb8.png");
        ImageBuffer::<Rgb<u8>, _>::from_raw(2, 1, vec![255, 0, 0, 0, 51, 255])
            .unwrap()
            .save(&path)
            .unwrap();
        let reader = ImageSequenceReader::open(vec![path]).unwrap();
        let img = reader.read_frame(0).unwrap();
        assert_eq!(img.channels, 3);
        assert_eq!(img.data, vec![1.0, 0.0, 0.0, 0.0, 0.2, 1.0]);
    }

    #[test]
    fn alpha_channel_is_dropped() {
        let path = temp_path("rgba.png");
        ImageBuffer::<Rgba<u8>, _>::from_raw(1, 1, vec![255, 0, 255, 10])
            .unwrap()
            .save(&path)
            .unwrap();
        let reader = ImageSequenceReader::open(vec![path]).unwrap();
        let img = reader.read_frame(0).unwrap();
        assert_eq!((img.channels, img.data), (3, vec![1.0, 0.0, 1.0]));
    }

    #[test]
    fn gray_alpha_png_becomes_mono() {
        let path = temp_path("la.png");
        image::GrayAlphaImage::from_raw(2, 1, vec![255, 7, 0, 200])
            .unwrap()
            .save(&path)
            .unwrap();
        let reader = ImageSequenceReader::open(vec![path]).unwrap();
        let img = reader.read_frame(0).unwrap();
        assert_eq!((img.channels, img.data), (1, vec![1.0, 0.0]));
    }

    #[test]
    fn bmp_and_tiff_roundtrip() {
        let bmp = temp_path("rgb.bmp");
        ImageBuffer::<Rgb<u8>, _>::from_raw(
            2,
            2,
            vec![0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255],
        )
        .unwrap()
        .save(&bmp)
        .unwrap();
        let tif = temp_path("g16.tif");
        gray16(&tif, 2, 2, vec![0, 65535, 32768, 1]);

        let reader = ImageSequenceReader::open(vec![bmp]).unwrap();
        assert_eq!(reader.info().format, "BMP");
        let img = reader.read_frame(0).unwrap();
        assert_eq!(
            img.data,
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        );

        let reader = ImageSequenceReader::open(vec![tif]).unwrap();
        assert_eq!(reader.info().format, "TIFF");
        let img = reader.read_frame(0).unwrap();
        assert_eq!(img.data, vec![0.0, 1.0, norm16(32768), norm16(1)]);
    }

    #[test]
    fn jpeg_is_decoded_approximately() {
        let path = temp_path("flat.jpg");
        ImageBuffer::<Rgb<u8>, _>::from_pixel(8, 8, Rgb([200, 100, 50]))
            .save_with_format(&path, ImageFormat::Jpeg)
            .unwrap();
        let reader = ImageSequenceReader::open(vec![path]).unwrap();
        assert_eq!(reader.info().format, "JPEG");
        let img = reader.read_frame(0).unwrap();
        let expect = [200.0 / 255.0, 100.0 / 255.0, 50.0 / 255.0];
        for px in img.data.as_chunks::<3>().0 {
            for (v, e) in px.iter().zip(expect) {
                assert!((v - e).abs() < 0.03, "{v} vs {e}");
            }
        }
    }

    #[test]
    fn info_describes_the_first_file() {
        let a = temp_path("a.png");
        let b = temp_path("b.png");
        gray16(&a, 4, 3, vec![0; 12]);
        gray16(&b, 4, 3, vec![0; 12]);
        let reader = ImageSequenceReader::open(vec![a, b]).unwrap();
        assert_eq!(
            reader.info(),
            &ImageSequenceInfo {
                format: "PNG",
                width: 4,
                height: 3,
                bit_depth: 16,
                mono: true,
                frame_count: 2
            }
        );
    }

    #[test]
    fn rgb8_info_reports_colour_and_depth() {
        let path = temp_path("info-rgb.png");
        ImageBuffer::<Rgb<u8>, _>::from_pixel(3, 2, Rgb([1, 2, 3]))
            .save(&path)
            .unwrap();
        let reader = ImageSequenceReader::open(vec![path]).unwrap();
        let info = reader.info();
        assert_eq!((info.bit_depth, info.mono), (8, false));
    }

    #[test]
    fn frames_are_read_by_index() {
        let a = temp_path("i0.png");
        let b = temp_path("i1.png");
        gray16(&a, 1, 1, vec![0]);
        gray16(&b, 1, 1, vec![65535]);
        let reader = ImageSequenceReader::open(vec![a, b]).unwrap();
        assert_eq!(reader.read_frame(1).unwrap().data, vec![1.0]);
        assert_eq!(reader.read_frame(0).unwrap().data, vec![0.0]);
        assert!(matches!(
            reader.read_frame(2),
            Err(ImageSeqError::FrameOutOfRange { index: 2, count: 2 })
        ));
    }

    #[test]
    fn size_mismatch_is_an_error_at_read_frame() {
        let a = temp_path("m0.png");
        let b = temp_path("m1.png");
        gray16(&a, 2, 2, vec![0; 4]);
        gray16(&b, 3, 2, vec![0; 6]);
        let reader = ImageSequenceReader::open(vec![a, b]).unwrap();
        assert!(reader.read_frame(0).is_ok());
        assert!(matches!(
            reader.read_frame(1),
            Err(ImageSeqError::FrameMismatch { index: 1, .. })
        ));
    }

    #[test]
    fn channel_mismatch_is_an_error() {
        let a = temp_path("c0.png");
        let b = temp_path("c1.png");
        gray16(&a, 1, 1, vec![0]);
        ImageBuffer::<Rgb<u8>, _>::from_pixel(1, 1, Rgb([0, 0, 0]))
            .save(&b)
            .unwrap();
        let reader = ImageSequenceReader::open(vec![a, b]).unwrap();
        assert!(matches!(
            reader.read_frame(1),
            Err(ImageSeqError::FrameMismatch { .. })
        ));
    }

    #[test]
    fn empty_selection_and_foreign_files_are_rejected() {
        assert!(matches!(
            ImageSequenceReader::open(vec![]),
            Err(ImageSeqError::NoFiles)
        ));
        assert!(matches!(
            ImageSequenceReader::open(vec![PathBuf::from("notes.txt")]),
            Err(ImageSeqError::NotAnImage(_))
        ));
    }

    #[test]
    fn fits_files_work_as_frames() {
        // Minimal 2x1 8-bit FITS file.
        let mut bytes = Vec::new();
        for card in [
            "SIMPLE  =                    T",
            "BITPIX  =                    8",
            "NAXIS   =                    2",
            "NAXIS1  =                    2",
            "NAXIS2  =                    1",
            "END",
        ] {
            bytes.extend(format!("{card:<80}").bytes());
        }
        bytes.resize(2880, b' ');
        bytes.extend([0u8, 255]);
        bytes.resize(5760, 0);
        let path = temp_path("frame.FIT");
        std::fs::write(&path, bytes).unwrap();
        let reader = ImageSequenceReader::open(vec![path]).unwrap();
        assert_eq!(reader.info().format, "FITS");
        assert_eq!(reader.read_frame(0).unwrap().data, vec![0.0, 1.0]);
    }

    #[test]
    fn extension_check_is_case_insensitive() {
        for name in [
            "a.PNG", "b.Tif", "c.jpeg", "d.fits", "e.FTS", "f.bmp", "g.fit",
        ] {
            assert!(is_image_file(Path::new(name)), "{name}");
        }
        for name in ["a.ser", "b.avi", "noext", "c.txt"] {
            assert!(!is_image_file(Path::new(name)), "{name}");
        }
    }

    #[test]
    fn natural_sort_orders_numbers_by_value() {
        let mut paths: Vec<PathBuf> = ["f10.png", "f2.png", "f1.png", "F03.png", "f002.png"]
            .iter()
            .map(PathBuf::from)
            .collect();
        natural_sort(&mut paths);
        let names: Vec<_> = paths.iter().map(|p| p.to_str().unwrap()).collect();
        assert_eq!(
            names,
            ["f1.png", "f2.png", "f002.png", "F03.png", "f10.png"]
        );
    }
}
