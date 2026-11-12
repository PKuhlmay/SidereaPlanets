//! The error that crosses the boundary: one variant per [`SessionError`] kind, each carrying the
//! text the session gives it (the one the window shows), so Swift can switch on the kind and show
//! `message` as it is.

use planets_session::SessionError;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum PlanetsError {
    #[error("{message}")]
    NoRecording { message: String },
    #[error("{message}")]
    NoAlignpoints { message: String },
    #[error("{message}")]
    NotAligned { message: String },
    #[error("{message}")]
    NotStacked { message: String },
    #[error("{message}")]
    NoImageToSharpen { message: String },
    #[error("{message}")]
    NoImageToAdjust { message: String },
    #[error("{message}")]
    NoStackedImage { message: String },
    #[error("{message}")]
    NoProcessedImage { message: String },
    #[error("{message}")]
    NoFramesSelected { message: String },
    #[error("{message}")]
    SortNeedsAlign { message: String },
    /// A run was stopped by `cancel`; nothing went wrong.
    #[error("{message}")]
    Cancelled { message: String },
    #[error("{message}")]
    BadImageSize { message: String },
    #[error("{message}")]
    Poisoned { message: String },
    /// A reader, writer, file or the thread pool failed; the message is complete as it is.
    #[error("{message}")]
    Failed { message: String },
}

impl From<SessionError> for PlanetsError {
    fn from(error: SessionError) -> Self {
        let message = error.to_string();
        match error {
            SessionError::NoRecording => Self::NoRecording { message },
            SessionError::NoAlignpoints => Self::NoAlignpoints { message },
            SessionError::NotAligned => Self::NotAligned { message },
            SessionError::NotStacked => Self::NotStacked { message },
            SessionError::NoImageToSharpen => Self::NoImageToSharpen { message },
            SessionError::NoImageToAdjust => Self::NoImageToAdjust { message },
            SessionError::NoStackedImage => Self::NoStackedImage { message },
            SessionError::NoProcessedImage => Self::NoProcessedImage { message },
            SessionError::NoFramesSelected => Self::NoFramesSelected { message },
            SessionError::SortNeedsAlign => Self::SortNeedsAlign { message },
            SessionError::Cancelled => Self::Cancelled { message },
            SessionError::BadImageSize { .. } => Self::BadImageSize { message },
            SessionError::Poisoned => Self::Poisoned { message },
            SessionError::Failed(_) => Self::Failed { message },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_text_survives_the_conversion() {
        let error = PlanetsError::from(SessionError::NotAligned);
        assert_eq!(
            error,
            PlanetsError::NotAligned {
                message: "align first".to_owned()
            }
        );
        assert_eq!(error.to_string(), "align first");
        let error = PlanetsError::from(SessionError::Failed("disk full".to_owned()));
        assert_eq!(error.to_string(), "disk full");
    }
}
