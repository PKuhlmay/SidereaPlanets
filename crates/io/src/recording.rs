//! One entry point for every supported recording format.

use crate::{
    avi::{AviError, AviReader},
    images::{self, ImageSeqError, ImageSequenceReader},
    ser::{SerError, SerReader},
};
use planets_core::Image;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum RecordingError {
    #[error(transparent)]
    Ser(#[from] SerError),
    #[error(transparent)]
    Avi(#[from] AviError),
    #[error(transparent)]
    Images(#[from] ImageSeqError),
    #[error(
        "unsupported file type '{0}' (supported: .ser, .avi, .bmp, .png, .tif, .jpg, .fit, .fits)"
    )]
    UnknownFormat(String),
    #[error(
        "several files can only be opened together if all are image or FITS files ('{0}' is not)"
    )]
    NotAnImageSeries(String),
}

/// Format-independent summary of a recording.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordingInfo {
    pub format: &'static str,
    /// Video codec of AVI files ("DIB" when uncompressed), shown next to the file name.
    pub codec: Option<String>,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub frame_count: u32,
    pub color: String,
    pub fps: Option<f64>,
    pub observer: Option<String>,
    pub instrument: Option<String>,
    pub telescope: Option<String>,
    /// The file has a time for every frame (the trailer of a SER file).
    pub has_timestamps: bool,
    /// When the recording started, Unix seconds (UTC), if the file says so (SER); see [`start_time_from_file_date`] otherwise.
    pub start_time: Option<f64>,
}

/// A guess at the start of a recording from the modification time of its file: the file was last written when the recording
/// ended, so the start is that minus the length of the recording. `None` without a frame rate or a readable file date.
pub fn start_time_from_file_date(path: &Path, frame_count: u32, fps: Option<f64>) -> Option<f64> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let unix = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs_f64();
    let fps = fps.filter(|&f| f > 0.0)?;
    Some(unix - f64::from(frame_count) / fps)
}

pub enum Recording {
    Ser(SerReader),
    Avi(AviReader),
    /// Still images or FITS files, one file per frame.
    Images(ImageSequenceReader),
}

