use std::mem::{take};
use crate::{Clip, Time, Track};

#[derive(Default)]
pub struct Session {
    recording_at: Time,
    record_data: Vec<f32>,

    track: Track,
}

impl Session {
    pub fn new() -> Session {
        Session::default()
    }

    pub fn record_start(&mut self, at: Time) {
        self.recording_at = at;
        self.record_data.clear();
    }

    pub fn record_append(&mut self, data: &[f32]) {
        self.record_data.extend_from_slice(data);
    }

    pub fn record_end(&mut self) -> Option<&Clip> {
        if self.record_data.is_empty() {
            return None;
        }

        let clip = Clip::new(self.recording_at, take(&mut self.record_data));
        Some(self.track.add_clip(clip))
    }

    pub fn render(&self, into: &mut [f32]) {
        self.track.render(into);
    }

    pub fn render_all(&self) -> Vec<f32> {
        self.track.render_all()
    }
}
