use std::sync::{Arc, Mutex};

use cpal::{BufferSize, StreamConfig};
use dasp::{signal, Signal};
use dasp::interpolate::linear::Linear;
use dasp::signal::ConstHz;

use crate::{Project, Time};
use crate::player::PlayerError::InvalidBufferSize;

pub struct Player {
    project: Arc<Mutex<Project>>,
    osc: signal::Sine<ConstHz>,
    time: Time,
    config: StreamConfig,

    render_buf: Vec<f32>,  // FIXME: Currently assuming mono
}

#[derive(thiserror::Error, Debug)]
pub enum PlayerError {
    #[error("invalid buffer size (expected BufferSize::Fixed): {0:?}")]
    InvalidBufferSize(BufferSize),
}

impl Player {
    pub fn new(project: Arc<Mutex<Project>>, config: StreamConfig) -> Result<Self, PlayerError> {
        let buffer = match config.buffer_size {
            BufferSize::Fixed(frame_count) => vec![0.0; frame_count as usize],
            _ => return Err(InvalidBufferSize(config.buffer_size)),
        };

        Ok(Player {
            project,
            // TODO: Record generators at playback rate or project rate?
            //   Probably playback, then downsample when recorded.
            osc: signal::rate(44100.0).const_hz(440.0).sine(),
            time: 0,
            config,
            render_buf: buffer,
        })
    }

    pub fn seek(&mut self, time: Time) {
        self.time = time;
    }

    pub fn write_next_block<T>(&mut self, output: &mut [T], channels: usize)
        where
            T: cpal::Sample + cpal::FromSample<f32>,
    {
        let project = self.project.lock().unwrap();

        let dst_samples = output.len() / channels;
        let src_sample_rate = project.sample_rate as f64;
        let dst_sample_rate = self.config.sample_rate.0 as f64;
        let src_samples_per_dst = src_sample_rate / dst_sample_rate;
        let src_samples = (dst_samples as f64 * src_samples_per_dst) as usize;

        if self.render_buf.len() < src_samples {
            eprintln!("increasing render buffer size from {} to {}", self.render_buf.len(), src_samples);
            self.render_buf.resize(src_samples, 0.0);
        }

        project.render(self.time, &mut self.render_buf[..src_samples]);
        self.time += src_samples;
        if self.time > project.len() {
            self.time = 0
        }

        for i in 0..src_samples {
            self.render_buf[i] = (self.render_buf[i] + 0.1 * self.osc.next() as f32) / 2.0;
            debug_assert!(-1.0 <= self.render_buf[i] && self.render_buf[i] <= 1.0);
        }

        if src_sample_rate != dst_sample_rate {
            let interpolator = Linear::new(self.render_buf[0], self.render_buf[1]);
            let src_signal = signal::from_iter(self.render_buf[..src_samples].iter().cloned());

            // TODO: Deduplicate this block -- surely both dst_signals have some trait in common?
            let mut dst_signal = src_signal.scale_hz(interpolator, src_samples_per_dst);

            for frame in output.chunks_mut(channels) {
                let value: T = T::from_sample(dst_signal.next());

                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        } else {
            let mut dst_signal = signal::from_iter(self.render_buf[..src_samples].iter().cloned());

            for frame in output.chunks_mut(channels) {
                let value: T = T::from_sample(dst_signal.next());

                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        }
    }
}