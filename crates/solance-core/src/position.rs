use crate::zobrist::ZobristKey;

#[derive(Debug, Clone)]
pub struct AnnotatedMove {
    pub uci:        String,
    pub san:        String,
    pub fen_before: String,
    pub fen_after:  String,
    pub key_before: ZobristKey,
    pub key_after:  ZobristKey,
}
