use std::{fs, io};
use std::path::Path;

use crate::{Clip, Time, Track};
use crate::session::SessionError::{LoadSessionError, SaveSessionError};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Session {
    sample_rate: i32,
    track: Track,
}

#[derive(thiserror::Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("failed to deserialize session data: {message}")]
    LoadSessionError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("failed to serialize session data")]
    SaveSessionError {
        message: String,
    },
}

impl Session {
    pub fn new() -> Self {
        let mut default = Session::default();
        default.sample_rate = 44100;
        default
    }

    pub fn load(path: &String) -> Result<Self, SessionError> {
        let path = Path::new(path);
        let serialized_session = fs::read_to_string(path.join("session.json"))?;
        let session: Session = serde_json::from_str(serialized_session.as_str())
            .map_err(|e| {
                LoadSessionError {
                    message: e.to_string(),
                    line: e.line(),
                    column: e.column(),
                }
            })?;

        Ok(session)
    }

    pub fn load_overwrite(&mut self, path: &String) -> Result<(), SessionError> {
        let new = Self::load(path)?;
        *self = new;

        Ok(())
    }

    pub fn sec_to_samples(&self, sec: f32) -> Time {
        (self.sample_rate as f32 * sec) as Time
    }

    pub fn add_clip(&mut self, track: usize, time: usize, clip: Clip) -> &Clip {
        if track != 0 {
            todo!("Multiple track support");
        }

        self.track.add_clip(time, clip)
    }

    fn render_all(&self) -> Vec<f32> {
        self.track.render_all()
    }

    pub fn len(&self) -> usize { self.track.len() }

    pub fn render(&self, start_time: Time, buf: &mut [f32]) {
        self.track.render(start_time, buf);
    }

    pub fn export(&self, path: &String) -> Result<(), SessionError> {
        const SPEC: hound::WavSpec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let samples = self.render_all();
        let mut writer = hound::WavWriter::create(path, SPEC)
            .expect("predefined wav spec must be valid");

        for sample in samples {
            writer.write_sample((sample * i16::MAX as f32) as i16)
                .map_err(|e| {
                    match e {
                        hound::Error::IoError(io_error) => SessionError::IoError(io_error),
                        _ => panic!("sample calculation was incorrect"),
                    }
                })?;
        }

        writer.finalize()
            .map_err(|e| {
                match e {
                    hound::Error::IoError(io_error) => SessionError::IoError(io_error),
                    hound::Error::UnfinishedSample => panic!("unfinished sample at export"),
                    _ => panic!("unexpected error"),
                }
            })?;

        Ok(())
    }

    pub fn save(&self, path: &String) -> Result<(), SessionError> {
        let path = Path::new(path);
        fs::create_dir_all(path)?;

        let serialized = serde_json::to_string(self)
            .map_err(|e| {
                SaveSessionError {
                    message: e.to_string()
                }
            })?;

        fs::write(path.join("session.json"), serialized)?;
        Ok(())
    }
}
