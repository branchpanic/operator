use std::f32::consts::PI;
use crate::generator::Generator;

pub struct SineGenerator {
    sample_rate: u32,
    phase: f32,
    note: u8,
    on: bool,
}

fn midi_note_to_hz(note: u8) -> f64 {
    440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
}

impl SineGenerator {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            note: 69,
            on: false,
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
        if !self.on {
            return 0.0;
        }

        let freq = midi_note_to_hz(self.note) as f32 / self.sample_rate as f32;
        self.phase += 2.0 * PI * freq;

        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        if self.phase < 0.0 {
            self.phase += 2.0 * PI;
        }

        self.phase.sin()
    }

    fn handle(&mut self, msg: midly::MidiMessage) {
        match msg {
            midly::MidiMessage::NoteOn { key, .. } => {
                self.note = key.into();
                self.on = true;
            }
            midly::MidiMessage::NoteOff { key, .. } => {
                let key: u8 = key.into();
                if self.note == key {
                    self.on = false;
                }
            }
            _ => ()
        }
    }
}
