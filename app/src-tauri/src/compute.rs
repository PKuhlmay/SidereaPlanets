//! The thread pool that all heavy calculations run in: the *CPUs* field of the toolbar sets its size.
//!
//! The commands wrap their calculation in [`ComputeState::install`]; everything the core crates parallelise with
//! `rayon` inside then uses this pool instead of the global one with all cores.

use rayon::{ThreadPool, ThreadPoolBuilder};
use std::{
    num::NonZeroUsize,
    sync::{Arc, PoisonError, RwLock},
};
use tauri::State;

/// The number of logical cores: the default and the upper end of the *CPUs* field.
pub fn logical_cores() -> usize {
    std::thread::available_parallelism().map_or(1, NonZeroUsize::get)
}

/// A requested thread count as a number from 1 to `max`.
pub fn clamp_cpus(requested: u32, max: usize) -> usize {
    (requested as usize).clamp(1, max.max(1))
}

fn build(threads: usize) -> Result<Arc<ThreadPool>, String> {
    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map(Arc::new)
        .map_err(|e| e.to_string())
}

/// The pool the calculations run in. A run that is already going keeps the pool it started in.
pub struct ComputeState {
    pool: RwLock<Arc<ThreadPool>>,
}

impl ComputeState {
    pub fn with_threads(threads: usize) -> Result<Self, String> {
        Ok(Self {
            pool: RwLock::new(build(threads.max(1))?),
        })
    }

    /// Replaces the pool by one of `requested` threads, clamped to 1 up to the number of logical cores. Returns
    /// the number that is in use now.
    pub fn set_threads(&self, requested: u32) -> Result<usize, String> {
        let threads = clamp_cpus(requested, logical_cores());
        let pool = build(threads)?;
        *self.pool.write().unwrap_or_else(PoisonError::into_inner) = pool;
        Ok(threads)
    }

    /// Runs `op` in the pool and returns its result. Blocks the calling thread until `op` is done, like a
    /// `rayon` call in the global pool does.
    pub fn install<R: Send>(&self, op: impl FnOnce() -> R + Send) -> R {
        let pool = self
            .pool
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        pool.install(op)
    }
}

impl Default for ComputeState {
    /// All logical cores; the original starts with 1 (a deliberate difference, `docs/ui.md`).
    fn default() -> Self {
        // Only fails when the system cannot start threads at all, and then nothing works anyway.
        Self::with_threads(logical_cores()).expect("cannot start the calculation threads")
    }
}

/// The number of logical cores, for the default and the maximum of the *CPUs* field.
#[tauri::command]
pub fn cpu_count() -> usize {
    logical_cores()
}

/// *CPUs*: from now on the calculations use this many threads. Returns the number in use, which differs from
/// `count` when that was outside 1 to the number of logical cores.
#[tauri::command]
pub fn set_cpus(count: u32, state: State<'_, ComputeState>) -> Result<usize, String> {
    state.set_threads(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn the_requested_count_is_clamped_to_one_up_to_the_cores() {
        assert_eq!(clamp_cpus(0, 8), 1);
        assert_eq!(clamp_cpus(1, 8), 1);
        assert_eq!(clamp_cpus(5, 8), 5);
        assert_eq!(clamp_cpus(8, 8), 8);
        assert_eq!(clamp_cpus(9, 8), 8);
        assert_eq!(clamp_cpus(u32::MAX, 8), 8);
        // A machine that reports nothing usable still has one core.
        assert_eq!(clamp_cpus(4, 0), 1);
    }

    #[test]
    fn set_threads_reports_the_clamped_value() {
        let state = ComputeState::default();
        assert_eq!(state.set_threads(0), Ok(1));
        assert_eq!(state.install(rayon::current_num_threads), 1);
        assert_eq!(state.set_threads(u32::MAX), Ok(logical_cores()));
        assert_eq!(state.install(rayon::current_num_threads), logical_cores());
    }

    #[test]
    fn the_default_is_all_cores() {
        let state = ComputeState::default();
        assert_eq!(state.install(rayon::current_num_threads), logical_cores());
    }

    #[test]
    fn install_runs_on_exactly_the_threads_of_the_pool() {
        let state = ComputeState::with_threads(2).unwrap();
        assert_eq!(state.install(rayon::current_num_threads), 2);
        // Work spread over the pool never uses a third thread.
        let threads: HashSet<_> = state.install(|| {
            (0..2000)
                .into_par_iter()
                .map(|_| {
                    std::thread::sleep(std::time::Duration::from_micros(50));
                    std::thread::current().id()
                })
                .collect()
        });
        assert!(threads.len() <= 2, "{} threads were used", threads.len());
    }

    #[test]
    fn a_new_size_applies_to_the_next_run() {
        let state = ComputeState::with_threads(1).unwrap();
        assert_eq!(state.install(rayon::current_num_threads), 1);
        let two = logical_cores().min(2) as u32;
        state.set_threads(two).unwrap();
        assert_eq!(state.install(rayon::current_num_threads), two as usize);
    }
}
