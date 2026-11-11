//! Commands for the Wavelet tab: preview, Do All and schemes (`docs/wavelets.md`).

use crate::{AppState, export_commands::ExportState};
use planets_core::{
    Image,
    align::Control,
    wavelet::{self, ColourMode, Previewer, Region, WaveletSettings, rgba_region, scheme},
};
use serde::Serialize;
use std::{
    fs,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State, ipc::Response};

/// What the Wavelet tab works on and what it has produced.
#[derive(Default)]
pub struct WaveletState {
    previewer: Mutex<Option<Previewer>>,
    processed: Mutex<Option<Arc<Image>>>,
    cancelled: AtomicBool,
    paused: AtomicBool,
}

impl WaveletState {
    /// Sets the image the tab sharpens (the stacked image) and forgets the previous result.
    /// Called by the `stack` command after *Stack*.
    pub fn set_source(&self, image: Image) -> Result<(), String> {
        *self.previewer.lock().map_err(|e| e.to_string())? = Some(Previewer::new(Arc::new(image)));
        *self.processed.lock().map_err(|e| e.to_string())? = None;
        Ok(())
    }

    /// *Do All* on the source: applies the settings to the whole image and keeps the result, here and in `export`
    /// (for *Save image*). `progress(done, total)` gets the finished layers of the run. A run stopped by
    /// [`cancel_wavelet`] changes neither the result nor `export`.
    fn do_all(
        &self,
        export: &ExportState,
        settings: &WaveletSettings,
        mode: ColourMode,
        mut progress: impl FnMut(u32, u32),
    ) -> Result<Arc<Image>, String> {
        let source = {
            let guard = self.previewer.lock().map_err(|e| e.to_string())?;
            guard
                .as_ref()
                .ok_or("no image to sharpen")?
                .source()
                .clone()
        };
        self.begin();
        let result = wavelet::do_all_until(&source, settings, mode, |done, total| {
            progress(done, total);
            self.checkpoint()
        })
        .ok_or("cancelled")?;
        let result = Arc::new(result);
        export.set_wavelet((*result).clone());
        *self.processed.lock().map_err(|e| e.to_string())? = Some(result.clone());
        Ok(result)
    }

    /// Clears Cancel and Pause before a *Do All* starts.
    fn begin(&self) {
        self.cancelled.store(false, Ordering::Relaxed);
        self.paused.store(false, Ordering::Relaxed);
    }

    /// Called between the layers: waits while paused and tells whether the run goes on.
    fn checkpoint(&self) -> Control {
        while self.paused.load(Ordering::Relaxed) && !self.cancelled.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(50));
        }
        if self.cancelled.load(Ordering::Relaxed) {
            Control::Cancel
        } else {
            Control::Continue
        }
    }

    /// Forgets the image and the result: they were made from an alignment that is gone.
    pub fn clear(&self) {
        if let Ok(mut previewer) = self.previewer.lock() {
            *previewer = None;
        }
        if let Ok(mut processed) = self.processed.lock() {
            *processed = None;
        }
    }

    /// The result of the last *Do All*, for *Save image* (RK-009).
    #[allow(dead_code)] // first caller is the export command of RK-009
    pub fn processed(&self) -> Option<Arc<Image>> {
        self.processed.lock().ok().and_then(|p| p.clone())
    }
}

/// Size of the image in the Wavelet tab.
#[derive(Serialize)]
pub struct SourceInfo {
    width: u32,
    height: u32,
    channels: u8,
}

fn info(image: &Image) -> SourceInfo {
    SourceInfo {
        width: image.width,
        height: image.height,
        channels: image.channels,
    }
}

/// The settings *Reset Wavelets* restores.
#[tauri::command]
pub fn wavelet_default_settings() -> WaveletSettings {
    WaveletSettings::default()
}

/// Uses a frame of the open recording as the image of the tab. A stand-in until *Stack* sets the
/// real source through [`WaveletState::set_source`].
#[tauri::command]
pub async fn wavelet_set_source_frame(
    index: u32,
    recording: State<'_, AppState>,
    state: State<'_, WaveletState>,
) -> Result<SourceInfo, String> {
    let image = {
        let mut guard = recording.0.lock().map_err(|e| e.to_string())?;
        let recording = guard.as_mut().ok_or("no recording is open")?;
        recording.read_frame(index).map_err(|e| e.to_string())?
    };
    let source = info(&image);
    state.set_source(image)?;
    Ok(source)
}

/// Renders the preview of one area, with the tone curve and colour treatment *Do All* uses. `settings` of `null` gives
/// the unprocessed image (*Before*).
///
/// Returns binary: x, y, width, height of the area that was rendered (4 x u32 LE; the requested area cut to
/// the image), then width * height * 4 bytes RGBA.
#[tauri::command]
pub async fn wavelet_preview(
    settings: Option<WaveletSettings>,
    area: Region,
    mode: ColourMode,
    state: State<'_, WaveletState>,
) -> Result<Response, String> {
    let mut guard = state.previewer.lock().map_err(|e| e.to_string())?;
    let previewer = guard.as_mut().ok_or("no image to sharpen")?;
    let preview = match settings {
        Some(settings) => previewer.render(&settings, mode, area),
        None => previewer.before(area),
    };
    let r = preview.region;
    let mut out = Vec::with_capacity(16 + preview.rgba.len());
    for v in [r.x, r.y, r.width, r.height] {
        out.extend(v.to_le_bytes());
    }
    out.extend(preview.rgba);
    Ok(Response::new(out))
}

