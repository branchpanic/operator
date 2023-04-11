use std::sync::{Arc, Mutex};
use cpal::{BufferSize, StreamConfig};
use dasp::{signal, Signal};
use dasp::signal::ConstHz;

use crate::{Project, Time};
use crate::player::PlayerError::InvalidBufferSize;

pub struct Player {
    project: Arc<Mutex<Project>>,
    osc: signal::Sine<ConstHz>,
    time: Time,
    config: StreamConfig,
    buffer: Vec<f32>,  // FIXME: Currently assuming mono
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
            osc: signal::rate(config.sample_rate.0.into()).const_hz(440.0).sine(),
            time: 0,
            config,
            buffer,
        })
    }

    pub fn seek(&mut self, time: Time) {
        self.time = time;
    }

    pub fn write_next_block<T>(&mut self, output: &mut [T], channels: usize)
        where
            T: cpal::Sample + cpal::FromSample<f32>,
    {
        debug_assert_eq!(self.buffer.len(), output.len() / self.config.channels as usize);

        {
            let project = self.project.lock().unwrap();
            project.render(self.time, &mut self.buffer);
            self.time += self.buffer.len();
            if self.time > project.len() {
                self.time = 0
            }

            for i in 0..self.buffer.len() {
                self.buffer[i] = (self.buffer[i] + 0.1 * self.osc.next() as f32) / 2.0;
                debug_assert!(-1.0 <= self.buffer[i] && self.buffer[i] <= 1.0);
            }

            if self.config.sample_rate.0 != project.sample_rate {
                todo!("resample to output sample rate");
            }
        }

        let mut i = 0;
        for frame in output.chunks_mut(channels) {
            let value: T = T::from_sample(self.buffer[i]);

            for sample in frame.iter_mut() {
                *sample = value;
            }

            i += 1;
        }
    }
}