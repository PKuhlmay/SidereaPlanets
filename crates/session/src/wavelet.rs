//! The Wavelet tab: preview, Do All and schemes (`docs/wavelets.md`).

use crate::{
    Session, SessionError,
    compute::ComputeState,
    error::lock,
    export::ExportState,
    image::RgbaImage,
    progress::{JobControl, Operation, ProgressSink},
};
use planets_core::{
    Image,
    wavelet::{self, ColourMode, Previewer, Region, WaveletSettings, rgba_region, scheme},
};
use serde::Serialize;
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

/// What the Wavelet tab works on and what it has produced.
#[derive(Default)]
pub(crate) struct WaveletState {
    previewer: Mutex<Option<Previewer>>,
    processed: Mutex<Option<Arc<Image>>>,
    pub(crate) control: JobControl,
}

impl WaveletState {
    /// Sets the image the tab sharpens (the stacked image) and forgets the previous result.
    /// Called after *Stack*.
    pub(crate) fn set_source(&self, image: Image) -> Result<(), SessionError> {
        *lock(&self.previewer)? = Some(Previewer::new(Arc::new(image)));
        *lock(&self.processed)? = None;
        Ok(())
    }

    /// *Do All* on the source: applies the settings to the whole image and keeps the result, here and in `export`
    /// (for *Save image*). `progress(done, total)` gets the finished layers of the run. A run stopped by
    /// Cancel changes neither the result nor `export`.
    pub(crate) fn do_all(
        &self,
        compute: &ComputeState,
        export: &ExportState,
        settings: &WaveletSettings,
        mode: ColourMode,
        mut progress: impl FnMut(u32, u32) + Send,
    ) -> Result<Arc<Image>, SessionError> {
        let source = {
            let guard = lock(&self.previewer)?;
            guard
                .as_ref()
                .ok_or(SessionError::NoImageToSharpen)?
                .source()
                .clone()
        };
        self.control.begin();
        let result = compute
            .install(|| {
                wavelet::do_all_until(&source, settings, mode, |done, total| {
                    progress(done, total);
                    self.control.checkpoint()
                })
            })
            .ok_or(SessionError::Cancelled)?;
        let result = Arc::new(result);
        export.set_wavelet((*result).clone());
        *lock(&self.processed)? = Some(result.clone());
        Ok(result)
    }

    /// Forgets the image and the result: they were made from an alignment that is gone.
    pub(crate) fn clear(&self) {
        if let Ok(mut previewer) = self.previewer.lock() {
            *previewer = None;
        }
        if let Ok(mut processed) = self.processed.lock() {
            *processed = None;
        }
    }

    #[cfg(test)]
    fn processed(&self) -> Option<Arc<Image>> {
        self.processed.lock().ok().and_then(|p| p.clone())
    }
}

/// Size of the image in the Wavelet tab and of the source of the colour panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SourceInfo {
    pub width: u32,
    pub height: u32,
    /// 1 for a mono image: the colour panels do nothing then.
    pub channels: u8,
}

impl SourceInfo {
    pub(crate) fn of(image: &Image) -> Self {
        Self {
            width: image.width,
            height: image.height,
            channels: image.channels,
        }
    }
}

/// One rendered area of the preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveletPreview {
    region: Region,
    rgba: Vec<u8>,
}

impl WaveletPreview {
    /// The area that was rendered: the requested area cut to the image.
    pub fn region(&self) -> Region {
        self.region
    }

    /// `region.width * region.height * 4` bytes, red, green, blue, alpha.
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    pub fn into_rgba(self) -> Vec<u8> {
        self.rgba
    }
}

/// The settings *Reset Wavelets* restores.
pub fn wavelet_default_settings() -> WaveletSettings {
    WaveletSettings::default()
}

/// *Save Scheme*: writes the settings to a `.rwv` file.
pub fn save_wavelet_scheme(path: &Path, settings: &WaveletSettings) -> Result<(), SessionError> {
    fs::write(path, scheme::to_rwv(settings))
        .map_err(|e| SessionError::Failed(format!("{}: {e}", path.display())))
}

/// *Load Scheme*: reads the settings from a `.rwv` file.
pub fn load_wavelet_scheme(path: &Path) -> Result<WaveletSettings, SessionError> {
    let failed =
        |e: &dyn std::fmt::Display| SessionError::Failed(format!("{}: {e}", path.display()));
    // The original writes ANSI text; the file only holds ASCII, so a lossy conversion is enough.
    let bytes = fs::read(path).map_err(|e| failed(&e))?;
    scheme::from_rwv(&String::from_utf8_lossy(&bytes)).map_err(|e| failed(&e))
}

impl Session {
    /// Uses a frame of the open recording as the image of the tab. A stand-in until *Stack* sets the
    /// real source.
    pub fn wavelet_set_source_frame(&self, index: u32) -> Result<SourceInfo, SessionError> {
        let image = self.read(index)?;
        let source = SourceInfo::of(&image);
        self.wavelet.set_source(image)?;
        Ok(source)
    }

