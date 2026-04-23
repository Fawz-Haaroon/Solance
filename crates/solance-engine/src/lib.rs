use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use shakmaty::{Chess, EnPassantMode, Position};
use shakmaty::fen::Fen;
use shakmaty::uci::Uci;

use solance_core::zobrist::{hash_board, update_key, ZobristKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Cp(i32),
    Mate(i32),
}

impl Score {
    // Normalized to white's perspective going in — negative means black is better.
    // Mate(n): positive n = white mates in n, negative = black mates in n.
    pub fn centipawns(&self) -> Option<i32> {
        match self {
            Score::Cp(n) => Some(*n),
            Score::Mate(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub mv:   String,
    pub score: Score,
    pub rank:  usize,
    pub pv:    Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Evaluation {
    pub candidates: Vec<Candidate>,
}

impl Evaluation {
    pub fn best(&self) -> Option<&Candidate> {
        self.candidates.iter().find(|c| c.rank == 1)
    }
}

#[derive(Debug)]
pub enum EngineError {
    SpawnFailed(std::io::Error),
    InvalidMove(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawnFailed(e)  => write!(f, "failed to spawn engine process: {e}"),
            Self::InvalidMove(mv) => write!(f, "move not legal in current position: {mv}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub struct Stockfish {
    _child:      Child,
    stdin:       ChildStdin,
    stdout:      BufReader<ChildStdout>,
    cache:       HashMap<ZobristKey, Evaluation>,
    position:    Chess,
    current_key: ZobristKey,
}

impl Stockfish {
    pub fn launch() -> Result<Self, EngineError> {
        let mut child = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(EngineError::SpawnFailed)?;

        let stdin  = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let position = Chess::default();
        let key      = hash_board(position.board(), position.turn());

        let mut sf = Self {
            _child:      child,
            stdin,
            stdout:      BufReader::new(stdout),
            cache:       HashMap::new(),
            position,
            current_key: key,
        };

        sf.send("uci");
        sf.await_token("uciok");
        sf.send("isready");
        sf.await_token("readyok");

        Ok(sf)
    }

    pub fn reset(&mut self) {
        self.cache.clear();
        self.position    = Chess::default();
        self.current_key = hash_board(self.position.board(), self.position.turn());

        self.send("ucinewgame");
        self.send("isready");
        self.await_token("readyok");
    }

    pub fn apply_move(&mut self, uci: &str) -> Result<(), EngineError> {
        let parsed: Uci = uci.parse().map_err(|_| EngineError::InvalidMove(uci.to_owned()))?;
        let mv = parsed
            .to_move(&self.position)
            .map_err(|_| EngineError::InvalidMove(uci.to_owned()))?;

        let next_key = update_key(
            self.current_key,
            self.position.board(),
            &mv,
            self.position.turn(),
        );

        self.position    = self.position.clone().play(&mv).unwrap();
        self.current_key = next_key;

        Ok(())
    }

    pub fn evaluate(&mut self, depth: u32) -> Evaluation {
        let key = self.current_key;

        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let eval = self.query_engine(depth);
        self.cache.insert(key, eval.clone());
        eval
    }

    fn send(&mut self, cmd: &str) {
        writeln!(self.stdin, "{cmd}").unwrap();
    }

    fn await_token(&mut self, token: &str) {
        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line).unwrap();
            if line.contains(token) {
                break;
            }
        }
    }

    fn query_engine(&mut self, depth: u32) -> Evaluation {
        let fen = Fen::from_position(self.position.clone(), EnPassantMode::Legal).to_string();
        self.send(&format!("position fen {fen}"));
        self.send("setoption name MultiPV value 5");
        self.send(&format!("go depth {depth}"));

        let white_to_move = self.position.turn().is_white();
        let mut candidates: Vec<Candidate> = Vec::new();
        let mut line = String::new();

        loop {
            line.clear();
            self.stdout.read_line(&mut line).unwrap();

            if line.starts_with("info") && line.contains("multipv") {
                if let Some(c) = parse_info_line(&line, white_to_move) {
                    // keep only the last info line per multipv rank (highest depth)
                    if let Some(existing) = candidates.iter_mut().find(|x| x.rank == c.rank) {
                        *existing = c;
                    } else {
                        candidates.push(c);
                    }
                }
            }

            if line.starts_with("bestmove") {
                break;
            }
        }

        candidates.sort_by_key(|c| c.rank);
        Evaluation { candidates }
    }
}

fn parse_info_line(line: &str, white_to_move: bool) -> Option<Candidate> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    let mut rank:    Option<usize>  = None;
    let mut score:   Option<Score>  = None;
    let mut pv_start: Option<usize> = None;

    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "multipv" => rank  = parts.get(i + 1).and_then(|v| v.parse().ok()),
            "cp"      => {
                if let Some(raw) = parts.get(i + 1).and_then(|v| v.parse::<i32>().ok()) {
                    // Stockfish always reports from the side to move's perspective
                    score = Some(Score::Cp(if white_to_move { raw } else { -raw }));
                }
            }
            "mate"    => {
                if let Some(raw) = parts.get(i + 1).and_then(|v| v.parse::<i32>().ok()) {
                    score = Some(Score::Mate(if white_to_move { raw } else { -raw }));
                }
            }
            "pv"      => { pv_start = Some(i + 1); break; }
            _ => {}
        }
        i += 1;
    }

    let pv: Vec<String> = pv_start
        .map(|s| parts[s..].iter().map(|t| t.to_string()).collect())
        .unwrap_or_default();

    let mv = pv.first()?.clone();

    Some(Candidate { mv, score: score?, rank: rank?, pv })
}