impl Recording {
    /// Picks the reader by file extension (case-insensitive).
    pub fn open(path: &Path) -> Result<Self, RecordingError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match ext.as_str() {
            "ser" => Ok(Self::Ser(SerReader::open(path)?)),
            "avi" => Ok(Self::Avi(AviReader::open(path)?)),
            _ if images::is_image_file(path) => Ok(Self::Images(ImageSequenceReader::open(vec![
                path.to_path_buf(),
            ])?)),
            _ => Err(RecordingError::UnknownFormat(ext)),
        }
    }

    /// Opens a selection from a file dialog. A single path behaves like [`Recording::open`];
    /// several paths must all be image or FITS files and become one frame series in natural
    /// order (`frame2` before `frame10`). Formats may be mixed as long as every frame has the
    /// size and channel count of the first one (checked per frame by `read_frame`).
    pub fn open_files(paths: &[PathBuf]) -> Result<Self, RecordingError> {
        match paths {
            [] => Err(ImageSeqError::NoFiles.into()),
            [single] => Self::open(single),
            many => {
                if let Some(bad) = many.iter().find(|p| !images::is_image_file(p)) {
                    return Err(RecordingError::NotAnImageSeries(bad.display().to_string()));
                }
                let mut sorted = many.to_vec();
                images::natural_sort(&mut sorted);
                Ok(Self::Images(ImageSequenceReader::open(sorted)?))
            }
        }
    }

    pub fn info(&self) -> RecordingInfo {
        match self {
            Self::Ser(r) => {
                let h = r.header();
                let text = |s: &str| (!s.is_empty()).then(|| s.to_owned());
                RecordingInfo {
                    format: "SER",
                    codec: None,
                    width: h.width,
                    height: h.height,
                    bit_depth: h.bit_depth,
                    frame_count: h.frame_count,
                    color: format!("{:?}", h.color),
                    fps: None,
                    observer: text(&h.observer),
                    instrument: text(&h.instrument),
                    telescope: text(&h.telescope),
                    has_timestamps: r.timestamps().is_some(),
                    start_time: r.start_time(),
                }
            }
            Self::Avi(r) => {
                let i = r.info();
                RecordingInfo {
                    format: "AVI",
                    codec: Some(i.codec.clone()),
                    width: i.width,
                    height: i.height,
                    bit_depth: 8,
                    frame_count: i.frame_count,
                    color: if i.mono { "Mono" } else { "Rgb" }.to_owned(),
                    fps: (i.fps > 0.0).then_some(i.fps),
                    observer: None,
                    instrument: None,
                    telescope: None,
                    has_timestamps: false,
                    start_time: None,
                }
            }
            Self::Images(r) => {
                let i = r.info();
                RecordingInfo {
                    format: i.format,
                    codec: None,
                    width: i.width,
                    height: i.height,
                    bit_depth: i.bit_depth,
                    frame_count: i.frame_count,
                    color: if i.mono { "Mono" } else { "Rgb" }.to_owned(),
                    fps: None,
                    observer: None,
                    instrument: None,
                    telescope: None,
                    has_timestamps: false,
                    start_time: None,
                }
            }
        }
    }

    /// The time of every frame in Unix seconds (UTC), if the file holds them (SER trailer).
    pub fn frame_timestamps(&self) -> Option<&[f64]> {
        match self {
            Self::Ser(r) => r.timestamps(),
            _ => None,
        }
    }

    pub fn read_frame(&mut self, index: u32) -> Result<Image, RecordingError> {
        Ok(match self {
            Self::Ser(r) => r.read_frame(index)?,
            Self::Avi(r) => r.read_frame(index)?,
            Self::Images(r) => r.read_frame(index)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_extension_is_rejected() {
        let err = Recording::open(Path::new("capture.mov")).err().unwrap();
        assert!(matches!(err, RecordingError::UnknownFormat(ext) if ext == "mov"));
    }

    #[test]
    fn extension_is_matched_case_insensitively() {
        // The original tools write upper-case names such as JUPITER.AVI; the file does not exist,
        // so the error must come from opening it, not from format detection.
        let err = Recording::open(Path::new("does-not-exist.AVI"))
            .err()
            .unwrap();
        assert!(matches!(err, RecordingError::Avi(AviError::Io(_))));
    }

    fn temp_png(name: &str, value: u8) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("planets-rec-test-{}-{name}", std::process::id()));
        image::GrayImage::from_pixel(2, 2, image::Luma([value]))
            .save(&path)
            .unwrap();
        path
    }

    #[test]
    fn single_image_opens_as_sequence_of_one() {
        let rec = Recording::open(&temp_png("single.png", 255)).unwrap();
        let info = rec.info();
        assert_eq!(
            (info.format, info.frame_count, info.fps, info.bit_depth),
            ("PNG", 1, None, 8)
        );
        assert_eq!(
            (info.width, info.height, info.color.as_str()),
            (2, 2, "Mono")
        );
    }

    #[test]
    fn selection_is_sorted_naturally() {
        let paths = [
            temp_png("f10.png", 100),
            temp_png("f2.png", 20),
            temp_png("f1.png", 10),
        ];
        let mut rec = Recording::open_files(&paths).unwrap();
        assert_eq!(rec.info().frame_count, 3);
        let firsts: Vec<f32> = (0..3)
            .map(|i| rec.read_frame(i).unwrap().data[0] * 255.0)
            .map(f32::round)
            .collect();
        assert_eq!(firsts, vec![10.0, 20.0, 100.0]);
    }

    #[test]
    fn empty_selection_is_rejected() {
        let err = Recording::open_files(&[]).err().unwrap();
        assert!(matches!(
            err,
            RecordingError::Images(ImageSeqError::NoFiles)
        ));
    }

    #[test]
    fn single_video_path_goes_through_open() {
        let err = Recording::open_files(&[PathBuf::from("does-not-exist.ser")])
            .err()
            .unwrap();
        assert!(matches!(err, RecordingError::Ser(SerError::Io(_))));
    }

    #[test]
    fn videos_cannot_be_mixed_with_images() {
        let paths = [temp_png("mix.png", 1), PathBuf::from("video.ser")];
        let err = Recording::open_files(&paths).err().unwrap();
        assert!(matches!(err, RecordingError::NotAnImageSeries(p) if p == "video.ser"));
    }

    #[test]
    fn the_start_is_guessed_from_the_file_date_minus_the_length() {
        let path = temp_png("dated.png", 1);
        let end = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(end)
            .unwrap();
        // 500 frames at 25 per second is 20 seconds.
        let start = start_time_from_file_date(&path, 500, Some(25.0)).unwrap();
        assert!((start - 1_699_999_980.0).abs() < 1e-6, "{start}");
        // Without a frame rate there is no length to take off.
        assert_eq!(start_time_from_file_date(&path, 500, None), None);
        assert_eq!(
            start_time_from_file_date(Path::new("missing.avi"), 5, Some(25.0)),
            None
        );
    }
}
