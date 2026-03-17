use shakmaty::{Chess, Move};

#[derive(Clone)]
pub struct GameState {
    pub initial_position: Chess,
    pub moves: Vec<MoveRecord>,
}

#[derive(Clone)]
pub struct MoveRecord {
    pub move_played: Move,
    pub fen_before: String,
    pub fen_after: String,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            initial_position: Chess::default(),
            moves: Vec::new(),
        }
    }
}
