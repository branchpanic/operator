use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Clip;

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct ClipId(usize);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClipDatabase {
    clips: HashMap<ClipId, Clip>,
}

impl ClipDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, clip: Clip) -> ClipId {
        let id = ClipId(self.clips.len());
        self.clips.insert(id, clip);
        id
    }

    pub fn get(&self, id: ClipId) -> Option<&Clip> {
        self.clips.get(&id)
    }
}
