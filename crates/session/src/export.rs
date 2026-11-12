//! Saving: *Save image* (Stack and Wavelet tabs) and the project file (`docs/formats.md`,
//! sections "Ausgabe" and "Projektdatei").
//!
//! The file dialogs belong to the user interface, which only decides *what* to save. The steps that
//! produce an image hand it over with [`ExportState::set_stack`] and [`ExportState::set_wavelet`].

use crate::{Session, SessionError, color::ColorSettings, error::lock};
use planets_core::{
    Image,
    color::{Adjustments, adjust},
    refine::{RefineSettings, refine as refine_image},
};
use planets_io::{
    export::{ExportFormat, save_image as write_image},
    project::Project,
};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

/// The images that *Save image* can write.
#[derive(Default)]
pub(crate) struct ExportState {
    stack: Mutex<Option<Arc<Image>>>,
    wavelet: Mutex<Option<Arc<Image>>>,
}

impl ExportState {
    /// Sets the stacked image (result of *Stack*). A processed image belongs to the previous stack, so it is
    /// dropped: *Save image* in the Wavelet tab must not write it for the new stack.
    pub(crate) fn set_stack(&self, image: Image) {
        if let Ok(mut slot) = self.stack.lock() {
            *slot = Some(Arc::new(image));
        }
        if let Ok(mut slot) = self.wavelet.lock() {
            *slot = None;
        }
    }

    /// Forgets both images: they were made from an alignment that is gone.
    pub(crate) fn clear(&self) {
        if let Ok(mut slot) = self.stack.lock() {
            *slot = None;
        }
        if let Ok(mut slot) = self.wavelet.lock() {
            *slot = None;
        }
    }

    /// Sets the processed image (result of *Process* or *Do All*).
    pub(crate) fn set_wavelet(&self, image: Image) {
        if let Ok(mut slot) = self.wavelet.lock() {
            *slot = Some(Arc::new(image));
        }
    }

    pub(crate) fn get(&self, source: ImageSource) -> Result<Arc<Image>, SessionError> {
        let (slot, missing) = match source {
            ImageSource::Stack => (&self.stack, SessionError::NoStackedImage),
            ImageSource::Wavelet => (&self.wavelet, SessionError::NoProcessedImage),
        };
        lock(slot)?.clone().ok_or(missing)
    }
}

/// Which image a save call writes: the tab it is called from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageSource {
    Stack,
    Wavelet,
}

/// One entry of the file type list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FormatInfo {
    pub id: ExportFormat,
    pub label: &'static str,
    pub extension: &'static str,
}

/// The file types offered by *Save image* in the given tab, in the order of the reference program.
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

/// Writes the image of `source`; with `adjustments` it is written as the colour and tone panels
/// show it (`docs/formats.md`), not as it came out of Stack or Wavelet. `refine` (*Denoise/Deringing* and *Masking*) finishes the
/// processed image first, against the stack it was made from; it does not apply to the stack itself.
fn save(
    state: &ExportState,
    source: ImageSource,
    format: ExportFormat,
    path: &Path,
    adjustments: Option<&Adjustments>,
    refine: Option<&RefineSettings>,
) -> Result<String, SessionError> {
    let image = state.get(source)?;
    let refined;
    let image = match refine {
        Some(r) if source == ImageSource::Wavelet && !r.is_neutral() => {
            let stack = state.get(ImageSource::Stack).ok();
            refined = refine_image(&image, stack.as_deref(), r);
            &refined
        }
        _ => &*image,
    };
    let adjusted;
    let image = match adjustments {
        Some(a) => {
            adjusted = adjust(image, a);
            &adjusted
        }
        None => image,
    };
    write_image(image, path, format)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(SessionError::failed)
}

/// A project read from disk.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedProject {
    pub path: String,
    pub project: Project,
    /// Recording files that are gone, so the user interface can say which ones to look for.
    pub missing_files: Vec<String>,
}

/// Writes the project. The extension `.rkproj` is added when missing; the result is the path
/// that was written.
pub fn project_save(project: &Project, path: &Path) -> Result<String, SessionError> {
    project
        .save(path)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(SessionError::failed)
}

/// Reads the project at `path`.
pub fn project_load(path: &Path) -> Result<LoadedProject, SessionError> {
    let project = Project::load(path).map_err(SessionError::failed)?;
    Ok(LoadedProject {
        path: path.to_string_lossy().into_owned(),
        missing_files: project.missing_files(),
        project,
    })
}

impl Session {
    /// Fails if there is no image of `source` to save, so that a user interface can fail before it opens a file
    /// dialog.
    pub fn require_image(&self, source: ImageSource) -> Result<(), SessionError> {
        self.export.get(source).map(|_| ())
    }

