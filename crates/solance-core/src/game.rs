use crate::AnnotatedMove;

#[derive(Debug, Clone, Default)]
pub struct GameMeta {
    pub white:  Option<String>,
    pub black:  Option<String>,
    pub event:  Option<String>,
    pub date:   Option<String>,
    pub result: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub meta:  GameMeta,
    pub moves: Vec<AnnotatedMove>,
}

impl Game {
    pub fn new() -> Self {
        Self { meta: GameMeta::default(), moves: Vec::new() }
    }
}
