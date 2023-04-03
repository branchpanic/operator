use std::cmp::min;

use crate::clip::Clip;
use crate::Time;

#[derive(Default)]
pub struct Track {
    clips: Vec<Clip>,
}

impl Track {
    pub fn new() -> Track {
        Track::default()
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
    }

    fn first_clip(&self) -> Option<&Clip> {
        self.clips.iter().min_by_key(|c| { c.start })
    }

    fn next_clip(&self, t: Time) -> Option<&Clip> {
        let mut best: Option<&Clip> = None;

        for clip in &self.clips {
            if clip.start <= t {
                continue;
            }

            if let Some(prev_best) = best {
                if clip.start < prev_best.start {
                    best = Some(clip);
                }
            } else {
                best = Some(clip);
            }
        }

        best
    }

    pub fn render(&self, into: &mut [f32]) {
        into.fill(0.0f32);

        let render_end = into.len();
        let mut opt_clip = self.first_clip();

        while let Some(current_clip) = opt_clip {
            let t = current_clip.start;
            let mut end = min(current_clip.end(), render_end);

            let opt_next_clip = self.next_clip(t);
            if let Some(next_clip) = opt_next_clip {
                end = min(end, next_clip.end());
            }

            let copied_amt = end - current_clip.start;
            into[t..t + copied_amt].copy_from_slice(&current_clip.data[..copied_amt]);

            opt_clip = opt_next_clip;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_clip() {
        let clip = Clip::new(0, vec![1.0f32]);
        let mut session = Track::new();
        session.add_clip(clip.clone());

        if let Some(added) = session.clips.first()
        {
            assert_eq!(added.start, clip.start);
            assert_eq!(added.data, clip.data);
        } else {
            assert!(false, "clip was not added");
        }
    }

    #[test]
    fn test_render_overlapping() {
        let c1 = Clip::new(0, vec![1.0f32; 5]);
        let c2 = Clip::new(2, vec![2.0f32; 5]);
        let mut session = Track::new();
        session.add_clip(c1);
        session.add_clip(c2);
        let mut buf = [0f32; 8];
        session.render(&mut buf);
        assert_eq!(buf, [1.0f32, 1.0f32, 2.0f32, 2.0f32, 2.0f32, 2.0f32, 2.0f32, 0.0f32]);
    }

    #[test]
    fn test_render_non_overlapping() {
        let c1 = Clip::new(1, vec![1.0f32; 2]);
        let c2 = Clip::new(4, vec![2.0f32; 2]);
        let mut session = Track::new();
        session.add_clip(c1);
        session.add_clip(c2);
        let mut buf = [0f32; 7];
        session.render(&mut buf);
        assert_eq!(buf, [0.0f32, 1.0f32, 1.0f32, 0.0f32, 2.0f32, 2.0f32, 0.0f32]);
    }

    #[test]
    fn test_render_overlapping_cut() {
        let c1 = Clip::new(0, vec![1.0f32; 5]);
        let c2 = Clip::new(2, vec![2.0f32; 1]);
        let mut session = Track::new();
        session.add_clip(c1);
        session.add_clip(c2);
        let mut buf = [0f32; 5];
        session.render(&mut buf);
        // Could possibly change the expected behavior to:
        //  (a)  [1, 1, 2, 1, 1]
        // Instead of:
        //  (b)  [1, 1, 2, 0, 0]
        // However, we might want to implement (a) using overdubbing instead.
        assert_eq!(buf, [1.0f32, 1.0f32, 2.0f32, 0.0f32, 0.0f32]);
    }
}
