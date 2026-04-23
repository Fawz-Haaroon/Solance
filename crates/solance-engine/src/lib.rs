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
    pub fn centipawns(&self) -> Option<i32> {
        match self {
            Score::Cp(n) => Some(*n),
            Score::Mate(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub mv:    String,
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
    SpawnFailed(String, std::io::Error),
    InvalidMove(String),
    NotReady(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawnFailed(bin, e) => write!(f, "failed to spawn '{bin}': {e}"),
            Self::InvalidMove(mv)     => write!(f, "move not legal in current position: {mv}"),
            Self::NotReady(msg)       => write!(f, "engine not ready: {msg}"),
        }
    }
}

impl std::error::Error for EngineError {}

// The contract every engine implementation must satisfy.
// Implementations own their internal position state — callers never
// pass FEN directly, they drive state through reset/apply_move.
pub trait Engine: Send {
    fn name(&self) -> &str;
    fn reset(&mut self);
    fn apply_move(&mut self, uci: &str) -> Result<(), EngineError>;
    fn evaluate(&mut self, depth: u32) -> Evaluation;
}

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
        Self::launch_from("stockfish")
    }

    pub fn launch_from(binary: &str) -> Result<Self, EngineError> {
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| EngineError::SpawnFailed(binary.to_owned(), e))?;

        let stdin  = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let position    = Chess::default();
        let current_key = hash_board(position.board(), position.turn());

        let mut sf = Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
            cache: HashMap::new(),
            position,
            current_key,
        };

        sf.send("uci");
        sf.await_token("uciok");
        sf.send("isready");
        sf.await_token("readyok");

        Ok(sf)
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

        let white_to_move       = self.position.turn().is_white();
        let mut candidates: Vec<Candidate> = Vec::new();
        let mut line            = String::new();

        loop {
            line.clear();
            self.stdout.read_line(&mut line).unwrap();

            if line.starts_with("info") && line.contains("multipv") {
                if let Some(c) = parse_info_line(&line, white_to_move) {
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

impl Engine for Stockfish {
    fn name(&self) -> &str {
        "Stockfish"
    }

    fn reset(&mut self) {
        self.cache.clear();
        self.position    = Chess::default();
        self.current_key = hash_board(self.position.board(), self.position.turn());
        self.send("ucinewgame");
        self.send("isready");
        self.await_token("readyok");
    }

    fn apply_move(&mut self, uci: &str) -> Result<(), EngineError> {
        let parsed: Uci = uci.parse().map_err(|_| EngineError::InvalidMove(uci.to_owned()))?;
        let mv = parsed
            .to_move(&self.position)
            .map_err(|_| EngineError::InvalidMove(uci.to_owned()))?;

        let next_key     = update_key(self.current_key, self.position.board(), &mv, self.position.turn());
        self.position    = self.position.clone().play(&mv).unwrap();
        self.current_key = next_key;
        Ok(())
    }

    fn evaluate(&mut self, depth: u32) -> Evaluation {
        let key = self.current_key;
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        let eval = self.query_engine(depth);
        self.cache.insert(key, eval.clone());
        eval
    }
}

fn parse_info_line(line: &str, white_to_move: bool) -> Option<Candidate> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    let mut rank:     Option<usize>  = None;
    let mut score:    Option<Score>  = None;
    let mut pv_start: Option<usize>  = None;

    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "multipv" => rank = parts.get(i + 1).and_then(|v| v.parse().ok()),
            "cp" => {
                if let Some(raw) = parts.get(i + 1).and_then(|v| v.parse::<i32>().ok()) {
                    score = Some(Score::Cp(if white_to_move { raw } else { -raw }));
                }
            }
            "mate" => {
                if let Some(raw) = parts.get(i + 1).and_then(|v| v.parse::<i32>().ok()) {
                    score = Some(Score::Mate(if white_to_move { raw } else { -raw }));
                }
            }
            "pv" => { pv_start = Some(i + 1); break; }
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
