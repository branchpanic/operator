use std::cmp::min;
use std::slice::Iter;

use serde::{Deserialize, Serialize};

use crate::clip::Clip;
use crate::clip_database::{ClipDatabase, ClipId};
use crate::Time;

/// A ClipInstance is a clip with a defined starting time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipInstance {
    pub time: Time,
    pub clip_id: ClipId,
}

impl ClipInstance {
    pub fn new(time: Time, clip_id: ClipId) -> ClipInstance {
        ClipInstance { time, clip_id }
    }

    /// Returns the first sample on the timeline that this clip is playing.
    pub fn start(&self) -> Time {
        self.time
    }

    /// Returns the first sample on the timeline after this clip ends.
    pub fn end(&self, database: &ClipDatabase) -> Option<Time> {
        self.len(database).map(|len| self.time + len)
    }

    pub fn len(&self, database: &ClipDatabase) -> Option<Time> {
        database.get(self.clip_id).map(|clip| clip.data.len())
    }
}

/// A Track is a sequence of clip instances. Clips may overlap, but only one clip is ever played
/// at a time on a single track.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    clips: Vec<ClipInstance>,
}

/// Copy up to `max_copy` samples from `clip` starting at `clip_start` to `buf` starting at
/// `buf_start`. Fewer than `max_copy` samples will be copied when:
///     - There is not enough space in the buffer
///     - The clip is not long enough
fn copy_clip_data(clip: &Clip,
                  buf: &mut [f32],
                  clip_start: usize,
                  buf_start: usize,
                  max_copy: usize,
) -> usize {
    debug_assert!(clip_start < clip.data.len());
    debug_assert!(buf_start <= buf.len());

    let buf_space = buf.len() - buf_start;
    let clip_space = clip.data.len() - clip_start;
    let actual_copy = min(max_copy, min(buf_space, clip_space));

    if actual_copy == 0 {
        return 0;
    }

    buf[buf_start..buf_start + actual_copy]
        .copy_from_slice(&clip.data[clip_start..clip_start + actual_copy]);

    actual_copy
}

impl Track {
    pub fn new() -> Track {
        Track::default()
    }

    pub fn instantiate_clip(&mut self, clip_id: ClipId, time: Time) -> &ClipInstance {
        self.clips.push(ClipInstance::new(time, clip_id));
        self.clips.last().unwrap()
    }

    /// Returns the clip with the latest end sample.
    fn last_clip(&self, database: &ClipDatabase) -> Option<&ClipInstance> {
        self.clips.iter()
            .max_by_key(|c| { c.end(database) })
    }

    /// Returns the first clip after t.
    fn next_clip(&self, t: Time) -> Option<&ClipInstance> {
        self.clips.iter()
            .filter(|c| c.start() > t)
            .min_by_key(|c| c.start())
    }

    /// Returns the first clip where clip start <= t < clip end.
    fn clip_at(&self, database: &ClipDatabase, t: Time) -> Option<&ClipInstance> {
        self.clips.iter()
            .rfind(|c| { c.start() <= t && c.end(database) > Some(t) })
    }

    pub fn render(&self, database: &ClipDatabase, start_time: Time, buf: &mut [f32]) {
        buf.fill(0.0);

        if start_time >= self.len(database) {
            return;
        }

        let mut time = start_time;
        let end_time = time + buf.len();

        // If there is a clip ongoing at the start, copy it partially
        if let Some(clip_instance) = self.clip_at(database, time) {
            if let Some(clip) = database.get(clip_instance.clip_id) {
                copy_clip_data(
                    &clip,
                    buf,
                    time - clip_instance.start(),
                    0,
                    clip.len(),
                );
            }
        }

        // Copy clips until end
        while let Some(clip_instance) = self.next_clip(time) {
            let clip = match database.get(clip_instance.clip_id) {
                Some(clip) => clip,
                None => break
            };

            time = clip_instance.start();

            if time > end_time {
                break;
            }

            copy_clip_data(
                &clip,
                buf,
                0,
                time - start_time,
                clip.len(),
            );
        }
    }

