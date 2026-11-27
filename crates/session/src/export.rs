//! Saving: *Save image* (Stack and Wavelet tabs) and the project file (`docs/formats.md`,
//! sections "Ausgabe" and "Projektdatei").
//!
//! The file dialogs belong to the user interface, which only decides *what* to save. The steps that
//! produce an image hand it over with [`ExportState::set_stack`] and [`ExportState::set_wavelet`].

use crate::{Session, SessionError, color::ColorSettings, error::lock};
use planets_core::{
    Image,
    color::{Adjustments, adjust},
    crop::CropRect,
    geometry::Geometry,
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

/// What *Save image* does to the image before it is written, in this order: `refine`, `adjustments`, the orientation of `geometry`,
/// `crop`, the resize of `geometry`.
#[derive(Default)]
struct SaveOptions<'a> {
    /// The colour and tone panels: the image is written as they show it (`docs/formats.md`), not as it came out of Stack or Wavelet.
    adjustments: Option<&'a Adjustments>,
    /// *Denoise/Deringing* and *Masking* finish the processed image first, against the stack it was made from; they do not apply to
    /// the stack itself.
    refine: Option<&'a RefineSettings>,
    /// Only that part of the image is written, in pixels of the oriented image (what the window shows). It is cut after the colours, so
    /// the edge pixels are treated like all others.
    crop: Option<CropRect>,
    /// *Flip and Rotate* (before the crop) and *Resize Image* (after it).
    geometry: Option<&'a Geometry>,
}

