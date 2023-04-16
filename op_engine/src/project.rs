use std::{fs, io};
use std::path::Path;

use crate::{Clip, mix, Time, Track};
use crate::project::ProjectError::{LoadProjectError, SaveProjectError};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub sample_rate: u32,
    tracks: Vec<Track>,
}

pub struct ClipInfo {
    pub track: usize,
    pub start: Time,
}

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

impl Project {
    pub fn new() -> Self {
        Project {
            sample_rate: 44100,
            tracks: vec![Track::new(); 4],
        }
    }

    pub fn load(path: &String) -> Result<Self, ProjectError> {
        let path = Path::new(path);
        let serialized_session = fs::read_to_string(path.join("project.json"))?;
        let project: Project = serde_json::from_str(serialized_session.as_str())
            .map_err(|e| {
                LoadProjectError {
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

    pub fn sec_to_samples(&self, sec: f32) -> Time {
        (self.sample_rate as f32 * sec) as Time
    }

    pub fn add_clip(&mut self, track: usize, time: usize, clip: Clip) -> &Clip {
        debug_assert!(track < self.tracks.len());
        self.tracks[track].add_clip(time, clip)
    }

    pub fn iter_clips(&self) -> impl Iterator<Item = ClipInfo> + '_ {
        self.tracks.iter()
            .enumerate()
            .flat_map(|(i, track)| {
                track.iter_clips().map(move |c| ClipInfo {
                    track: i,
                    start: c.start(),
                })
            })
    }

    fn render_all(&self) -> Vec<f32> {
        if self.tracks.is_empty() {
            return Vec::new()
        }

        let mut buf = vec![0.0f32; self.len()];
        self.render(0, &mut buf);
        buf
    }

    pub fn len(&self) -> usize {
        self.tracks.iter()
            .map(|t| t.len())
            .max()
            .unwrap()
    }

    pub fn render(&self, start_time: Time, buf: &mut [f32]) {
        if start_time >= self.len() {
            buf.fill(0.0);
            return;
        }

        let rendered: Vec<Vec<f32>> = self.tracks.iter()
            .map(|t| {
                let mut track_buf = vec![0.0f32; buf.len()];
                t.render(start_time, &mut track_buf);
                track_buf
            }).collect();

        let sources: Vec<&[f32]> = rendered.iter().map(|v| &v[..]).collect();
        mix(&sources, buf)
    }

    pub fn export(&self, path: &String) -> Result<(), ProjectError> {
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
                SaveProjectError {
                    message: e.to_string()
                }
            })?;

        fs::write(path.join("project.json"), serialized)?;
        Ok(())
    }
}