    pub fn len(&self, database: &ClipDatabase) -> usize {
        self.last_clip(database)
            .and_then(|c| c.end(database))
            .unwrap_or(0)
    }

    pub fn render_all(&self, database: &ClipDatabase) -> Vec<f32> {
        let end = match self.last_clip(database).and_then(|c| c.end(database)) {
            None => return vec![0.0; 0],
            Some(end) => end,
        };

        let mut buf = vec![0.0; end];
        self.render(database, 0, buf.as_mut_slice());
        buf
    }

    pub fn iter_clips(&self) -> Iter<'_, ClipInstance> {
        self.clips.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_clip() {
        let mut track = Track::new();
        let mut db = ClipDatabase::new();

        let clip_1 = db.add(Clip::new(vec![1.0]));
        let result = track.instantiate_clip(clip_1, 0);
        assert_eq!(clip_1, result.clip_id);

        let clip_2 = db.add(Clip::new(vec![2.0]));
        let result = track.instantiate_clip(clip_2, 1);
        assert_eq!(clip_2, result.clip_id);
    }

    #[test]
    fn test_last_clip() {
        let mut track = Track::new();
        let mut db = ClipDatabase::new();

        assert!(track.last_clip(&db).is_none(),
                "last clip must be None when track does not contain clips");

        // 1. only clip in the track should be the last clip
        // last
        // v
        // 1--------------
        let clip = db.add(Clip::new(vec![1.0]));
        track.instantiate_clip(clip, 0);
        let last_clip = track.last_clip(&db)
            .expect("last clip must be present when track contains clips");

        assert_eq!(clip, last_clip.clip_id);

        // 2. two disjoint clips, latter should be the last clip
        //   last
        //   v
        // 1-2------------
        let clip = db.add(Clip::new(vec![2.0]));
        track.instantiate_clip(clip, 2);
        let last_clip = track.last_clip(&db)
            .expect("last clip must be present when track contains clips");

        assert_eq!(clip, last_clip.clip_id);

        // 3. overlapping clips, clip with latest end should be the last clip
        //          last
        //          v
        // 1-2------------
        // 3333333333-----
        let clip = db.add(Clip::new(vec![3.0; 10]));
        track.instantiate_clip(clip, 0);
        let last_clip = track.last_clip(&db)
            .expect("last clip must be present when track contains clips");

        assert_eq!(clip, last_clip.clip_id);
    }

    #[test]
    fn test_clip_at() {
        let mut track = Track::new();
        let mut db = ClipDatabase::new();

        assert!(track.clip_at(&db, 0).is_none());

        let clip = db.add(Clip::new(vec![1.0, 1.0]));
        track.instantiate_clip(clip, 0);

        assert_eq!(clip, track.clip_at(&db, 0).unwrap().clip_id);
        assert_eq!(clip, track.clip_at(&db, 1).unwrap().clip_id);
        assert!(track.clip_at(&db, 2).is_none());

        let short_clip = db.add(Clip::new(vec![2.0]));
        track.instantiate_clip(short_clip, 1);

        assert_eq!(clip, track.clip_at(&db, 0).unwrap().clip_id);
        assert_eq!(short_clip, track.clip_at(&db, 1).unwrap().clip_id,
                   "latest added clip should take precedence when overlapping");
        assert!(track.clip_at(&db, 2).is_none());
    }

    #[test]
    fn test_next_clip() {
        let mut track = Track::new();
        let mut db = ClipDatabase::new();

        assert!(track.next_clip(0).is_none());

        // 1. query during clip should not return clip
        // t
        // v        next=None
        // 11-----------
        let clip = db.add(Clip::new(vec![1.0, 1.0]));
        track.instantiate_clip(clip, 0);
        assert!(track.next_clip(0).is_none(),
                "next_clip should not return clips where start <= t < end");

        // 2. two disjoint clips, query during first should return second
        // t  next
        // v  v
        // 11-2---------
        let clip = db.add(Clip::new(vec![2.0]));
        track.instantiate_clip(clip, 3);
        let result = track.next_clip(0).unwrap();
        assert_eq!(3, result.time);
        assert_eq!(clip, result.clip_id);
        assert!(track.next_clip(3).is_none());

        // 3. clips overlapping at t, query before overlap should return overlapping clip
        // t
        // v
        // 11-2---------
        // -3-----------
        //  ^
        //  next
        let clip = db.add(Clip::new(vec![3.0]));
        track.instantiate_clip(clip, 1);
        let result = track.next_clip(0).unwrap();
        assert_eq!(1, result.time);
        assert_eq!(clip, result.clip_id);

        // 4. t is past end of all clips, query should return none
        assert!(track.next_clip(1234).is_none());
    }

