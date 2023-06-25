use std::collections::HashSet;
use iced::keyboard::KeyCode;

const SCALE: [KeyCode; 13] = {
    use KeyCode::*;
    [A, W, S, E, D, F, T, G, Y, H, U, J, K]
};

const OCTAVE_UP: KeyCode = KeyCode::X;
const OCTAVE_DOWN: KeyCode = KeyCode::Z;

pub struct Keyboard {
    base: u8,
    velocity: midly::num::u7,
    keys_held: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            base: 60,
            velocity: 127.into(),
            keys_held: HashSet::new(),
        }
    }

    fn key_to_note(&self, key: &KeyCode) -> Option<midly::num::u7> {
        SCALE.iter()
            .position(|k| k == key)
            .map(|i| (self.base + i as u8).into())
    }

    pub fn update(&mut self, keys_down: &HashSet<KeyCode>) -> Vec<midly::MidiMessage> {
        let just_pressed = keys_down.difference(&self.keys_held);

        for key in just_pressed.clone() {
            if key == &OCTAVE_UP {
                self.base += 12;
            } else if key == &OCTAVE_DOWN {
                self.base -= 12;
            }
        }

        let notes_on = just_pressed
            .filter_map(|k| self.key_to_note(k))
            .map(|note| midly::MidiMessage::NoteOn {
                key: note,
                vel: self.velocity,
            });

        let notes_off = self.keys_held.difference(keys_down)
            .filter_map(|k| self.key_to_note(k))
            .map(|note| midly::MidiMessage::NoteOff {
                key: note,
                vel: self.velocity,
            });

        let result: Vec<midly::MidiMessage> = notes_on.chain(notes_off).collect();
        self.keys_held.clone_from(keys_down);

        result
    }
}