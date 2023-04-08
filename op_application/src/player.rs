// TODO: Some part of this belongs in the library, which part exactly is TBD.

use std::sync::{Arc, Mutex};

use op_engine::Session;

pub struct Player {
    pub session: Arc<Mutex<Session>>,
    pub last_sample: usize,
}

impl Player {
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