//! GUI-free session of SidereaPlanets: the state and the logic behind the workflow steps (Open,
//! Alignpoints, Align, Limit, Stack, Wavelets, Save, `docs/workflow.md`).
//!
//! A user interface owns one [`Session`] and calls its methods. The Tauri shell of the Windows app
//! (`app/src-tauri`) and the interface to the Mac app (`crates/ffi`) are thin layers over it: they
//! translate arguments, show images and map [`ProgressSink`] to what their toolkit uses.
//!
//! - Inputs and results are plain types; where the window exchanges them as JSON they carry the
//!   serde attributes of that format (camelCase).
//! - Images leave the session as [`RgbaImage`]: 8-bit RGBA with width and height.
//! - Errors are [`SessionError`]; its texts are the ones shown in the window.
//! - File dialogs, the recording list and everything else that is about windows is not here.

mod align;
mod color;
mod compute;
mod derotate;
mod error;
mod export;
mod geometry;
mod image;
mod progress;
mod sequence;
mod session;
mod stack;
mod wavelet;

pub use align::{
    AlignSettings, Aligned, Alignpoints, FrameRating, Intensity, LimitSettings, Limited, Point,
    PointSettings, RoiSettings,
};
pub use color::{
    ChannelShift, ChannelShifts, ColorMix, ColorSettings, HistogramData, HistogramPoints,
};
pub use compute::logical_cores;
pub use derotate::{DerotationInfo, DerotationSettings};
pub use error::SessionError;
pub use export::{
    FormatInfo, ImageSource, LoadedProject, export_formats, project_load, project_save,
};
pub use image::RgbaImage;
pub use progress::{NoProgress, Operation, ProgressSink};
pub use planets_core::crop::CropRect;
pub use planets_core::derotate::DiscShape;
pub use planets_core::geometry::{Geometry, Orientation, Resampling, Resize, ResizeTarget};
pub use planets_core::planet::{Planet, PoleGeometry, unix_seconds};
pub use sequence::{SavedVideo, SequenceSettings, with_avi_extension};
pub use session::{RecordingInfo, Session};
pub use stack::{PointPosition, StackSettings, Stacked};
pub use wavelet::{
    SourceInfo, WaveletPreview, load_wavelet_scheme, save_wavelet_scheme, wavelet_default_settings,
};
