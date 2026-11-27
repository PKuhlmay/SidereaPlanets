//! CPU time and memory of the app for the readout at the left end of the status bar. The frontend asks about once a
//! second while the readout is shown and the window is visible, and turns two readings into the load (app/src/logic/processUsage.ts).
//! The measuring (`sysinfo`, Windows and macOS) stays here in the shell; the core (`crates/`) does not need it.

use std::sync::Mutex;

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// The process table of `sysinfo`, kept between readings so that a reading only refreshes this one process. `None` where the
/// system cannot be read; every reading is then `None` as well.
pub struct UsageState {
    system: Option<Mutex<System>>,
    pid: Option<Pid>,
}

impl Default for UsageState {
    fn default() -> Self {
        Self {
            system: System::new().ok().map(Mutex::new),
            pid: sysinfo::get_current_pid().ok(),
        }
    }
}

/// One reading: CPU time used since the start (user plus system) and the memory held (resident set: the working set on Windows).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessUsage {
    cpu_seconds: f64,
    memory_bytes: u64,
}

/// Reads the CPU time and memory of this process; `None` when the system does not tell, the status bar shows a dash then.
#[tauri::command]
pub fn process_usage(state: tauri::State<'_, UsageState>) -> Option<ProcessUsage> {
    read(&state)
}

fn read(state: &UsageState) -> Option<ProcessUsage> {
    let pid = state.pid?;
    let mut system = state.system.as_ref()?.lock().ok()?;
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
    let process = system.process(pid)?;
    Some(ProcessUsage {
        cpu_seconds: process.accumulated_cpu_time() as f64 / 1000.0,
        memory_bytes: process.memory(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_this_process_and_its_cpu_time_grows_with_work() {
        let state = UsageState::default();
        let first = read(&state).expect("a reading of the own process");
        assert!(first.memory_bytes > 0);
        // Some work, so that the second reading has used more CPU time: guards against a platform that reports a constant 0.
        let busy: u64 =
            (0..50_000_000u64).fold(0, |acc, n| acc.wrapping_add(std::hint::black_box(n) * n));
        assert!(busy > 0);
        let second = read(&state).expect("a second reading");
        assert!(second.cpu_seconds > first.cpu_seconds);
    }
}