    #[test]
    fn test_render() {
        {
            let track = Track::new();
            let db = ClipDatabase::new();
            let mut buf = vec![0.0; 4];
            track.render(&db, 0, &mut buf);
            assert_eq!(buf, vec![0.0; 4]);
        }

        // clip that matches window exactly
        // [  ]
        // 1111
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 4];

            track.instantiate_clip(db.add(Clip::new(vec![1.0; 4])), 0);
            track.render(&db, 0, &mut buf);
            assert_eq!(buf, vec![1.0; 4])
        }

        // window expands past clip on right
        //   [  ]
        // 1111--
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 4];
            track.instantiate_clip(db.add(Clip::new(vec![1.0; 4])), 0);
            track.render(&db, 2, &mut buf);
            assert_eq!(buf, vec![1.0, 1.0, 0.0, 0.0])
        }

        // window expands past clip on left
        // [  ]
        // --1111
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 4];
            track.instantiate_clip(db.add(Clip::new(vec![1.0; 4])), 2);
            track.render(&db, 0, &mut buf);
            assert_eq!(buf, vec![0.0, 0.0, 1.0, 1.0])
        }

        // window expands past clip on both sides
        // [    ]
        // --11--
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 6];
            track.instantiate_clip(db.add(Clip::new(vec![1.0; 2])), 2);
            track.render(&db, 0, &mut buf);
            assert_eq!(buf, vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0])
        }

        // window beyond all clips
        //             [  ]
        // --11-- ... ------
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 4];
            track.instantiate_clip(db.add(Clip::new(vec![1.0; 2])), 2);
            track.render(&db, 100, &mut buf);
            assert_eq!(buf, vec![0.0; 4])
        }

        // window containing multiple clips
        // [ ]
        // 1-2
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 3];
            track.instantiate_clip(db.add(Clip::new(vec![1.0])), 0);
            track.instantiate_clip(db.add(Clip::new(vec![2.0])), 2);
            track.render(&db, 0, &mut buf);
            assert_eq!(buf, vec![1.0, 0.0, 2.0])
        }

        // window containing multiple clips past bounds
        //  [ ]
        // 11-22
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 3];
            track.instantiate_clip(db.add(Clip::new(vec![1.0; 2])), 0);
            track.instantiate_clip(db.add(Clip::new(vec![2.0; 2])), 3);
            track.render(&db, 1, &mut buf);
            assert_eq!(buf, vec![1.0, 0.0, 2.0])
        }

        // window containing multiple overlapping clips
        // [    ]
        // 1111--
        // --2222
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 6];
            track.instantiate_clip(db.add(Clip::new(vec![1.0; 4])), 0);
            track.instantiate_clip(db.add(Clip::new(vec![2.0; 4])), 2);
            track.render(&db, 0, &mut buf);
            assert_eq!(buf, vec![1.0, 1.0, 2.0, 2.0, 2.0, 2.0])
        }

        // window containing short overlapping clip
        // [    ]
        // 111111
        // --22--
        {
            let mut track = Track::new();
            let mut db = ClipDatabase::new();
            let mut buf = vec![0.0; 6];
            track.instantiate_clip(db.add(Clip::new(vec![1.0; 6])), 0);
            track.instantiate_clip(db.add(Clip::new(vec![2.0; 2])), 2);
            track.render(&db, 0, &mut buf);
            assert_eq!(buf, vec![1.0, 1.0, 2.0, 2.0, 1.0, 1.0])
        }
    }
}
