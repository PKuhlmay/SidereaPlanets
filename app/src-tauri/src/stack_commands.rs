//! Commands for the Stack tab: Stack and fetching the stacked image (`docs/workflow.md`, section 3).

use crate::{CommandResult, Events, pack};
use planets_session::{Operation, Session, StackSettings, Stacked};
use tauri::{AppHandle, State, ipc::Response};

/// Registers the selected frames against the reference frame and stacks them. Reports progress
/// through the `stack-progress` event and stops on [`cancel_stacking`].
#[tauri::command]
pub async fn stack(
    settings: StackSettings,
    app: AppHandle,
    session: State<'_, Session>,
) -> CommandResult<Stacked> {
    Ok(session.stack(&settings, &Events(app))?)
}

/// Returns the stacked image as binary, like `read_frame`.
#[tauri::command]
pub fn stacked_image(session: State<'_, Session>) -> CommandResult<Response> {
    Ok(pack(session.stacked_image()?))
}

/// Stops a running *Stack* (and *Save Aligned* / *Save Integrated*, which share its flags).
#[tauri::command]
pub fn cancel_stacking(session: State<'_, Session>) {
    session.cancel(Operation::Stack);
}

/// Pauses or resumes a running *Stack*.
#[tauri::command]
pub fn pause_stacking(paused: bool, session: State<'_, Session>) {
    session.pause(Operation::Stack, paused);
}
