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

    pub fn add_clip(&mut self, clip: Clip) -> &Clip {
        self.clips.push(clip);
        self.clips.last().unwrap()
    }

    pub fn add_new_clip(&mut self, at: Time, data: &[f32]) -> &Clip {
        self.add_clip(Clip::new(at, Vec::from(data)))
    }

    fn last_clip(&self) -> Option<&Clip> {
        // Note: assumes that overlapping clips cut previous ones.
        // c1:   ---xxxxx------
        // c2:   yyyyyyyyyyyy--
        // cut:  yyyxxxxx------
        self.clips.iter().max_by_key(|c| { c.start })
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

    fn clip_at(&self, t: Time) -> Option<&Clip> {
        self.clips.iter().find(|c| { c.start <= t && c.end() > t })
    }

    pub fn render_all(&self) -> Vec<f32> {
        let end = match self.last_clip() {
            None => return Vec::new(),
            Some(last_clip) => last_clip.end(),
        };

        let mut buf = vec![0.0; end];
        self.render(0, buf.as_mut_slice());
        buf
    }

    pub fn render(&self, start: Time, into: &mut [f32]) {
        into.fill(0.0);

        let render_end = into.len();
        let mut opt_clip = self.clip_at(start);

        if opt_clip.is_none() {
            opt_clip = self.next_clip(start);
        }

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
        session.render(0, &mut buf);
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
        session.render(0, &mut buf);
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
        session.render(0, &mut buf);
        // Could possibly change the expected behavior to:
        //  (a)  [1, 1, 2, 1, 1]
        // Instead of:
        //  (b)  [1, 1, 2, 0, 0]
        // However, we might want to implement (a) using overdubbing instead.
        assert_eq!(buf, [1.0f32, 1.0f32, 2.0f32, 0.0f32, 0.0f32]);
    }
}
