use std::mem::take;
use std::sync::{Arc, Mutex};

use cpal::{BufferSize, StreamConfig};
use dasp::{signal, Signal};
use dasp::interpolate::linear::Linear;

use crate::{Clip, Project, Time};

pub struct Player {
    config: StreamConfig,
    output_buf: Vec<f32>,  // FIXME: Currently assuming mono

    project: Arc<Mutex<Project>>,
    pub playing_project: bool,
    time: Time,

    recording: bool,
    record_track: usize,
    record_start: Time,
    record_buf: Vec<f32>,
}

#[derive(thiserror::Error, Debug)]
pub enum PlayerError {
    #[error("invalid buffer size (expected BufferSize::Fixed): {0:?}")]
    InvalidBufferSize(BufferSize),
}

impl Player {
    pub fn new(project: Arc<Mutex<Project>>, config: StreamConfig) -> Result<Self, PlayerError> {
        let buf_size = match config.buffer_size {
            BufferSize::Fixed(frame_count) => frame_count as usize,
            _ => return Err(PlayerError::InvalidBufferSize(config.buffer_size)),
        };

        Ok(Player {
            project,
            time: 0,
            config,
            output_buf: vec![0.0; buf_size],

            playing_project: false,

            recording: false,
            record_track: 0,
            record_start: 0,
            record_buf: vec![],
        })
    }

    pub fn set_recording(&mut self, recording: bool, record_track: usize) {
        if self.recording == recording {
            return;
        }

        self.recording = recording;

        if !recording && !self.record_buf.is_empty() {
            self.write_recorded_clip();
        } else if recording {
            self.record_start = self.time;
            self.record_track = record_track;
            self.record_buf.clear();
        }
    }

    fn write_recorded_clip(&mut self) {
        let mut project = self.project.lock().unwrap();
        let clip = Clip::new(take(&mut self.record_buf));
        project.timeline.tracks[self.record_track].add_clip(self.record_start, clip);
    }

    pub fn seek(&mut self, time: Time) {
        self.time = time;
    }

    pub fn time(&self) -> Time {
        self.time
    }

    fn write_signal<T, U>(signal: &mut impl Signal<Frame=T>, output: &mut [U], channels: usize)
        where
            U: cpal::Sample + cpal::FromSample<T>
    {
        for frame in output.chunks_mut(channels) {
            let value = U::from_sample(signal.next());
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }

    pub fn write_next_block<T>(&mut self, output: &mut [T], channels: usize)
        where
            T: cpal::Sample + cpal::FromSample<f32>,
    {
        let mut project = self.project.lock().unwrap();

        let dst_samples = output.len() / channels;
        let src_sample_rate = project.sample_rate as f64;
        let dst_sample_rate = self.config.sample_rate.0 as f64;
        let src_samples_per_dst = src_sample_rate / dst_sample_rate;
        let src_samples = (dst_samples as f64 * src_samples_per_dst) as usize;

        if self.output_buf.len() < src_samples {
            eprintln!("increasing render buffer size from {} to {}", self.output_buf.len(), src_samples);
            self.output_buf.resize(src_samples, 0.0);
        }

        if self.playing_project {
            project.timeline.render(self.time, &mut self.output_buf[..src_samples]);
            self.time += src_samples;
            if self.time > project.timeline.len() && !self.recording {
                self.time = 0;
            }
        } else {
            self.output_buf.fill(0.0);
        }

        for i in 0..src_samples {
            let sample = project.generator.next();
            self.output_buf[i] += sample;
            self.output_buf[i] = self.output_buf[i].clamp(-1.0, 1.0);

            if self.playing_project && self.recording {
                self.record_buf.push(sample);
            }
        }

        let mut src_signal = signal::from_iter(self.output_buf[..src_samples].iter().cloned());

        if src_sample_rate == dst_sample_rate {
            Self::write_signal(&mut src_signal, output, channels);
            return;
        }

        let interpolator = Linear::new(self.output_buf[0], self.output_buf[1]);
        let mut resampled = src_signal.scale_hz(interpolator, src_samples_per_dst);
        Self::write_signal(&mut resampled, output, channels);
    }
}