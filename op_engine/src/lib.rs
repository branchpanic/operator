mod track;
mod clip;
mod session;
mod player;

use std::cmp::{max, min};
pub use track::Track;
pub use clip::Clip;
pub use session::Session;
pub use player::Player;

type Time = usize;  // in samples

fn mix(sources: &[&[f32]], buf: &mut [f32]) {
    for i in 0..buf.len() {
        buf[i] = 0.0;
        for source in sources {
            if i >= source.len() {
                continue;
            }

            buf[i] += source[i];
        }

        buf[i] = (buf[i] / sources.len() as f32).max(-1.0).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix() {
        let c1 = [1.0f32, 1.0f32, 1.0f32, 1.0f32];
        let c2 = [1.0f32, 1.0f32, 1.0f32];
        let c3 = [1.0f32, 1.0f32];
        let c4 = [1.0f32];
        let mut result = [0f32; 5];
        mix(&[&c1, &c2, &c3, &c4], &mut result);
        assert_eq!(result, [1.0f32, 0.75f32, 0.5f32, 0.25f32, 0.0f32]);
    }
}
