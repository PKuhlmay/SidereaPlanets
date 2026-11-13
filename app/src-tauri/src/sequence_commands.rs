//! Commands for *Create AVI* in the Stack tab: *Save Aligned* and *Save Integrated*
//! (`docs/formats.md`, section "Create AVI"). Cancel and Pause use the commands of *Stack*.

use crate::{
    CommandResult, Events,
    export_commands::{pick_save, start_dialog},
};
use planets_session::{SavedVideo, SequenceSettings, Session, with_avi_extension};
use std::path::Path;
use tauri::{AppHandle, State};

/// Opens the save dialog for a video. `None` if the user cancels. The file dialog comes before the
/// job so that the time used does not include the time spent choosing a name.
#[tauri::command]
pub async fn choose_video_path(
    app: AppHandle,
    suggested_name: Option<String>,
    directory: Option<String>,
) -> CommandResult<Option<String>> {
    let name = suggested_name.map(|n| with_avi_extension(Path::new(&n)));
    let dialog = start_dialog(&app, name.as_deref(), directory.as_deref())
        .set_title("Save Video")
        .add_filter("AVI video", &["avi"]);
    Ok(pick_save(dialog)?.map(|p| with_avi_extension(&p).to_string_lossy().into_owned()))
}

/// *Save Aligned*: every frame that stays, moved onto the reference frame, as one video frame each.
/// Reports progress through the `sequence-progress` event.
#[tauri::command]
pub async fn save_aligned(
    path: String,
    settings: SequenceSettings,
    app: AppHandle,
    session: State<'_, Session>,
) -> CommandResult<SavedVideo> {
    Ok(session.save_aligned(Path::new(&path), &settings, &Events(app))?)
}

/// *Save Integrated*: the moving average over *Average of n Frames* aligned frames.
#[tauri::command]
pub async fn save_integrated(
    path: String,
    settings: SequenceSettings,
    app: AppHandle,
    session: State<'_, Session>,
) -> CommandResult<SavedVideo> {
    Ok(session.save_integrated(Path::new(&path), &settings, &Events(app))?)
}
