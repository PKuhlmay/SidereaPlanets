//! One entry point for every supported recording format.

use crate::{
    avi::{AviError, AviReader},
    ser::{SerError, SerReader},
};
use planets_core::Image;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum RecordingError {
    #[error(transparent)]
    Ser(#[from] SerError),
    #[error(transparent)]
    Avi(#[from] AviError),
    #[error("unsupported file type '{0}' (supported: .ser, .avi)")]
    UnknownFormat(String),
}

/// Format-independent summary of a recording.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordingInfo {
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub frame_count: u32,
    pub color: String,
    pub fps: Option<f64>,
    pub observer: Option<String>,
    pub instrument: Option<String>,
    pub telescope: Option<String>,
}

pub enum Recording {
    Ser(SerReader),
    Avi(AviReader),
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
            _ => Err(RecordingError::UnknownFormat(ext)),
        }
    }

    pub fn info(&self) -> RecordingInfo {
        match self {
            Self::Ser(r) => {
                let h = r.header();
                let text = |s: &str| (!s.is_empty()).then(|| s.to_owned());
                RecordingInfo {
                    format: "SER",
                    width: h.width,
                    height: h.height,
                    bit_depth: h.bit_depth,
                    frame_count: h.frame_count,
                    color: format!("{:?}", h.color),
                    fps: None,
                    observer: text(&h.observer),
                    instrument: text(&h.instrument),
                    telescope: text(&h.telescope),
                }
            }
            Self::Avi(r) => {
                let i = r.info();
                RecordingInfo {
                    format: "AVI",
                    width: i.width,
                    height: i.height,
                    bit_depth: 8,
                    frame_count: i.frame_count,
                    color: if i.mono { "Mono" } else { "Rgb" }.to_owned(),
                    fps: (i.fps > 0.0).then_some(i.fps),
                    observer: None,
                    instrument: None,
                    telescope: None,
                }
            }
        }
    }

    pub fn read_frame(&mut self, index: u32) -> Result<Image, RecordingError> {
        Ok(match self {
            Self::Ser(r) => r.read_frame(index)?,
            Self::Avi(r) => r.read_frame(index)?,
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
}
