//! Progress reporting and job control (Cancel and Pause) of the long operations.

use planets_core::align::Control;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// The operations that report progress. *Save Aligned* and *Save Integrated* are one operation,
/// [`Operation::Sequence`], and share Cancel and Pause with [`Operation::Stack`] (the window sends
/// both to the same buttons).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Align,
    Stack,
    Sequence,
    Wavelet,
}

/// Receives the progress of a running operation: `done` of `total` steps (frames, or layers for
/// the wavelets) and the time since the operation started. Called from the calculating thread.
pub trait ProgressSink: Send + Sync {
    fn progress(&self, operation: Operation, done: u32, total: u32, elapsed_ms: u64);
}

impl<F> ProgressSink for F
where
    F: Fn(Operation, u32, u32, u64) + Send + Sync,
{
    fn progress(&self, operation: Operation, done: u32, total: u32, elapsed_ms: u64) {
        self(operation, done, total, elapsed_ms);
    }
}

/// A sink that ignores everything.
pub struct NoProgress;

impl ProgressSink for NoProgress {
    fn progress(&self, _: Operation, _: u32, _: u32, _: u64) {}
}

/// The Cancel and Pause flags of one job kind.
#[derive(Default)]
pub(crate) struct JobControl {
    cancelled: AtomicBool,
    paused: AtomicBool,
}

impl JobControl {
    /// Clears Cancel and Pause before a job starts: a Cancel from before the run must not stop it.
    pub(crate) fn begin(&self) {
        self.cancelled.store(false, Ordering::Relaxed);
        self.paused.store(false, Ordering::Relaxed);
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub(crate) fn pause(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    /// Blocks while paused; a Cancel ends the wait.
    pub(crate) fn wait_while_paused(&self) {
        while self.paused.load(Ordering::Relaxed) && !self.cancelled.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Whether the job goes on, as the core crates ask it between two steps.
    pub(crate) fn control(&self) -> Control {
        if self.cancelled.load(Ordering::Relaxed) {
            Control::Cancel
        } else {
            Control::Continue
        }
    }

    /// Called after each step: waits while paused and tells whether the job goes on.
    pub(crate) fn checkpoint(&self) -> Control {
        self.wait_while_paused();
        self.control()
    }
}
