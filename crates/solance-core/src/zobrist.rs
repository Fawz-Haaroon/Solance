use std::sync::OnceLock;

use shakmaty::{Board, Color, Piece, Square};
use shakmaty::{Move, Position};

pub type ZobristKey = u64;

static TABLE: OnceLock<ZobristTable> = OnceLock::new();

pub struct ZobristTable {
    pub pieces: [[u64; 64]; 12],
    pub side: u64,
}

impl ZobristTable {
    fn new() -> Self {
        let mut seed: u64 = 0x9E3779B97F4A7C15;

        fn next(seed: &mut u64) -> u64 {
            *seed ^= *seed >> 12;
            *seed ^= *seed << 25;
            *seed ^= *seed >> 27;
            *seed = seed.wrapping_mul(0x2545F4914F6CDD1D);
            *seed
        }

        let mut pieces = [[0u64; 64]; 12];

        for p in 0..12 {
            for sq in 0..64 {
                pieces[p][sq] = next(&mut seed);
            }
        }

        let side = next(&mut seed);

        Self { pieces, side }
    }
}

pub fn table() -> &'static ZobristTable {
    TABLE.get_or_init(ZobristTable::new)
}

pub fn hash_board(board: &Board, side: Color) -> ZobristKey {
    let t = table();
    let mut key = 0u64;

    for sq in Square::ALL {
        if let Some(piece) = board.piece_at(sq) {
            let idx = piece_index(piece);
            key ^= t.pieces[idx][sq as usize];
        }
    }

    if side == Color::Black {
        key ^= t.side;
    }

    key
}

fn piece_index(p: Piece) -> usize {
    use shakmaty::Color::*;
    use shakmaty::Role::*;

    match (p.color, p.role) {
        (White, Pawn) => 0,
        (White, Knight) => 1,
        (White, Bishop) => 2,
        (White, Rook) => 3,
        (White, Queen) => 4,
        (White, King) => 5,
        (Black, Pawn) => 6,
        (Black, Knight) => 7,
        (Black, Bishop) => 8,
        (Black, Rook) => 9,
        (Black, Queen) => 10,
        (Black, King) => 11,
    }
}

// abstraction boundary — engine must NOT call hash_board directly

pub fn update_key(_prev: ZobristKey, pos: &shakmaty::Chess, mv: &Move) -> ZobristKey {
    let next = pos.clone().play(mv).unwrap();
    hash_board(next.board(), next.turn())
}
