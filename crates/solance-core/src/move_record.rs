#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub mv: String,
    pub uci: String,
    pub fen_before: String,
    pub fen_after: String,
}
