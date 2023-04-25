pub mod sine;

pub trait Generator : Send {
    fn next(&mut self) -> f32;
    fn handle(&mut self, msg: midly::MidiMessage);
}
