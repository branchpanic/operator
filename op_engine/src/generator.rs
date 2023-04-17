pub mod sine;

pub trait Generator {
    fn next(&mut self) -> f32;
}
