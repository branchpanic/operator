use std::{fs, io};
use std::path::Path;

use crate::{Time, Timeline};

#[derive(thiserror::Error, Debug)]
pub enum ProjectError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("failed to deserialize project data: {message}")]
    LoadProjectError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("failed to serialize project data: {message}")]
    SaveProjectError {
        message: String,
    },
}

/// Owns persistent project data. This is what is saved, loaded, and exported by the user. Its main
/// component is a Timeline, but it also contains audio configuration.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub sample_rate: u32,
    pub timeline: Timeline,
}

const PROJECT_EXPORT_SPEC: hound::WavSpec = hound::WavSpec {
    channels: 1,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
};

const PROJECT_FILE_NAME: &str = "project.json";

impl Project {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            timeline: Timeline::new(),
        }
    }

    /// Loads a new Project. The path should be a directory containing a project file.
    pub fn load(path: &String) -> Result<Self, ProjectError> {
        let path = Path::new(path);
        let serialized_session = fs::read_to_string(path.join(PROJECT_FILE_NAME))?;
        let project: Project = serde_json::from_str(serialized_session.as_str())
            .map_err(|e| {
                ProjectError::LoadProjectError {
                    message: e.to_string(),
                    line: e.line(),
                    column: e.column(),
                }
            })?;

        Ok(project)
    }

    pub fn load_overwrite(&mut self, path: &String) -> Result<(), ProjectError> {
        let new = Self::load(path)?;
        *self = new;

        Ok(())
    }

    pub fn export_wav(&self, path: &String) -> Result<(), ProjectError> {
        let samples = self.timeline.render_all();
        let mut writer = hound::WavWriter::create(path, PROJECT_EXPORT_SPEC)
            .expect("predefined wav spec must be valid");

        for sample in samples {
            writer.write_sample((sample * i16::MAX as f32) as i16)
                .map_err(|e| {
                    match e {
                        hound::Error::IoError(io_error) => ProjectError::IoError(io_error),
                        _ => panic!("sample calculation was incorrect"),
                    }
                })?;
        }

        writer.finalize()
            .map_err(|e| {
                match e {
                    hound::Error::IoError(io_error) => ProjectError::IoError(io_error),
                    hound::Error::UnfinishedSample => panic!("unfinished sample at export"),
                    _ => panic!("unexpected error"),
                }
            })?;

        Ok(())
    }

    pub fn save(&self, path: &String) -> Result<(), ProjectError> {
        let path = Path::new(path);
        fs::create_dir_all(path)?;

        let serialized = serde_json::to_string(self)
            .map_err(|e| {
                ProjectError::SaveProjectError {
                    message: e.to_string()
                }
            })?;

        fs::write(path.join(PROJECT_FILE_NAME), serialized)?;
        Ok(())
    }

    pub fn sec_to_samples(&self, sec: f32) -> Time {
        (self.sample_rate as f32 * sec) as Time
    }

    pub fn samples_to_sec(&self, samples: Time) -> f32 {
        samples as f32 / self.sample_rate as f32
    }
}
