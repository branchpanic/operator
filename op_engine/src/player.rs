// TODO: Some part of this belongs in the library, which part exactly is TBD.

use std::sync::{Arc, Mutex};

use crate::{Session, Time};

pub struct Player {
    session: Arc<Mutex<Session>>,
    last_sample: Time,
}

impl Player {
    pub fn new(session: Arc<Mutex<Session>>) -> Self {
        Player {
            session,
            last_sample: 0
        }
    }

    pub fn seek(&mut self, sample: Time) {
        self.last_sample = sample;
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

            s.render(self.last_sample, &mut next_block);
            self.last_sample += buf_size;
            if self.last_sample > s.len() {
                self.last_sample = 0
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