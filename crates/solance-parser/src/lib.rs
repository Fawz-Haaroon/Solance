use pgn_reader::{SanPlus, Visitor};
use shakmaty::{Chess, Position, Move};
use shakmaty::fen::Fen;

use solance_core::{GameState, MoveRecord};

pub struct GameBuilder {
    state: Chess,
    game: GameState,
}

impl GameBuilder {
    pub fn new() -> Self {
        Self {
            state: Chess::default(),
            game: GameState::new(),
        }
    }
}

impl Visitor for GameBuilder {
    type Result = GameState;

    fn san(&mut self, san_plus: SanPlus) {
        let mv: Move = san_plus.san.to_move(&self.state).unwrap();

        let fen_before = Fen::from_position(
            self.state.clone(),
            shakmaty::EnPassantMode::Legal
        ).to_string();

        let uci = mv.to_string(); // THIS is already UCI (important)

        self.state.play_unchecked(&mv);

        let fen_after = Fen::from_position(
            self.state.clone(),
            shakmaty::EnPassantMode::Legal
        ).to_string();

        self.game.moves.push(MoveRecord {
            mv: mv.to_string(),
            uci,
            fen_before,
            fen_after,
        });

    }

    fn end_game(&mut self) -> GameState {
        GameState {
            moves: self.game.moves.clone(),
        }
    }
}
