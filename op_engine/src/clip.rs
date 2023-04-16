use hound::SampleFormat;

use crate::clip::ClipError::ClipReadError;

/// Represents a chunk of single channel, f32 audio.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Clip {
    pub data: Vec<f32>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClipError {
    #[error("failed to read audio file: {source}")]
    ClipReadError {
        source: hound::Error,
    },

    #[error("unsupported sample format: {bits_per_sample}")]
    UnsupportedSampleFormat {
        bits_per_sample: u16
    },
}

impl Clip {
    pub fn new(data: Vec<f32>) -> Self {
        Clip { data }
    }

    // TODO: Session should probably own sample data, since this will load from disk every time.
    // (However, this isn't actually on the critical path. Not many samples will be loaded directly
    // onto the timeline. Instead, they'll be played through instruments.)
    pub fn from_file(path: &String) -> Result<Self, ClipError> {
        let mut reader = hound::WavReader::open(path).map_err(|e| ClipReadError { source: e })?;
        let spec = reader.spec();

        // TODO: Figure out where sample rate info lives (probably Session)
        debug_assert_eq!(spec.sample_rate, 44100);
        debug_assert!(spec.sample_format == SampleFormat::Int, "TODO: float support");

        let samples = reader.samples::<i32>()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ClipReadError { source: e })?
            .into_iter()
            .map(|s| { s as f32 / ((1 << spec.bits_per_sample) / 2 - 1) as f32 })
            .step_by(spec.channels as usize)
            .collect::<Vec<_>>();

        Ok(Clip::new(samples))
    }

    pub fn len(&self) -> usize { self.data.len() }
}
