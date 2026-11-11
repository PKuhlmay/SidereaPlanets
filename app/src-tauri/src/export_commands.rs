//! Commands for saving: *Save image* (Stack and Wavelet tabs), the project file and the file
//! pickers of *Load Scheme* and *Save Scheme* (`docs/formats.md`, sections "Ausgabe" and
//! "Projektdatei").
//!
//! The file dialogs are the ones of the operating system and are opened from here, so the
//! frontend only decides *what* to save. The steps that produce an image hand it over with
//! [`ExportState::set_stack`] and [`ExportState::set_wavelet`].

use planets_core::Image;
use planets_io::{
    export::{ExportFormat, save_image as write_image},
    project::{PROJECT_EXTENSION, Project, with_project_extension},
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

/// The images that *Save image* can write.
#[derive(Default)]
pub struct ExportState {
    stack: Mutex<Option<Arc<Image>>>,
    wavelet: Mutex<Option<Arc<Image>>>,
}

impl ExportState {
    /// Sets the stacked image (result of *Stack*).
    #[allow(dead_code)] // called by the Stack step once it exists (RK-006)
    pub fn set_stack(&self, image: Image) {
        if let Ok(mut slot) = self.stack.lock() {
            *slot = Some(Arc::new(image));
        }
    }

    /// Sets the processed image (result of *Process* or *Do All*).
    #[allow(dead_code)] // called by the Wavelet step once it exists (RK-007)
    pub fn set_wavelet(&self, image: Image) {
        if let Ok(mut slot) = self.wavelet.lock() {
            *slot = Some(Arc::new(image));
        }
    }

    fn get(&self, source: ImageSource) -> Result<Arc<Image>, String> {
        let slot = match source {
            ImageSource::Stack => &self.stack,
            ImageSource::Wavelet => &self.wavelet,
        };
        let guard = slot.lock().map_err(|e| e.to_string())?;
        guard.clone().ok_or_else(|| {
            match source {
                ImageSource::Stack => "there is no stacked image yet",
                ImageSource::Wavelet => "there is no processed image yet",
            }
            .to_owned()
        })
    }
}

/// Which image a save command writes: the tab it is called from.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageSource {
    Stack,
    Wavelet,
}

/// One entry of the file type list.
#[derive(Serialize)]
pub struct FormatInfo {
    id: ExportFormat,
    label: &'static str,
    extension: &'static str,
}

/// The file types offered by *Save image* in the given tab, in the order of the reference program.
#[tauri::command]
pub fn export_formats(source: ImageSource) -> Vec<FormatInfo> {
    let formats = match source {
        ImageSource::Stack => ExportFormat::STACK,
        ImageSource::Wavelet => ExportFormat::WAVELET,
    };
    formats
        .iter()
        .map(|&id| FormatInfo {
            id,
            label: id.label(),
            extension: id.extension(),
        })
        .collect()
}

/// Writes the image of `source` to `path` in `format`. The extension is corrected to match the
/// format; the result is the path that was written.
#[tauri::command]
pub async fn save_image(
    source: ImageSource,
    format: ExportFormat,
    path: String,
    state: State<'_, ExportState>,
) -> Result<String, String> {
    save(&state, source, format, Path::new(&path))
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
    state: State<'_, ExportState>,
) -> Result<Option<String>, String> {
    // Fail before the dialog if there is nothing to save.
    state.get(source)?;
    let name = suggested_name.map(|n| format.with_extension(Path::new(&n)));
    let dialog = start_dialog(&app, name.as_deref(), directory.as_deref())
        .set_title("Save image")
        .add_filter(format.label(), format.extensions());
    match pick_save(dialog)? {
        Some(path) => save(&state, source, format, &path).map(Some),
        None => Ok(None),
    }
}

