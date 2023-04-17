use dasp::{signal, Signal};
use crate::generator::Generator;

pub struct SineGenerator {
    osc: signal::Sine<signal::ConstHz>,
}

impl SineGenerator {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            osc: signal::rate(sample_rate as f64).const_hz(440.0).sine(),
        }
    }
}

impl Default for SineGenerator {
    fn default() -> Self {
        Self::new(44100)
    }
}

impl Generator for SineGenerator {
    fn next(&mut self) -> f32 {
        self.osc.next() as f32
    }
}
