//! Commands for the Align tab: Set Alignpoints, Align and Limit (`docs/workflow.md`, section 2).

use crate::{CommandResult, Events};
use planets_session::{
    AlignSettings, Aligned, Alignpoints, LimitSettings, Limited, Operation, PointSettings, Session,
};
use tauri::{AppHandle, State};

/// Sets alignpoints on one frame, with the default threshold.
#[tauri::command]
pub async fn set_alignpoints(
    frame: u32,
    settings: PointSettings,
    session: State<'_, Session>,
) -> CommandResult<Alignpoints> {
    Ok(session.set_alignpoints(frame, &settings)?)
}

/// Moves the *Threshold* slider.
#[tauri::command]
pub fn set_alignpoint_threshold(
    threshold: usize,
    session: State<'_, Session>,
) -> CommandResult<Alignpoints> {
    Ok(session.set_alignpoint_threshold(threshold)?)
}

/// Aligns all frames to the reference frame and rates them. Reports progress through the
/// `align-progress` event and stops on [`cancel_processing`].
#[tauri::command]
pub async fn align(
    settings: AlignSettings,
    app: AppHandle,
    session: State<'_, Session>,
) -> CommandResult<Aligned> {
    Ok(session.align(&settings, &Events(app))?)
}

/// Stops a running *Align*.
#[tauri::command]
pub fn cancel_processing(session: State<'_, Session>) {
    session.cancel(Operation::Align);
}

/// Pauses or resumes a running *Align*.
#[tauri::command]
pub fn pause_processing(paused: bool, session: State<'_, Session>) {
    session.pause(Operation::Align, paused);
}

/// *Limit*: the frames that stay and the mean shift of every point over them.
#[tauri::command]
pub fn limit(settings: LimitSettings, session: State<'_, Session>) -> CommandResult<Limited> {
    Ok(session.limit(&settings)?)
}
