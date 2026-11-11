//! Project file: everything needed to pick up a session again (`docs/formats.md`, section
//! "Projektdatei").
//!
//! The file is JSON. Besides the recordings it holds named **sections**, one per processing
//! step (`align`, `stack`, `wavelet`, `colour`). Each step owns the layout of its section and
//! stores it with [`Project::set_section`]; this module only keeps the raw values. A section that
//! a newer or older program does not know is kept as it is when the file is saved again.

use crate::export::write_atomic;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

/// Extension of project files, without the dot.
pub const PROJECT_EXTENSION: &str = "rkproj";

/// Layout version written to new files. Files with a higher number are refused.
pub const FORMAT_VERSION: u32 = 1;

/// Section names used by the processing steps.
pub const SECTION_ALIGN: &str = "align";
pub const SECTION_STACK: &str = "stack";
pub const SECTION_WAVELET: &str = "wavelet";
pub const SECTION_COLOUR: &str = "colour";

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a valid project file: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "the project was saved by a newer version (format {found}, this version reads up to {supported})"
    )]
    UnsupportedVersion { found: u32, supported: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub format_version: u32,
    /// The files passed to *Select*: one video, or the still frames of a sequence in order.
    pub recording: Vec<String>,
    /// Settings and results of the processing steps by name.
    #[serde(default)]
    pub sections: BTreeMap<String, Value>,
}

impl Project {
    /// A project for the given recording files, without any step settings yet.
    pub fn new(recording: Vec<String>) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            recording,
            sections: BTreeMap::new(),
        }
    }

    /// Stores the settings of one step, replacing an earlier value.
    pub fn set_section<T: Serialize>(&mut self, name: &str, value: &T) -> Result<(), ProjectError> {
        self.sections
            .insert(name.to_owned(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Reads the settings of one step; `None` if the project has no such section.
    pub fn section<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, ProjectError> {
        self.sections
            .get(name)
            .map(|v| serde_json::from_value(v.clone()).map_err(ProjectError::from))
            .transpose()
    }

    /// Recording files that do not exist (any more), for a message when reopening.
    pub fn missing_files(&self) -> Vec<String> {
        self.recording
            .iter()
            .filter(|p| !Path::new(p).exists())
            .cloned()
            .collect()
    }

    /// Writes the project. A path without the project extension gets it; the returned path is the
    /// file that was written.
    pub fn save(&self, path: &Path) -> Result<PathBuf, ProjectError> {
        let path = with_project_extension(path);
        write_atomic(&path, |out| {
            serde_json::to_writer_pretty(out, self)?;
            Ok::<(), ProjectError>(())
        })?;
        Ok(path)
    }

    /// Reads a project written by [`Project::save`].
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        let project: Project = serde_json::from_reader(BufReader::new(File::open(path)?))?;
        if project.format_version > FORMAT_VERSION {
            return Err(ProjectError::UnsupportedVersion {
                found: project.format_version,
                supported: FORMAT_VERSION,
            });
        }
        Ok(project)
    }
}

/// Keeps a `.rkproj` extension (any case) and otherwise sets it.
pub fn with_project_extension(path: &Path) -> PathBuf {
    let has = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(PROJECT_EXTENSION));
    if has {
        path.to_path_buf()
    } else {
        path.with_extension(PROJECT_EXTENSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Sample {
        min_distance: u32,
        levels: Vec<f32>,
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("planets-project-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn project_roundtrip_keeps_recording_and_sections() {
        let dir = temp_dir("roundtrip");
        let mut project = Project::new(vec!["C:/data/jupiter.avi".to_owned()]);
        let sample = Sample {
            min_distance: 30,
            levels: vec![1.0, 2.5, -0.5],
        };
        project.set_section(SECTION_WAVELET, &sample).unwrap();

        let path = project.save(&dir.join("session.rkproj")).unwrap();
        let loaded = Project::load(&path).unwrap();
        assert_eq!(loaded, project);
        assert_eq!(
            loaded.section::<Sample>(SECTION_WAVELET).unwrap(),
            Some(sample)
        );
        assert_eq!(loaded.section::<Sample>(SECTION_ALIGN).unwrap(), None);
    }

    #[test]
    fn unknown_sections_survive_a_second_save() {
        let dir = temp_dir("unknown");
        let path = dir.join("u.rkproj");
        fs::write(
            &path,
            r#"{"formatVersion":1,"recording":[],"sections":{"future":{"a":[1,2,3]}}}"#,
        )
        .unwrap();
        let loaded = Project::load(&path).unwrap();
        let again = loaded.save(&dir.join("u2.rkproj")).unwrap();
        let text = fs::read_to_string(again).unwrap();
        assert!(text.contains("\"future\""));
        assert!(text.contains("\"formatVersion\""));
    }

    #[test]
    fn missing_sections_key_means_no_sections() {
        let dir = temp_dir("nosections");
        let path = dir.join("n.rkproj");
        fs::write(&path, r#"{"formatVersion":1,"recording":["a.ser"]}"#).unwrap();
        let loaded = Project::load(&path).unwrap();
        assert!(loaded.sections.is_empty());
        assert_eq!(loaded.recording, vec!["a.ser"]);
    }

    #[test]
    fn newer_format_is_refused() {
        let dir = temp_dir("newer");
        let path = dir.join("v.rkproj");
        fs::write(&path, r#"{"formatVersion":99,"recording":[]}"#).unwrap();
        assert!(matches!(
            Project::load(&path),
            Err(ProjectError::UnsupportedVersion { found: 99, .. })
        ));
    }

    #[test]
    fn foreign_files_are_rejected() {
        let dir = temp_dir("foreign");
        let path = dir.join("f.rkproj");
        fs::write(&path, "not json").unwrap();
        assert!(matches!(Project::load(&path), Err(ProjectError::Json(_))));
        assert!(matches!(
            Project::load(&dir.join("absent.rkproj")),
            Err(ProjectError::Io(_))
        ));
    }

    #[test]
    fn extension_is_added_but_not_doubled() {
        assert_eq!(
            with_project_extension(Path::new("a/b")),
            Path::new("a/b.rkproj")
        );
        assert_eq!(
            with_project_extension(Path::new("a/b.RKPROJ")),
            Path::new("a/b.RKPROJ")
        );
        assert_eq!(
            with_project_extension(Path::new("a/b.txt")),
            Path::new("a/b.rkproj")
        );
    }

    #[test]
    fn missing_recording_files_are_reported() {
        let dir = temp_dir("missing");
        let present = dir.join("present.ser");
        fs::write(&present, b"x").unwrap();
        let gone = dir.join("gone.ser");
        let project = Project::new(vec![
            present.to_string_lossy().into_owned(),
            gone.to_string_lossy().into_owned(),
        ]);
        assert_eq!(
            project.missing_files(),
            vec![gone.to_string_lossy().into_owned()]
        );
    }
}
