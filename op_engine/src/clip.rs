use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::Time;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub start: Time,
    pub data: Vec<f32>,
}

impl Clip {
    pub fn new(start: Time, data: Vec<f32>) -> Self {
        Clip { start, data }
    }

    // TODO: Session should probably own sample data, since this will load from disk every time.
    // (However, this isn't actually on the critical path. Not many samples will be loaded directly
    // onto the timeline. Instead, they'll be played through instruments.)
    pub fn from_file(start: Time, path: &String) -> Result<Self, Box<dyn Error>> {
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();

        // TODO: Figure out where sample rate info lives (probably Session)
        assert_eq!(spec.sample_rate, 44100);

        let samples = reader.samples::<i16>()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|s| { s as f32 / i16::MAX as f32 })
            .step_by(spec.channels as usize)
            .collect::<Vec<_>>();

        Ok(Clip::new(start, samples))
    }

    pub fn end(&self) -> Time {
        self.start + self.data.len() as Time
    }
}
