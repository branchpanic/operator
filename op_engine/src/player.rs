use std::f32::consts::{PI, TAU};
use std::sync::{Arc, Mutex};

use crate::{Project, Time};

struct SimpleOsc {
    phase: f32,
}

impl SimpleOsc {
    fn new() -> Self { SimpleOsc { phase: 0.0 } }

    fn sample(&mut self, sample_rate: u32, freq: f32) -> f32 {
        self.phase = self.phase + 2.0 * PI * (freq / sample_rate as f32);

        while self.phase > 2.0 * PI {
            self.phase = self.phase - 2.0 * PI;
        }

        while self.phase < 0.0 {
            self.phase = self.phase + 2.0 * PI;
        }

        self.phase.sin()
    }
}

pub struct Player {
    project: Arc<Mutex<Project>>,
    osc: SimpleOsc,
    time: Time,
}

impl Player {
    pub fn new(project: Arc<Mutex<Project>>) -> Self {
        Player {
            project,
            osc: SimpleOsc::new(),
            time: 0,
        }
    }

    pub fn seek(&mut self, time: Time) {
        self.time = time;
    }

    pub fn write_next_block<T>(&mut self, output: &mut [T], channels: usize)
        where
            T: cpal::Sample + cpal::FromSample<f32>,
    {
        let buf_size = output.len() / channels;

        // TODO: Probably don't want to allocate here
        let mut next_block = vec![0f32; buf_size];

        {
            let project = self.project.lock().unwrap();
            project.render(self.time, &mut next_block);
            self.time += buf_size;
            if self.time > project.len() {
                self.time = 0
            }

            for i in 0..buf_size {
                next_block[i] = (next_block[i] + 0.1 * self.osc.sample(48000, 440.0)) / 2.0;
                debug_assert!(-1.0 <= next_block[i] && next_block[i] <= 1.0);
            }
        }

        let mut i = 0;
        for frame in output.chunks_mut(channels) {
            let value: T = T::from_sample(next_block[i]);

            for sample in frame.iter_mut() {
                *sample = value;
            }

            i += 1;
        }
    }
}