/// Writes the image of `source` as `options` say.
fn save(
    state: &ExportState,
    source: ImageSource,
    format: ExportFormat,
    path: &Path,
    options: &SaveOptions,
) -> Result<String, SessionError> {
    let image = state.get(source)?;
    let refined;
    let image = match options.refine {
        Some(r) if source == ImageSource::Wavelet && !r.is_neutral() => {
            let stack = state.get(ImageSource::Stack).ok();
            refined = refine_image(&image, stack.as_deref(), r);
            &refined
        }
        _ => &*image,
    };
    let adjusted;
    let image = match options.adjustments {
        Some(a) => {
            adjusted = adjust(image, a);
            &adjusted
        }
        None => image,
    };
    let finished;
    let image = if options.crop.is_some() || options.geometry.is_some() {
        finished = options
            .geometry
            .copied()
            .unwrap_or_default()
            .finish(image, options.crop)
            .ok_or_else(|| SessionError::failed("the crop rectangle is outside the image"))?;
        &finished
    } else {
        image
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

/// Writes the project. The extension `.sidereaplanets` is added when missing; the result is the path
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

    /// Writes the image of `source` to `path` in `format`; with `crop` only that part of it. The extension is
    /// corrected to match the format; the result is the path that was written. The order is refine, colours, orientation (in
    /// `geometry`), crop, resize (in `geometry`): the crop rectangle is in pixels of the oriented image.
    pub fn save_image(
        &self,
        source: ImageSource,
        format: ExportFormat,
        path: &Path,
        adjustments: Option<&ColorSettings>,
        crop: Option<CropRect>,
        geometry: Option<&Geometry>,
    ) -> Result<String, SessionError> {
        let refine = adjustments.map(|a| a.refine);
        let adjustments = adjustments.map(|a| Adjustments::from(*a));
        self.compute.install(|| {
            save(
                &self.export,
                source,
                format,
                path,
                &SaveOptions {
                    adjustments: adjustments.as_ref(),
                    refine: refine.as_ref(),
                    crop,
                    geometry,
                },
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
            &SaveOptions::default(),
        )
        .unwrap_err();
        assert_eq!(err, SessionError::NoStackedImage);
        assert!(err.to_string().contains("no stacked image"));
        let err = save(
            &state,
            ImageSource::Wavelet,
            ExportFormat::Png16,
            &path,
            &SaveOptions::default(),
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
            &SaveOptions::default(),
        )
        .unwrap();
        // The format wins over the typed extension.
        assert!(from_stack.ends_with("s.tif"));
        let from_wavelet = save(
            &state,
            ImageSource::Wavelet,
            ExportFormat::Tiff16,
            &dir.join("w.tif"),
            &SaveOptions::default(),
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
            &SaveOptions {
                adjustments: Some(&brighter),
                ..SaveOptions::default()
            },
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

    fn read_back(path: &str) -> Image {
        ImageSequenceReader::open(vec![PathBuf::from(path)])
            .unwrap()
            .read_frame(0)
            .unwrap()
    }

    /// A 4 x 3 gray image whose samples are 0/12 .. 11/12, so every pixel is told apart.
    fn ramp() -> Image {
        Image {
            width: 4,
            height: 3,
            channels: 1,
            data: (0..12).map(|i| i as f32 / 12.0).collect(),
        }
    }

    #[test]
    fn a_crop_writes_only_that_part_of_the_image() {
        let state = ExportState::default();
        state.set_stack(ramp());
        let dir = std::env::temp_dir().join(format!("planets-cmd-crop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let crop = CropRect {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        };
        let path = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &dir.join("c.tif"),
            &SaveOptions {
                crop: Some(crop),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        let out = read_back(&path);
        assert_eq!((out.width, out.height), (2, 2));
        for (got, want) in out.data.iter().zip([5.0, 6.0, 9.0, 10.0]) {
            assert!((got - want / 12.0).abs() < 1e-4, "{got} vs {}", want / 12.0);
        }
    }

    #[test]
    fn a_crop_over_the_edge_is_cut_at_the_edge_and_the_full_image_is_unchanged() {
        let state = ExportState::default();
        state.set_stack(ramp());
        let dir = std::env::temp_dir().join(format!("planets-cmd-crop2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let over = CropRect {
            x: 3,
            y: 2,
            width: 50,
            height: 50,
        };
        let path = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &dir.join("o.tif"),
            &SaveOptions {
                crop: Some(over),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        let out = read_back(&path);
        assert_eq!((out.width, out.height), (1, 1));
        assert!((out.data[0] - 11.0 / 12.0).abs() < 1e-4);

        let full = CropRect {
            x: 0,
            y: 0,
            width: 4,
            height: 3,
        };
        let path = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &dir.join("f.tif"),
            &SaveOptions {
                crop: Some(full),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        assert_eq!((read_back(&path).width, read_back(&path).height), (4, 3));
    }

    #[test]
    fn a_crop_outside_the_image_is_an_error_and_writes_nothing() {
        let state = ExportState::default();
        state.set_stack(ramp());
        let path =
            std::env::temp_dir().join(format!("planets-cmd-crop3-{}.tif", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let outside = CropRect {
            x: 10,
            y: 10,
            width: 2,
            height: 2,
        };
        let err = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &path,
            &SaveOptions {
                crop: Some(outside),
                ..SaveOptions::default()
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("crop"), "{err}");
        assert!(!path.exists());
    }

    #[test]
    fn the_crop_is_cut_from_the_adjusted_image() {
        let state = ExportState::default();
        state.set_stack(ramp());
        let dir = std::env::temp_dir().join(format!("planets-cmd-crop4-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let brighter = Adjustments {
            brightness: 0.25,
            ..Adjustments::default()
        };
        let crop = CropRect {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        let path = save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &dir.join("b.tif"),
            &SaveOptions {
                adjustments: Some(&brighter),
                crop: Some(crop),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        let out = read_back(&path);
        assert_eq!((out.width, out.height), (2, 1));
        for (got, want) in out.data.iter().zip([0.25, 0.25 + 1.0 / 12.0]) {
            assert!((got - want).abs() < 1e-4, "{got} vs {want}");
        }
    }

    use planets_core::geometry::{Orientation, Resampling, Resize, ResizeTarget};

    fn turned(angle: f32) -> Geometry {
        Geometry {
            orientation: Orientation {
                angle,
                ..Orientation::default()
            },
            resize: None,
        }
    }

    fn save_with(name: &str, options: &SaveOptions) -> Result<String, SessionError> {
        let state = ExportState::default();
        state.set_stack(ramp());
        let dir = std::env::temp_dir().join(format!("planets-cmd-geo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let _ = std::fs::remove_file(&path);
        save(
            &state,
            ImageSource::Stack,
            ExportFormat::Tiff16,
            &path,
            options,
        )
    }

    #[test]
    fn a_turned_image_is_written_turned() {
        let geometry = turned(90.0);
        let path = save_with(
            "t.tif",
            &SaveOptions {
                geometry: Some(&geometry),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        let out = read_back(&path);
        // 4 x 3 turned clockwise is 3 x 4: 8 4 0 / 9 5 1 / 10 6 2 / 11 7 3 (twelfths).
        assert_eq!((out.width, out.height), (3, 4));
        for (got, want) in out
            .data
            .iter()
            .zip([8.0, 4.0, 0.0, 9.0, 5.0, 1.0, 10.0, 6.0, 2.0, 11.0, 7.0, 3.0])
        {
            assert!((got - want / 12.0).abs() < 1e-4, "{got} vs {}", want / 12.0);
        }
    }

    #[test]
    fn the_crop_is_set_on_the_turned_image() {
        let geometry = turned(90.0);
        let crop = CropRect {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        };
        let path = save_with(
            "tc.tif",
            &SaveOptions {
                crop: Some(crop),
                geometry: Some(&geometry),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        let out = read_back(&path);
        // The part 5 1 / 6 2 of the turned picture; cut from the unturned one the same rectangle would hold 5 6 / 9 10.
        assert_eq!((out.width, out.height), (2, 2));
        for (got, want) in out.data.iter().zip([5.0, 1.0, 6.0, 2.0]) {
            assert!((got - want / 12.0).abs() < 1e-4, "{got} vs {}", want / 12.0);
        }
    }

    #[test]
    fn a_crop_that_only_lies_inside_the_unturned_image_is_an_error() {
        let geometry = turned(90.0);
        let crop = CropRect {
            x: 3,
            y: 0,
            width: 1,
            height: 1,
        };
        let err = save_with(
            "tx.tif",
            &SaveOptions {
                crop: Some(crop),
                geometry: Some(&geometry),
                ..SaveOptions::default()
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("crop"), "{err}");
        let dir = std::env::temp_dir().join(format!("planets-cmd-geo-{}", std::process::id()));
        assert!(!dir.join("tx.tif").exists());
    }

    #[test]
    fn the_resize_comes_after_the_crop() {
        let geometry = Geometry {
            orientation: Orientation::default(),
            resize: Some(Resize {
                target: ResizeTarget::Percent(200.0),
                method: Resampling::Nearest,
            }),
        };
        let crop = CropRect {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
        };
        let path = save_with(
            "r.tif",
            &SaveOptions {
                crop: Some(crop),
                geometry: Some(&geometry),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        let out = read_back(&path);
        // 200 % of the 2 x 2 crop, not of the 4 x 3 image.
        assert_eq!((out.width, out.height), (4, 4));
        let want = [
            5.0, 5.0, 6.0, 6.0, 5.0, 5.0, 6.0, 6.0, 9.0, 9.0, 10.0, 10.0, 9.0, 9.0, 10.0, 10.0,
        ];
        for (got, want) in out.data.iter().zip(want) {
            assert!((got - want / 12.0).abs() < 1e-4, "{got} vs {}", want / 12.0);
        }
    }

    #[test]
    fn the_orientation_is_applied_to_the_adjusted_image() {
        let geometry = Geometry {
            orientation: Orientation {
                flip_horizontal: true,
                ..Orientation::default()
            },
            resize: None,
        };
        let brighter = Adjustments {
            brightness: 0.25,
            ..Adjustments::default()
        };
        let path = save_with(
            "ab.tif",
            &SaveOptions {
                adjustments: Some(&brighter),
                geometry: Some(&geometry),
                ..SaveOptions::default()
            },
        )
        .unwrap();
        let out = read_back(&path);
        // First row 0 1 2 3 twelfths, brightened by a quarter and mirrored.
        for (got, want) in out.data.iter().take(4).zip([3.0, 2.0, 1.0, 0.0]) {
            assert!((got - (0.25 + want / 12.0)).abs() < 1e-4, "{got}");
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
                &SaveOptions {
                    refine: Some(&refine),
                    ..SaveOptions::default()
                },
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