/// *Do All*: applies the settings to the whole image (tone curve and colour treatment included) and keeps the result
/// for saving.
///
/// Returns binary like `read_frame`: width (u32 LE), height (u32 LE), then width * height * 4 bytes RGBA.
#[tauri::command]
pub async fn wavelet_do_all(
    settings: WaveletSettings,
    mode: ColourMode,
    app: AppHandle,
    state: State<'_, WaveletState>,
) -> Result<Response, String> {
    let result = state.do_all(
        &app.state::<ExportState>(),
        &settings,
        mode,
        |done, total| {
            let _ = app.emit("wavelet-progress", Progress { done, total });
        },
    )?;
    let rgba = rgba_region(&result, Region::full(result.width, result.height));
    let mut out = Vec::with_capacity(8 + rgba.len());
    out.extend(result.width.to_le_bytes());
    out.extend(result.height.to_le_bytes());
    out.extend(rgba);
    Ok(Response::new(out))
}

/// Layers of a *Do All* that are finished, for the progress bar.
#[derive(Serialize, Clone)]
struct Progress {
    done: u32,
    total: u32,
}

/// Stops a running *Do All*; it then returns the error `cancelled` and leaves the earlier result untouched.
#[tauri::command]
pub fn cancel_wavelet(state: State<'_, WaveletState>) {
    state.cancelled.store(true, Ordering::Relaxed);
}

/// Pauses or resumes a running *Do All* between two layers.
#[tauri::command]
pub fn pause_wavelet(paused: bool, state: State<'_, WaveletState>) {
    state.paused.store(paused, Ordering::Relaxed);
}

/// *Save Scheme*: writes the settings to a `.rwv` file.
#[tauri::command]
pub fn save_wavelet_scheme(path: String, settings: WaveletSettings) -> Result<(), String> {
    fs::write(&path, scheme::to_rwv(&settings)).map_err(|e| format!("{path}: {e}"))
}

/// *Load Scheme*: reads the settings from a `.rwv` file.
#[tauri::command]
pub fn load_wavelet_scheme(path: String) -> Result<WaveletSettings, String> {
    // The original writes ANSI text; the file only holds ASCII, so a lossy conversion is enough.
    let bytes = fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
    scheme::from_rwv(&String::from_utf8_lossy(&bytes)).map_err(|e| format!("{path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_commands::ImageSource;

    fn image() -> Image {
        Image::new(4, 4, 1)
    }

    /// A state with a source image and settings that need three layers.
    fn ready() -> (WaveletState, ExportState, WaveletSettings) {
        let state = WaveletState::default();
        let mut source = Image::new(16, 16, 1);
        for (i, v) in source.data.iter_mut().enumerate() {
            *v = (i % 7) as f32 / 7.0;
        }
        state.set_source(source).unwrap();
        let mut settings = WaveletSettings::default();
        settings.layers[2].gain = 3.0;
        (state, ExportState::default(), settings)
    }

    #[test]
    fn do_all_keeps_the_result_for_the_wavelet_tab_and_for_saving() {
        let (state, export, settings) = ready();
        let mut seen = Vec::new();
        let result = state
            .do_all(&export, &settings, ColourMode::Colour, |d, t| {
                seen.push((d, t))
            })
            .unwrap();
        assert_eq!(seen, vec![(0, 3), (1, 3), (2, 3), (3, 3)]);
        assert_eq!(*state.processed().unwrap(), *result);
        assert_eq!(*export.get(ImageSource::Wavelet).unwrap(), *result);
    }

    #[test]
    fn a_cancelled_do_all_changes_neither_the_result_nor_the_export() {
        let (state, export, settings) = ready();
        // An earlier Do All left a result behind.
        let earlier = state
            .do_all(
                &export,
                &WaveletSettings::default(),
                ColourMode::Colour,
                |_, _| {},
            )
            .unwrap();
        let mut layers_done = 0;
        let err = state
            .do_all(&export, &settings, ColourMode::Colour, |done, _| {
                layers_done = done;
                // Cancel arrives while the first layer is being worked on.
                if done == 1 {
                    state.cancelled.store(true, Ordering::Relaxed);
                }
            })
            .unwrap_err();
        assert_eq!(err, "cancelled");
        assert_eq!(layers_done, 1, "no layer is worked on after Cancel");
        assert_eq!(*state.processed().unwrap(), *earlier);
        assert_eq!(*export.get(ImageSource::Wavelet).unwrap(), *earlier);
    }

    #[test]
    fn a_cancel_from_before_the_run_does_not_stop_the_next_one() {
        let (state, export, settings) = ready();
        state.cancelled.store(true, Ordering::Relaxed);
        let run = state.do_all(&export, &settings, ColourMode::Colour, |_, _| {});
        assert!(run.is_ok());
    }

    #[test]
    fn pause_holds_the_run_until_it_is_resumed() {
        let (state, export, settings) = ready();
        let state = Arc::new(state);
        let worker = {
            let state = state.clone();
            std::thread::spawn(move || {
                state.do_all(&export, &settings, ColourMode::Colour, |done, _| {
                    if done == 1 {
                        state.paused.store(true, Ordering::Relaxed);
                    }
                })
            })
        };
        std::thread::sleep(Duration::from_millis(300));
        assert!(!worker.is_finished(), "the run goes on while paused");
        state.paused.store(false, Ordering::Relaxed);
        assert!(worker.join().unwrap().is_ok());
    }

    #[test]
    fn clearing_forgets_the_source_and_the_result() {
        let state = WaveletState::default();
        state.set_source(image()).unwrap();
        *state.processed.lock().unwrap() = Some(Arc::new(image()));
        state.clear();
        assert!(state.previewer.lock().unwrap().is_none());
        assert!(state.processed().is_none());
    }
}