    /// Writes the image of `source` to `path` in `format`. The extension is corrected to match the
    /// format; the result is the path that was written.
    pub fn save_image(
        &self,
        source: ImageSource,
        format: ExportFormat,
        path: &Path,
        adjustments: Option<&ColorSettings>,
    ) -> Result<String, SessionError> {
        let refine = adjustments.map(|a| a.refine);
        let adjustments = adjustments.map(|a| Adjustments::from(*a));
        self.compute.install(|| {
            save(
                &self.export,
                source,
                format,
                path,
                adjustments.as_ref(),
                refine.as_ref(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use planets_io::images::ImageSequenceReader;
    use std::path::PathBuf;

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
        let err = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Png16,
            &path,
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err, SessionError::NoStackedImage);
        assert!(err.to_string().contains("no stacked image"));
        let err = save(
            &state,
            ImageSource::Wavelet,
            ExportFormat::Png16,
            &path,
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(err, SessionError::NoProcessedImage);
        assert!(err.to_string().contains("no processed image"));
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
            None,
            None,
        )
        .unwrap();
        // The format wins over the typed extension.
        assert!(from_stack.ends_with("s.tif"));
        let from_wavelet = save(
            &state,
            ImageSource::Wavelet,
            ExportFormat::Tiff16,
            &dir.join("w.tif"),
            None,
            None,
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
    fn the_panel_values_are_written_when_given() {
        let state = ExportState::default();
        state.set_stack(image());
        let dir = std::env::temp_dir().join(format!("planets-cmd-adj-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Brightness +0.25 of the range moves every sample up by a quarter.
        let brighter = Adjustments {
            brightness: 0.25,
            ..Adjustments::default()
        };
        let path = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &dir.join("a.tif"),
            Some(&brighter),
            None,
        )
        .unwrap();
        let read = ImageSequenceReader::open(vec![PathBuf::from(path)])
            .unwrap()
            .read_frame(0)
            .unwrap();
        for (got, want) in read.data.iter().zip([0.25, 0.75, 1.0, 0.5]) {
            assert!((got - want).abs() < 1e-4, "{got} vs {want}");
        }
    }

    #[test]
    fn the_refine_settings_finish_the_processed_image_against_the_stack_when_saved() {
        use planets_core::refine::MaskSettings;
        let state = ExportState::default();
        let row = |data: Vec<f32>| Image {
            width: 4,
            height: 1,
            channels: 1,
            data,
        };
        state.set_stack(row(vec![0.05, 0.05, 0.8, 0.8]));
        state.set_wavelet(row(vec![0.9; 4]));
        let refine = RefineSettings {
            mask: MaskSettings {
                enabled: true,
                threshold: 100.0,
                feather: 0.0,
                invert: false,
            },
            ..RefineSettings::default()
        };
        let dir = std::env::temp_dir().join(format!("planets-cmd-ref-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let read = |source, name: &str| {
            let path = save(
                &state,
                source,
                ExportFormat::Tiff16,
                &dir.join(name),
                None,
                Some(&refine),
            )
            .unwrap();
            ImageSequenceReader::open(vec![PathBuf::from(path)])
                .unwrap()
                .read_frame(0)
                .unwrap()
                .data
        };
        let written = read(ImageSource::Wavelet, "r.tif");
        for (got, want) in written.iter().zip([0.05, 0.05, 0.9, 0.9]) {
            assert!((got - want).abs() < 1e-4, "{got} vs {want}");
        }
        // The stack itself is saved as it is.
        let stack = read(ImageSource::Stack, "s.tif");
        for (got, want) in stack.iter().zip([0.05, 0.05, 0.8, 0.8]) {
            assert!((got - want).abs() < 1e-4, "{got} vs {want}");
        }
    }

    #[test]
    fn a_new_stack_drops_the_processed_image_of_the_old_one() {
        let state = ExportState::default();
        state.set_stack(image());
        state.set_wavelet(image());
        assert!(state.get(ImageSource::Wavelet).is_ok());

        state.set_stack(image());
        let err = state.get(ImageSource::Wavelet).unwrap_err();
        assert!(err.to_string().contains("no processed image"), "{err}");
        assert!(state.get(ImageSource::Stack).is_ok());
    }

    #[test]
    fn clearing_forgets_both_images() {
        let state = ExportState::default();
        state.set_stack(image());
        state.set_wavelet(image());
        state.clear();
        let err = state.get(ImageSource::Stack).unwrap_err();
        assert!(err.to_string().contains("no stacked image"), "{err}");
        let err = state.get(ImageSource::Wavelet).unwrap_err();
        assert!(err.to_string().contains("no processed image"), "{err}");
    }

    #[test]
    fn wavelet_tab_offers_the_shorter_list() {
        let stack = export_formats(ImageSource::Stack);
        let wavelet = export_formats(ImageSource::Wavelet);
        assert!(stack.len() > wavelet.len());
        assert!(wavelet.iter().all(|w| stack.iter().any(|s| s.id == w.id)));
    }
}
