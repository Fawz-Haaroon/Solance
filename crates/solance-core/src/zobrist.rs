use std::sync::OnceLock;

use shakmaty::{Board, Color, Piece, Square};

pub type ZobristKey = u64;

static TABLE: OnceLock<ZobristTable> = OnceLock::new();

pub struct ZobristTable {
    pub pieces: [[u64; 64]; 12],
    pub side:   u64,
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

        Self { pieces, side: next(&mut seed) }
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
            key ^= t.pieces[piece_index(piece)][sq as usize];
        }
    }
    if side == Color::Black {
        key ^= t.side;
    }
    key
}

pub fn update_key(
    prev: ZobristKey,
    board: &Board,
    mv: &shakmaty::Move,
    _side: Color,
) -> ZobristKey {
    let t = table();
    let mut key = prev;

    let from    = mv.from().unwrap();
    let to      = mv.to();
    let moving  = board.piece_at(from).unwrap();

    key ^= t.pieces[piece_index(moving)][from as usize];

    if moving.role == shakmaty::Role::King {
        let file_diff = (from.file() as i8 - to.file() as i8).abs();
        if file_diff == 2 {
            let rank = from.rank();
            let (rook_from, rook_to) = if to.file() > from.file() {
                (
                    shakmaty::Square::from_coords(shakmaty::File::H, rank),
                    shakmaty::Square::from_coords(shakmaty::File::F, rank),
                )
            } else {
                (
                    shakmaty::Square::from_coords(shakmaty::File::A, rank),
                    shakmaty::Square::from_coords(shakmaty::File::D, rank),
                )
            };
            let rook = board.piece_at(rook_from).unwrap();
            key ^= t.pieces[piece_index(rook)][rook_from as usize];
            key ^= t.pieces[piece_index(rook)][rook_to as usize];
        }
    }

    if let Some(captured) = board.piece_at(to) {
        key ^= t.pieces[piece_index(captured)][to as usize];
    }

    if moving.role == shakmaty::Role::Pawn {
        if from.file() != to.file() && board.piece_at(to).is_none() {
            let ep_sq  = shakmaty::Square::from_coords(to.file(), from.rank());
            let pawn   = board.piece_at(ep_sq).unwrap();
            key ^= t.pieces[piece_index(pawn)][ep_sq as usize];
        }
    }

    let placed = match mv.promotion() {
        Some(role) => Piece { color: moving.color, role },
        None       => moving,
    };
    key ^= t.pieces[piece_index(placed)][to as usize];

    key ^= t.side;
    key
}

fn piece_index(p: Piece) -> usize {
    use shakmaty::Color::*;
    use shakmaty::Role::*;
    match (p.color, p.role) {
        (White, Pawn)   => 0,
        (White, Knight) => 1,
        (White, Bishop) => 2,
        (White, Rook)   => 3,
        (White, Queen)  => 4,
        (White, King)   => 5,
        (Black, Pawn)   => 6,
        (Black, Knight) => 7,
        (Black, Bishop) => 8,
        (Black, Rook)   => 9,
        (Black, Queen)  => 10,
        (Black, King)   => 11,
    }
}
