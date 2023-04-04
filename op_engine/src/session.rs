use std::error::Error;
use std::fs;
use std::fs::{create_dir_all, write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::{Clip, Time, Track};

#[derive(Default, Serialize, Deserialize)]
pub struct Session {
    sample_rate: i32,
    track: Track,
}

impl Session {
    pub fn new() -> Self {
        let mut default = Session::default();
        default.sample_rate = 44100;
        default
    }

    pub fn load(path: &String) -> Result<Self, Box<dyn Error>> {
        let path = Path::new(path);
        let serialized_session = fs::read_to_string(path.join("session.json"))?;
        let session: Session = serde_json::from_str(serialized_session.as_str())?;
        Ok(session)
    }

    pub fn sec_to_samples(&self, sec: f32) -> Time {
        (self.sample_rate as f32 * sec) as Time
    }

    pub fn add_clip(&mut self, track: usize, clip: Clip) -> &Clip {
        if track != 0 {
            todo!("Multiple track support");
        }

        self.track.add_clip(clip)
    }

    fn render_all(&self) -> Vec<f32> {
        self.track.render_all()
    }

    pub fn export(&self, path: &String) -> Result<(), Box<dyn Error>> {
        let samples = self.render_all();
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;

        for sample in samples {
            writer.write_sample((sample * i16::MAX as f32) as i16)?;
        }

        Ok(())
    }

    pub fn save(&self, path: &String) -> Result<(), Box<dyn Error>> {
        let path = Path::new(path);
        create_dir_all(path)?;
        let serialized = serde_json::to_string(self)?;
        write(path.join("session.json"), serialized)?;
        Ok(())
    }
}
