use crate::{mix, Time, Track};
use crate::track::ClipInstance;

/// A Timeline is a composition of a fixed number of tracks. All of the tracks can be mixed down to
/// a single audio signal.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Timeline {
    pub tracks: Vec<Track>,
}

impl Timeline {
    pub fn new() -> Self {
        Timeline {
            tracks: vec![Track::new(); 4],
        }
    }

    pub fn len(&self) -> usize {
        self.tracks.iter()
            .map(|t| t.len())
            .max()
            .unwrap()
    }

    /// Returns an iterator over the clips on all tracks in order of start time.
    pub fn iter_clips(&self) -> impl Iterator<Item=(usize, &ClipInstance)> + '_ {
        self.tracks.iter()
            .enumerate()
            .flat_map(|(i, track)| {
                track.iter_clips().map(move |c| (i, c))
            })
    }

    pub fn render(&self, start_time: Time, buf: &mut [f32]) {
        self.render_exclude(start_time, buf, &[]);
    }

    pub fn render_exclude(&self, start_time: Time, buf: &mut [f32], exclude: &[usize]) {
        if start_time >= self.len() {
            buf.fill(0.0);
            return;
        }

        let rendered: Vec<Vec<f32>> = self.tracks.iter()
            .enumerate()
            .filter(|(i, _)| !exclude.contains(i))
            .map(|(_, t)| {
                let mut track_buf = vec![0.0f32; buf.len()];
                t.render(start_time, &mut track_buf);
                track_buf
            }).collect();

        let sources: Vec<&[f32]> = rendered.iter().map(|v| &v[..]).collect();
        mix(&sources, buf)
    }

    pub fn render_all(&self) -> Vec<f32> {
        if self.tracks.is_empty() {
            return Vec::new();
        }

        let mut buf = vec![0.0f32; self.len()];
        self.render(0, &mut buf);
        buf
    }
}
