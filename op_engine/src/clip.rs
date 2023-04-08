use crate::Time;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Clip {
    pub data: Vec<f32>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClipError {
    #[error("failed to read audio file")]
    ClipReadError(#[from] hound::Error),
}

impl Clip {
    pub fn new(data: Vec<f32>) -> Self {
        Clip { data }
    }

    // TODO: Session should probably own sample data, since this will load from disk every time.
    // (However, this isn't actually on the critical path. Not many samples will be loaded directly
    // onto the timeline. Instead, they'll be played through instruments.)
    pub fn from_file(path: &String) -> Result<Self, ClipError> {
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

        Ok(Clip::new(samples))
    }

    pub fn len(&self) -> usize { self.data.len() }
}