fn save(
    state: &ExportState,
    source: ImageSource,
    format: ExportFormat,
    path: &Path,
) -> Result<String, String> {
    let image = state.get(source)?;
    write_image(&image, path, format)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

/// Writes the project. The extension `.rkproj` is added when missing; the result is the path
/// that was written.
#[tauri::command]
pub async fn project_save(project: Project, path: String) -> Result<String, String> {
    project
        .save(Path::new(&path))
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

/// Opens the save dialog and writes the project there. `None` if the user cancels.
#[tauri::command]
pub async fn project_save_as(
    app: AppHandle,
    project: Project,
    suggested_name: Option<String>,
    directory: Option<String>,
) -> Result<Option<String>, String> {
    let name = suggested_name.map(|n| with_project_extension(Path::new(&n)));
    let dialog = start_dialog(&app, name.as_deref(), directory.as_deref())
        .set_title("Save project")
        .add_filter("SidereaPlanets project", &[PROJECT_EXTENSION]);
    match pick_save(dialog)? {
        Some(path) => project
            .save(&path)
            .map(|p| Some(p.to_string_lossy().into_owned()))
            .map_err(|e| e.to_string()),
        None => Ok(None),
    }
}

/// A project read from disk.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedProject {
    path: String,
    project: Project,
    /// Recording files that are gone, so the frontend can say which ones to look for.
    missing_files: Vec<String>,
}

fn load(path: &Path) -> Result<LoadedProject, String> {
    let project = Project::load(path).map_err(|e| e.to_string())?;
    Ok(LoadedProject {
        path: path.to_string_lossy().into_owned(),
        missing_files: project.missing_files(),
        project,
    })
}

/// Reads the project at `path`.
#[tauri::command]
pub async fn project_load(path: String) -> Result<LoadedProject, String> {
    load(Path::new(&path))
}

/// Opens the open dialog and reads the chosen project. `None` if the user cancels.
#[tauri::command]
pub async fn project_open(
    app: AppHandle,
    directory: Option<String>,
) -> Result<Option<LoadedProject>, String> {
    let dialog = start_dialog(&app, None, directory.as_deref())
        .set_title("Open project")
        .add_filter("SidereaPlanets project", &[PROJECT_EXTENSION]);
    let picked = dialog
        .blocking_pick_file()
        .map(|p| p.into_path().map_err(|e| e.to_string()))
        .transpose()?;
    picked.map(|p| load(&p)).transpose()
}

/// A dialog that starts in `directory` with `file_name` filled in.
fn start_dialog(
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

fn pick_save(dialog: FileDialogBuilder<tauri::Wry>) -> Result<Option<PathBuf>, String> {
    dialog
        .blocking_save_file()
        .map(|p| p.into_path().map_err(|e| e.to_string()))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use planets_io::images::ImageSequenceReader;

    fn image() -> Image {
        Image {
            width: 2,
            height: 2,
            channels: 1,
            data: vec![0.0, 0.5, 1.0, 0.25],
        }
    }

    #[test]
    fn saving_without_an_image_says_so() {
        let state = ExportState::default();
        let path = std::env::temp_dir().join("planets-cmd-none.png");
        let err = save(&state, ImageSource::Stack, ExportFormat::Png16, &path).unwrap_err();
        assert!(err.contains("no stacked image"));
        let err = save(&state, ImageSource::Wavelet, ExportFormat::Png16, &path).unwrap_err();
        assert!(err.contains("no processed image"));
    }

    #[test]
    fn each_source_writes_its_own_image() {
        let state = ExportState::default();
        state.set_stack(image());
        let mut other = image();
        other.data = vec![1.0; 4];
        state.set_wavelet(other);

        let dir = std::env::temp_dir().join(format!("planets-cmd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let from_stack = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &dir.join("s.bmp"),
        )
        .unwrap();
        // The format wins over the typed extension.
        assert!(from_stack.ends_with("s.tif"));
        let from_wavelet = save(
            &state,
            ImageSource::Wavelet,
            ExportFormat::Tiff16,
            &dir.join("w.tif"),
        )
        .unwrap();
        let read = |p: &str| {
            ImageSequenceReader::open(vec![PathBuf::from(p)])
                .unwrap()
                .read_frame(0)
                .unwrap()
        };
        let a = read(&from_stack);
        let b = read(&from_wavelet);
        for (got, want) in a.data.iter().zip([0.0, 0.5, 1.0, 0.25]) {
            assert!((got - want).abs() < 1e-4, "{got} vs {want}");
        }
        assert_eq!(b.data, vec![1.0; 4]);
    }

    #[test]
    fn wavelet_tab_offers_the_shorter_list() {
        let stack = export_formats(ImageSource::Stack);
        let wavelet = export_formats(ImageSource::Wavelet);
        assert!(stack.len() > wavelet.len());
        assert!(wavelet.iter().all(|w| stack.iter().any(|s| s.id == w.id)));
    }
}
