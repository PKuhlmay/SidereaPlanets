//! The errors of the session. The texts are the ones the window shows, so they must not change
//! without the frontend (`app/src`) and the Mac app agreeing.

use std::{
    fmt::Display,
    sync::{Mutex, MutexGuard},
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SessionError {
    #[error("no recording is open")]
    NoRecording,
    #[error("set alignpoints first")]
    NoAlignpoints,
    #[error("align first")]
    NotAligned,
    #[error("stack first")]
    NotStacked,
    #[error("no image to sharpen")]
    NoImageToSharpen,
    #[error("no image to adjust")]
    NoImageToAdjust,
    #[error("there is no stacked image yet")]
    NoStackedImage,
    #[error("there is no processed image yet")]
    NoProcessedImage,
    #[error("no frames selected")]
    NoFramesSelected,
    #[error("Sort On Quality needs the result of Align")]
    SortNeedsAlign,
    /// A job was stopped by Cancel.
    #[error("cancelled")]
    Cancelled,
    #[error("{bytes} bytes do not fit an image of {width}x{height}")]
    BadImageSize {
        width: u32,
        height: u32,
        bytes: usize,
    },
    #[error("poisoned lock: another task failed inside")]
    Poisoned,
    /// A reader, writer, file or the thread pool failed; the message is complete as it is.
    #[error("{0}")]
    Failed(String),
}

impl SessionError {
    pub(crate) fn failed(error: impl Display) -> Self {
        Self::Failed(error.to_string())
    }
}

/// Locks `mutex`; a lock that another thread poisoned is an error, not a panic.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, SessionError> {
    mutex.lock().map_err(|_| SessionError::Poisoned)
}
