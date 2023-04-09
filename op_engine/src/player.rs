use std::sync::{Arc, Mutex};

use crate::{Project, Time};

pub struct Player {
    session: Arc<Mutex<Project>>,
    time: Time,
}

impl Player {
    pub fn new(project: Arc<Mutex<Project>>) -> Self {
        Player {
            session: project,
            time: 0
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
            let s = self.session.lock().unwrap();

            s.render(self.time, &mut next_block);
            self.time += buf_size;
            if self.time > s.len() {
                self.time = 0
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