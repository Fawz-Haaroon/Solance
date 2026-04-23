use pgn_reader::{RawHeader, SanPlus, Skip, Visitor};
use shakmaty::{Chess, EnPassantMode, Position, san::San};
use shakmaty::fen::Fen;

use solance_core::{
    AnnotatedMove, Game, GameMeta,
    zobrist::{hash_board, update_key},
};

#[derive(Debug)]
pub enum ParseError {
    InvalidSan(String),
    IllegalMove(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSan(s)  => write!(f, "invalid SAN: {s}"),
            Self::IllegalMove(s) => write!(f, "illegal move: {s}"),
        }
    }
}

impl std::error::Error for ParseError {}

pub struct GameBuilder {
    position: Chess,
    game:     Game,
    error:    Option<ParseError>,
}

impl GameBuilder {
    pub fn new() -> Self {
        let position = Chess::default();
        Self { position, game: Game::new(), error: None }
    }
}

impl Visitor for GameBuilder {
    type Result = Result<Game, ParseError>;

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        let v = value.decode_utf8().map(|s| s.into_owned()).ok();
        match key {
            b"White"  => self.game.meta.white  = v,
            b"Black"  => self.game.meta.black  = v,
            b"Event"  => self.game.meta.event  = v,
            b"Date"   => self.game.meta.date   = v,
            b"Result" => self.game.meta.result = v,
            _ => {}
        }
    }

    fn san(&mut self, san_plus: SanPlus) {
        if self.error.is_some() {
            return;
        }

        let san_str = san_plus.san.to_string();

        let mv = match San::from_ascii(san_str.as_bytes()) {
            Ok(san) => match san.to_move(&self.position) {
                Ok(m)  => m,
                Err(_) => {
                    self.error = Some(ParseError::IllegalMove(san_str));
                    return;
                }
            },
            Err(_) => {
                self.error = Some(ParseError::InvalidSan(san_str));
                return;
            }
        };

        let uci        = mv.to_uci(shakmaty::CastlingMode::Standard).to_string();
        let key_before = hash_board(self.position.board(), self.position.turn());
        let fen_before = Fen::from_position(self.position.clone(), EnPassantMode::Legal).to_string();

        self.position.play_unchecked(&mv);

        let key_after = hash_board(self.position.board(), self.position.turn());
        let fen_after = Fen::from_position(self.position.clone(), EnPassantMode::Legal).to_string();

        self.game.moves.push(AnnotatedMove {
            uci,
            san: san_str,
            fen_before,
            fen_after,
            key_before,
            key_after,
        });
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        match self.error.take() {
            Some(e) => Err(e),
            None    => Ok(self.game.clone()),
        }
    }
}
