use crate::Time;

#[derive(Default, Debug, Clone)]
pub struct Clip {
    pub start: Time,
    pub data: Vec<f32>,
}

impl Clip {
    pub fn new(start: Time, data: Vec<f32>) -> Self {
        Clip { start, data }
    }

    pub fn end(&self) -> Time {
        self.start + self.data.len() as Time
    }
}
