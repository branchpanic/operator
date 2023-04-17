use std::collections::HashSet;
use egui::Key;

const SCALE: [Key; 12] = {
    use egui::Key::*;
    [Z, S, X, D, C, V, G, B, H, N, J, M]
};

pub struct Keyboard {
    base: u8,
    velocity: midly::num::u7,
    keys_held: HashSet<Key>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            base: 60,
            velocity: 127.into(),
            keys_held: HashSet::new(),
        }
    }

    fn key_to_note(&self, key: &Key) -> Option<midly::num::u7> {
        SCALE.iter()
            .position(|k| k == key)
            .map(|i| (self.base + i as u8).into())
    }

    pub fn update(&mut self, keys_down: &HashSet<Key>) -> Vec<midly::MidiMessage> {
        let notes_on = keys_down.difference(&self.keys_held)
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