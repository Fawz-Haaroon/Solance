use shakmaty::Move;

#[derive(Clone)]
pub struct MoveRecord {
    pub mv: Move,
    pub uci: String,
    pub fen_before: String,
    pub fen_after: String,
}

pub struct GameState {
    pub moves: Vec<MoveRecord>,
}

impl GameState {
    pub fn new() -> Self {
        Self { moves: Vec::new() }
    }
}
