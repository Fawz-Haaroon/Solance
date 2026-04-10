use crate::MoveRecord;

#[derive(Debug, Clone)]
pub struct GameState {
    pub moves: Vec<MoveRecord>,
}

impl GameState {
    pub fn new() -> Self {
        Self { moves: Vec::new() }
    }
}
