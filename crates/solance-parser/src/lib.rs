use pgn_reader::{SanPlus, Visitor};
use shakmaty::{
    fen::Fen,
    EnPassantMode,
    Chess,
    Position,
};

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
        let mv = san_plus.san.to_move(&self.state).unwrap();

        let fen_before =
            Fen::from_position(self.state.clone(), EnPassantMode::Legal).to_string();

        self.state.play_unchecked(&mv);

        let fen_after =
            Fen::from_position(self.state.clone(), EnPassantMode::Legal).to_string();

        self.game.moves.push(MoveRecord {
            move_played: mv,
            fen_before,
            fen_after,
        });
    }

    fn end_game(&mut self) -> GameState {
        self.game.clone()
    }
}
