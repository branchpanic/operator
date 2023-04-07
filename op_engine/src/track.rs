use serde::{Deserialize, Serialize};

use crate::clip::Clip;
use crate::Time;

#[derive(Debug, Serialize, Deserialize)]
struct ClipInstance {
    time: Time,
    clip: Clip,
}

impl ClipInstance {
    pub fn new(time: Time, clip: Clip) -> ClipInstance {
        ClipInstance { time, clip }
    }

    /// Returns the first sample on the timeline that this clip is playing.
    pub fn start(&self) -> Time {
        self.time
    }

    /// Returns the first sample on the timeline after this clip ends.
    pub fn end(&self) -> Time {
        self.time + self.clip.data.len()
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Track {
    clips: Vec<ClipInstance>,
}

impl Track {
    pub fn new() -> Track {
        Track::default()
    }

    pub fn add_clip(&mut self, time: Time, clip: Clip) -> &Clip {
        self.clips.push(ClipInstance::new(time, clip));
        &self.clips.last().unwrap().clip
    }

    /// Returns the clip with the latest end sample.
    fn last_clip(&self) -> Option<&ClipInstance> {
        self.clips.iter()
            .max_by_key(|c| { c.end() })
    }

    /// Returns the first clip after t.
    fn next_clip(&self, t: Time) -> Option<&ClipInstance> {
        self.clips.iter()
            .filter(|c| c.start() > t)
            .min_by_key(|c| c.start())
    }

    /// Returns the first clip where clip start <= t < clip end.
    fn clip_at(&self, t: Time) -> Option<&ClipInstance> {
        self.clips.iter()
            .rfind(|c| { c.start() <= t && c.end() > t })
    }

    pub fn render_all(&self) -> Vec<f32> {
        let end = match self.last_clip() {
            None => return vec![0.0; 0],
            Some(last_clip) => last_clip.end(),
        };

        let mut buf = vec![0.0; end];
        self.render(0, buf.as_mut_slice());
        buf
    }

    pub fn render(&self, start: Time, into: &mut [f32]) {
        into.fill(0.0);
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_clip() {
        let mut track = Track::new();

        let clip = Clip::new(vec![1.0]);
        let result = track.add_clip(0, clip.clone());
        assert_eq!(clip.data, result.data);

        let clip = Clip::new(vec![2.0]);
        let result = track.add_clip(1, clip.clone());
        assert_eq!(clip.data, result.data);
    }

    #[test]
    fn test_last_clip() {
        let mut track = Track::new();

        assert!(track.last_clip().is_none(),
                "last clip must be None when track does not contain clips");

        // 1. only clip in the track should be the last clip
        // last
        // v
        // 1--------------
        let clip = Clip::new(vec![1.0]);
        track.add_clip(0, clip.clone());
        let last_clip = track.last_clip()
            .expect("last clip must be present when track contains clips");

        assert_eq!(clip.data, last_clip.clip.data);

        // 2. two disjoint clips, latter should be the last clip
        //   last
        //   v
        // 1-2------------
        let clip = Clip::new(vec![2.0]);
        track.add_clip(2, clip.clone());
        let last_clip = track.last_clip()
            .expect("last clip must be present when track contains clips");

        assert_eq!(clip.data, last_clip.clip.data);

        // 3. overlapping clips, clip with latest end should be the last clip
        //          last
        //          v
        // 1-2------------
        // 3333333333-----
        let clip = Clip::new(vec![3.0; 10]);
        track.add_clip(0, clip.clone());
        let last_clip = track.last_clip()
            .expect("last clip must be present when track contains clips");

        assert_eq!(clip.data, last_clip.clip.data);
    }

    #[test]
    fn test_clip_at() {
        let mut track = Track::new();

        assert!(track.clip_at(0).is_none());

        let clip = Clip::new(vec![1.0, 1.0]);
        track.add_clip(0, clip.clone());

        assert_eq!(clip.data, track.clip_at(0).unwrap().clip.data);
        assert_eq!(clip.data, track.clip_at(1).unwrap().clip.data);
        assert!(track.clip_at(2).is_none());

        let short_clip = Clip::new(vec![2.0]);
        track.add_clip(1, short_clip.clone());

        assert_eq!(clip.data, track.clip_at(0).unwrap().clip.data);
        assert_eq!(short_clip.data, track.clip_at(1).unwrap().clip.data,
            "latest added clip should take precedence when overlapping");
        assert!(track.clip_at(2).is_none());
    }

    #[test]
    fn test_next_clip() {
        let mut track = Track::new();

        assert!(track.next_clip(0).is_none());

        // 1. query during clip should not return clip
        // t
        // v        next=None
        // 11-----------
        let clip = Clip::new(vec![1.0, 1.0]);
        track.add_clip(0, clip.clone());
        assert!(track.next_clip(0).is_none(),
            "next_clip should not return clips where start <= t < end");

        // 2. two disjoint clips, query during first should return second
        // t  next
        // v  v
        // 11-2---------
        let clip = Clip::new(vec![2.0]);
        track.add_clip(3, clip.clone());
        let result = track.next_clip(0).unwrap();
        assert_eq!(3, result.time);
        assert_eq!(clip.data, result.clip.data);
        assert!(track.next_clip(3).is_none());

        // 3. clips overlapping at t, query before overlap should return overlapping clip
        // t
        // v
        // 11-2---------
        // -3-----------
        //  ^
        //  next
        let clip = Clip::new(vec![3.0]);
        track.add_clip(1, clip.clone());
        let result = track.next_clip(0).unwrap();
        assert_eq!(1, result.time);
        assert_eq!(clip.data, result.clip.data);

        // 4. t is past end of all clips, query should return none
        assert!(track.next_clip(1234).is_none());
    }
}
