//! The Tauri shell of the Windows app: thin commands over [`planets_session::Session`], which holds all state
//! and logic. A command translates its arguments, calls the session, packs images into binary
//! responses and maps progress to events. Command names, argument names, return shapes and event
//! payloads are the contract with the Vue frontend (`app/src`).

mod align_commands;
mod bench_commands;
mod color_commands;
mod export_commands;
mod geometry_commands;
mod menu;
mod sequence_commands;
mod stack_commands;
mod update_commands;
mod usage_commands;
mod wavelet_commands;

use planets_session::{
    Operation, ProgressSink, RecordingInfo, RgbaImage, Session, SessionError, logical_cores,
};
use serde::{Serialize, Serializer};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State, ipc::Response};

/// The error of a command: the text the window shows. It reaches the frontend as a plain string.
pub(crate) struct CommandError(String);

impl From<SessionError> for CommandError {
    fn from(error: SessionError) -> Self {
        Self(error.to_string())
    }
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        Self(message)
    }
}

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

pub(crate) type CommandResult<T> = Result<T, CommandError>;

/// Progress of the long operations as the events the frontend listens to: `align-progress`,
/// `stack-progress` and `sequence-progress` carry `{ done, total, elapsedMs }`, `wavelet-progress`
/// carries `{ done, total }`.
pub(crate) struct Events(pub(crate) AppHandle);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TimedProgress {
    done: u32,
    total: u32,
    elapsed_ms: u64,
}

#[derive(Serialize, Clone)]
struct LayerProgress {
    done: u32,
    total: u32,
}

impl ProgressSink for Events {
    fn progress(&self, operation: Operation, done: u32, total: u32, elapsed_ms: u64) {
        let timed = TimedProgress {
            done,
            total,
            elapsed_ms,
        };
        let _ = match operation {
            Operation::Align => self.0.emit("align-progress", timed),
            Operation::Stack => self.0.emit("stack-progress", timed),
            Operation::Sequence => self.0.emit("sequence-progress", timed),
            Operation::Wavelet => self
                .0
                .emit("wavelet-progress", LayerProgress { done, total }),
        };
    }
}

/// Packs an image as binary: width (u32 LE), height (u32 LE), then width * height * 4 bytes RGBA.
pub(crate) fn pack(image: RgbaImage) -> Response {
    let (width, height) = (image.width(), image.height());
    let mut out = Vec::with_capacity(8 + image.rgba().len());
    out.extend(width.to_le_bytes());
    out.extend(height.to_le_bytes());
    out.extend(image.into_rgba());
    Response::new(out)
}

/// Opens the files picked in the dialog: one video, or one or more still frames as a sequence.
#[tauri::command]
async fn open_recording(
    paths: Vec<String>,
    session: State<'_, Session>,
) -> CommandResult<RecordingInfo> {
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    Ok(session.open_recording(&paths)?)
}

/// Returns one frame as binary, see [`pack`].
#[tauri::command]
async fn read_frame(index: u32, session: State<'_, Session>) -> CommandResult<Response> {
    Ok(pack(session.read_frame(index)?))
}

/// The number of logical cores, for the default and the maximum of the *CPUs* field.
#[tauri::command]
fn cpu_count() -> usize {
    logical_cores()
}

/// *CPUs*: from now on the calculations use this many threads. Returns the number in use, which differs from
/// `count` when that was outside 1 to the number of logical cores.
#[tauri::command]
fn set_cpus(count: u32, session: State<'_, Session>) -> CommandResult<usize> {
    Ok(session.set_cpus(count)?)
}

/// Files the system asked the app to open (double click or "Open with" in Finder) that the window has not taken yet. The
/// window may not exist when the request comes (the app was started by it), so it asks for them with
/// [`take_opened_files`] once it listens, and again whenever the `files-opened` event says there are new ones.
#[derive(Default)]
struct OpenedFiles(Mutex<Vec<String>>);

/// The files waiting to be opened, oldest first; the list is empty afterwards.
#[tauri::command]
fn take_opened_files(opened: State<'_, OpenedFiles>) -> Vec<String> {
    opened
        .0
        .lock()
        .map(|mut files| std::mem::take(&mut *files))
        .unwrap_or_default()
}

/// Remembers the files of an open request and tells the window.
#[cfg(target_os = "macos")]
fn queue_opened_files(app: &AppHandle, urls: &[tauri::Url]) {
    let paths: Vec<String> = urls
        .iter()
        .filter_map(|url| url.to_file_path().ok())
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    if paths.is_empty() {
        return;
    }
    if let Ok(mut files) = tauri::Manager::state::<OpenedFiles>(app).0.lock() {
        files.extend(paths);
    }
    let _ = app.emit("files-opened", ());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Opens the source code link of About in the browser.
        .plugin(tauri_plugin_opener::init());
    // The updater checks GitHub Releases and installs the new version; the process plugin relaunches the app after it (PLANETS-13).
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());
    let app = builder
        .setup(|app| {
            // The Mac has a menu bar; Windows and Linux keep the window without one.
            if cfg!(target_os = "macos") {
                menu::install(app)?;
            }
            Ok(())
        })
        .manage(OpenedFiles::default())
        .manage(Session::new().expect("cannot start the calculation threads"))
        .manage(usage_commands::UsageState::default())
        .invoke_handler(tauri::generate_handler![
            open_recording,
            read_frame,
            cpu_count,
            set_cpus,
            take_opened_files,
            menu::update_menu,
            update_commands::update_install_supported,
            usage_commands::process_usage,
            align_commands::set_alignpoints,
            align_commands::set_alignpoint_threshold,
            align_commands::align,
            align_commands::cancel_processing,
            align_commands::pause_processing,
            align_commands::limit,
            stack_commands::stack,
            stack_commands::derotation_info,
            stack_commands::stacked_image,
            stack_commands::cancel_stacking,
            stack_commands::pause_stacking,
            bench_commands::preview_bench_config,
            bench_commands::preview_bench_report,
            wavelet_commands::wavelet_default_settings,
            wavelet_commands::wavelet_set_source_frame,
            wavelet_commands::wavelet_preview,
            wavelet_commands::wavelet_do_all,
            wavelet_commands::cancel_wavelet,
            wavelet_commands::pause_wavelet,
            wavelet_commands::save_wavelet_scheme,
            wavelet_commands::load_wavelet_scheme,
            color_commands::set_color_source,
            color_commands::estimate_rgb_align,
            color_commands::apply_colors,
            color_commands::color_histogram,
            color_commands::auto_histogram_points,
            color_commands::refine_mask_overlay,
            geometry_commands::orient_preview,
            export_commands::export_formats,
            export_commands::save_image,
            export_commands::save_image_as,
            export_commands::project_save,
            export_commands::project_save_as,
            export_commands::project_load,
            export_commands::project_open,
            sequence_commands::choose_video_path,
            sequence_commands::save_aligned,
            sequence_commands::save_integrated,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app.run(|handle, event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Opened { urls } = &event {
            queue_opened_files(handle, urls);
        }
        #[cfg(not(target_os = "macos"))]
        let _ = (handle, event);
    });
}
