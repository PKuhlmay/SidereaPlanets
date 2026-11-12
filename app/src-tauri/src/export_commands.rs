//! Commands for saving: *Save image* (Stack and Wavelet tabs), the project file and the file
//! pickers of *Load Scheme* and *Save Scheme* (`docs/formats.md`, sections "Ausgabe" and
//! "Projektdatei").
//!
//! The file dialogs are the ones of the operating system and are opened from here, so the
//! frontend only decides *what* to save; writing is the session's job.

use crate::CommandResult;
use planets_io::{
    export::ExportFormat,
    project::{PROJECT_EXTENSION, Project, with_project_extension},
};
use planets_session::{
    ColorSettings, FormatInfo, ImageSource, LoadedProject, Session, project_load as load_project,
    project_save as write_project,
};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

/// The file types offered by *Save image* in the given tab, in the order of the reference program.
#[tauri::command]
pub fn export_formats(source: ImageSource) -> Vec<FormatInfo> {
    planets_session::export_formats(source)
}

/// Writes the image of `source` to `path` in `format`. The extension is corrected to match the
/// format; the result is the path that was written.
#[tauri::command]
pub async fn save_image(
    source: ImageSource,
    format: ExportFormat,
    path: String,
    adjustments: Option<ColorSettings>,
    session: State<'_, Session>,
) -> CommandResult<String> {
    // The crop and the geometry (Flip and Rotate, Resize Image) belong to the Mac app for now (MAC-016, MAC-017); the Windows app
    // saves the whole image as it is.
    Ok(session.save_image(
        source,
        format,
        Path::new(&path),
        adjustments.as_ref(),
        None,
        None,
    )?)
}

/// Opens the save dialog for `format` and writes the image there. `None` if the user cancels.
/// `suggested_name` is the proposed file name (the reference program proposes the name of the recording) and
/// `directory` the folder the dialog starts in (the folder of the recording).
#[tauri::command]
pub async fn save_image_as(
    app: AppHandle,
    source: ImageSource,
    format: ExportFormat,
    suggested_name: Option<String>,
    directory: Option<String>,
    adjustments: Option<ColorSettings>,
    session: State<'_, Session>,
) -> CommandResult<Option<String>> {
    // Fail before the dialog if there is nothing to save.
    session.require_image(source)?;
    let name = suggested_name.map(|n| format.with_extension(Path::new(&n)));
    let dialog = start_dialog(&app, name.as_deref(), directory.as_deref())
        .set_title("Save image")
        .add_filter(format.label(), format.extensions());
    match pick_save(dialog)? {
        Some(path) => Ok(Some(session.save_image(
            source,
            format,
            &path,
            adjustments.as_ref(),
            None,
            None,
        )?)),
        None => Ok(None),
    }
}

/// Writes the project. The extension `.rkproj` is added when missing; the result is the path
/// that was written.
#[tauri::command]
pub async fn project_save(project: Project, path: String) -> CommandResult<String> {
    Ok(write_project(&project, Path::new(&path))?)
}

/// Opens the save dialog and writes the project there. `None` if the user cancels.
#[tauri::command]
pub async fn project_save_as(
    app: AppHandle,
    project: Project,
    suggested_name: Option<String>,
    directory: Option<String>,
) -> CommandResult<Option<String>> {
    let name = suggested_name.map(|n| with_project_extension(Path::new(&n)));
    let dialog = start_dialog(&app, name.as_deref(), directory.as_deref())
        .set_title("Save project")
        .add_filter("Siderea Planets project", &[PROJECT_EXTENSION]);
    match pick_save(dialog)? {
        Some(path) => Ok(Some(write_project(&project, &path)?)),
        None => Ok(None),
    }
}

/// Reads the project at `path`.
#[tauri::command]
pub async fn project_load(path: String) -> CommandResult<LoadedProject> {
    Ok(load_project(Path::new(&path))?)
}

/// Opens the open dialog and reads the chosen project. `None` if the user cancels.
#[tauri::command]
pub async fn project_open(
    app: AppHandle,
    directory: Option<String>,
) -> CommandResult<Option<LoadedProject>> {
    let dialog = start_dialog(&app, None, directory.as_deref())
        .set_title("Open project")
        .add_filter("Siderea Planets project", &[PROJECT_EXTENSION]);
    let picked = dialog
        .blocking_pick_file()
        .map(|p| p.into_path().map_err(|e| e.to_string()))
        .transpose()?;
    Ok(picked.map(|p| load_project(&p)).transpose()?)
}

/// A dialog that starts in `directory` with `file_name` filled in.
pub(crate) fn start_dialog(
    app: &AppHandle,
    file_name: Option<&Path>,
    directory: Option<&str>,
) -> FileDialogBuilder<tauri::Wry> {
    let mut dialog = app.dialog().file();
    if let Some(name) = file_name.and_then(Path::file_name) {
        dialog = dialog.set_file_name(name.to_string_lossy());
    }
    if let Some(dir) = directory.map(PathBuf::from).filter(|d| d.is_dir()) {
        dialog = dialog.set_directory(dir);
    }
    dialog
}

pub(crate) fn pick_save(dialog: FileDialogBuilder<tauri::Wry>) -> CommandResult<Option<PathBuf>> {
    Ok(dialog
        .blocking_save_file()
        .map(|p| p.into_path().map_err(|e| e.to_string()))
        .transpose()?)
}
