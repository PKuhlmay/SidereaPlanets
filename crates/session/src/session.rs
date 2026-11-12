//! The [`Session`]: everything one open recording and the work built on it consist of.

use crate::{
    SessionError,
    align::AlignState,
    color::ColorState,
    compute::{ComputeState, logical_cores},
    export::ExportState,
    image::RgbaImage,
    progress::Operation,
    stack::StackState,
    wavelet::WaveletState,
};
use planets_core::Image;
use planets_io::recording::{Recording, RecordingInfo as IoInfo};
use serde::Serialize;
use std::{path::PathBuf, sync::Mutex};

/// Summary of an opened recording, as shown in the UI.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingInfo {
    pub format: &'static str,
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
}

impl From<IoInfo> for RecordingInfo {
    fn from(i: IoInfo) -> Self {
        Self {
            format: i.format,
            codec: i.codec,
            width: i.width,
            height: i.height,
            bit_depth: i.bit_depth,
            frame_count: i.frame_count,
            color: i.color,
            fps: i.fps,
            observer: i.observer,
            instrument: i.instrument,
            telescope: i.telescope,
        }
    }
}

/// The state behind the workflow steps (Open, Alignpoints, Align, Limit, Stack, Wavelets, Save):
/// the open recording, the results of every step, the thread pool of the *CPUs* field and the Cancel
/// and Pause flags of the long operations. A user interface owns one session and calls the step
/// methods, which live next to the step they belong to (`align.rs`, `stack.rs`, ...).
///
/// All methods take `&self`: the session can be shared between threads, and Cancel and Pause can
/// be called while another thread runs an operation.
pub struct Session {
    pub(crate) recording: Mutex<Option<Recording>>,
    pub(crate) compute: ComputeState,
    pub(crate) align: AlignState,
    pub(crate) stack: StackState,
    pub(crate) wavelet: WaveletState,
    pub(crate) color: ColorState,
    pub(crate) export: ExportState,
}

impl Session {
    /// A session without a recording that calculates on all logical cores.
    pub fn new() -> Result<Self, SessionError> {
        Ok(Self {
            recording: Mutex::default(),
            compute: ComputeState::with_threads(logical_cores())?,
            align: AlignState::default(),
            stack: StackState::default(),
            wavelet: WaveletState::default(),
            color: ColorState::default(),
            export: ExportState::default(),
        })
    }

    /// Opens the files picked in the dialog: one video, or one or more still frames as a sequence.
    /// The recording replaces the open one; results built on the old one stay until the next
    /// Set Alignpoints (the window starts over at Open).
    pub fn open_recording(&self, paths: &[PathBuf]) -> Result<RecordingInfo, SessionError> {
        let recording = Recording::open_files(paths).map_err(SessionError::failed)?;
        let info = recording.info().into();
        *crate::error::lock(&self.recording)? = Some(recording);
        Ok(info)
    }

    /// One frame of the open recording.
    pub fn read_frame(&self, index: u32) -> Result<RgbaImage, SessionError> {
        Ok(RgbaImage::from_image(&self.read(index)?))
    }

    /// One frame at full precision, for the calculations.
    pub(crate) fn read(&self, index: u32) -> Result<Image, SessionError> {
        let mut guard = crate::error::lock(&self.recording)?;
        let recording = guard.as_mut().ok_or(SessionError::NoRecording)?;
        recording.read_frame(index).map_err(SessionError::failed)
    }

    /// The number of frames of the open recording.
    pub(crate) fn frame_count(&self) -> Result<u32, SessionError> {
        let guard = crate::error::lock(&self.recording)?;
        Ok(guard
            .as_ref()
            .ok_or(SessionError::NoRecording)?
            .info()
            .frame_count)
    }

    /// Forgets everything that was built on the current alignment: the stack, the sharpened image, the colour
    /// adjustment and the two images *Save image* writes. Called when Set Alignpoints, Align or Limit run again, so
    /// that no old image stays around and *Save image* cannot write one.
    pub(crate) fn discard_results(&self) {
        self.stack.clear();
        self.wavelet.clear();
        self.color.clear();
        self.export.clear();
    }

    /// Stops a running operation; it then returns [`SessionError::Cancelled`]. Without a running operation the
    /// Cancel is forgotten when the next one starts.
    pub fn cancel(&self, operation: Operation) {
        self.job(operation).cancel();
    }

    /// Pauses or resumes a running operation (between two frames, for the wavelets between two layers).
    pub fn pause(&self, operation: Operation, paused: bool) {
        self.job(operation).pause(paused);
    }

    fn job(&self, operation: Operation) -> &crate::progress::JobControl {
        match operation {
            Operation::Align => &self.align.control,
            Operation::Stack | Operation::Sequence => &self.stack.control,
            Operation::Wavelet => &self.wavelet.control,
        }
    }

    /// The number of logical cores: the default and the upper end of the *CPUs* field.
    pub fn cpu_count(&self) -> usize {
        logical_cores()
    }

    /// *CPUs*: from now on the calculations use this many threads. Returns the number in use, which differs from
    /// `count` when that was outside 1 to the number of logical cores.
    pub fn set_cpus(&self, count: u32) -> Result<usize, SessionError> {
        self.compute.set_threads(count)
    }

    /// The number of threads the calculations use now.
    pub fn cpus(&self) -> usize {
        self.compute.install(rayon::current_num_threads)
    }
}