    /// Renders the preview of one area, with the tone curve and colour treatment *Do All* uses. `settings` of `None`
    /// gives the unprocessed image (*Before*).
    pub fn wavelet_preview(
        &self,
        settings: Option<&WaveletSettings>,
        area: Region,
        mode: ColourMode,
    ) -> Result<WaveletPreview, SessionError> {
        let mut guard = lock(&self.wavelet.previewer)?;
        let previewer = guard.as_mut().ok_or(SessionError::NoImageToSharpen)?;
        let preview = self.compute.install(|| match settings {
            Some(settings) => previewer.render(settings, mode, area),
            None => previewer.before(area),
        });
        Ok(WaveletPreview {
            region: preview.region,
            rgba: preview.rgba,
        })
    }

    /// *Do All*: applies the settings to the whole image (tone curve and colour treatment included) and keeps the
    /// result for saving. `progress` gets the finished layers; Cancel ([`Operation::Wavelet`]) ends the run with
    /// [`SessionError::Cancelled`] and leaves the earlier result untouched.
    pub fn wavelet_do_all(
        &self,
        settings: &WaveletSettings,
        mode: ColourMode,
        progress: &dyn ProgressSink,
    ) -> Result<RgbaImage, SessionError> {
        let started = Instant::now();
        let result = self.wavelet.do_all(
            &self.compute,
            &self.export,
            settings,
            mode,
            |done, total| {
                progress.progress(
                    Operation::Wavelet,
                    done,
                    total,
                    started.elapsed().as_millis() as u64,
                );
            },
        )?;
        let rgba = rgba_region(&result, Region::full(result.width, result.height));
        Ok(RgbaImage::new(result.width, result.height, rgba))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ImageSource;
    use std::time::Duration;

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
        let compute = ComputeState::default();
        let mut seen = Vec::new();
        let result = state
            .do_all(&compute, &export, &settings, ColourMode::Colour, |d, t| {
                seen.push((d, t))
            })
            .unwrap();
        assert_eq!(seen, vec![(0, 3), (1, 3), (2, 3), (3, 3)]);
        assert_eq!(*state.processed().unwrap(), *result);
        assert_eq!(*export.get(ImageSource::Wavelet).unwrap(), *result);
    }

    #[test]
    fn do_all_runs_in_the_pool_of_the_cpus_field() {
        let (state, export, settings) = ready();
        // One thread: any machine with more cores shows the difference to the global pool.
        let compute = ComputeState::with_threads(1).unwrap();
        let mut threads = Vec::new();
        state
            .do_all(&compute, &export, &settings, ColourMode::Colour, |_, _| {
                threads.push(rayon::current_num_threads())
            })
            .unwrap();
        assert_eq!(threads, vec![1; 4]);
    }

    #[test]
    fn a_cancelled_do_all_changes_neither_the_result_nor_the_export() {
        let (state, export, settings) = ready();
        let compute = ComputeState::default();
        // An earlier Do All left a result behind.
        let earlier = state
            .do_all(
                &compute,
                &export,
                &WaveletSettings::default(),
                ColourMode::Colour,
                |_, _| {},
            )
            .unwrap();
        let mut layers_done = 0;
        let err = state
            .do_all(
                &compute,
                &export,
                &settings,
                ColourMode::Colour,
                |done, _| {
                    layers_done = done;
                    // Cancel arrives while the first layer is being worked on.
                    if done == 1 {
                        state.control.cancel();
                    }
                },
            )
            .unwrap_err();
        assert_eq!(err, SessionError::Cancelled);
        assert_eq!(layers_done, 1, "no layer is worked on after Cancel");
        assert_eq!(*state.processed().unwrap(), *earlier);
        assert_eq!(*export.get(ImageSource::Wavelet).unwrap(), *earlier);
    }

    #[test]
    fn a_cancel_from_before_the_run_does_not_stop_the_next_one() {
        let (state, export, settings) = ready();
        let compute = ComputeState::default();
        state.control.cancel();
        let run = state.do_all(&compute, &export, &settings, ColourMode::Colour, |_, _| {});
        assert!(run.is_ok());
    }

    #[test]
    fn pause_holds_the_run_until_it_is_resumed() {
        let (state, export, settings) = ready();
        let compute = ComputeState::default();
        let state = Arc::new(state);
        let worker = {
            let state = state.clone();
            std::thread::spawn(move || {
                state.do_all(
                    &compute,
                    &export,
                    &settings,
                    ColourMode::Colour,
                    |done, _| {
                        if done == 1 {
                            state.control.pause(true);
                        }
                    },
                )
            })
        };
        std::thread::sleep(Duration::from_millis(300));
        assert!(!worker.is_finished(), "the run goes on while paused");
        state.control.pause(false);
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
