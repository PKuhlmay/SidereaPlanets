//! Measuring the live preview in the real window (`app/src/previewBench.ts`, `docs/wavelets.md`). Both commands do
//! nothing unless the environment says so, so a normal start never reaches them.
//!
//! `PLANETS_PREVIEW_BENCH=<recording>` makes the window open the file, run Do All and move a slider;
//! `PLANETS_PREVIEW_BENCH_OUT=<file>` is where the result goes before the app quits.

use crate::{CommandError, CommandResult};
use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize)]
pub(crate) struct BenchConfig {
    recording: String,
}

/// The recording to measure on, or `null` in a normal run.
#[tauri::command]
pub fn preview_bench_config() -> Option<BenchConfig> {
    std::env::var("PLANETS_PREVIEW_BENCH")
        .ok()
        .filter(|path| !path.is_empty())
        .map(|recording| BenchConfig { recording })
}

/// Writes the result of the measurement to the file named by the environment and quits the app.
#[tauri::command]
pub fn preview_bench_report(report: String, app: AppHandle) -> CommandResult<()> {
    if let Ok(out) = std::env::var("PLANETS_PREVIEW_BENCH_OUT") {
        std::fs::write(&out, report).map_err(|e| CommandError::from(format!("{out}: {e}")))?;
    }
    app.exit(0);
    Ok(())
}
