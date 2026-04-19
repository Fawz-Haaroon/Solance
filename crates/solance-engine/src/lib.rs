use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use shakmaty::{Chess, Position};
use shakmaty::fen::Fen;
use shakmaty::EnPassantMode;

use solance_core::zobrist::{update_key, hash_board, ZobristKey};

#[derive(Debug, Clone)]
pub struct Candidate {
    pub mv: String,
    pub score: Option<i32>,
    pub rank: usize,
}

#[derive(Debug, Clone)]
pub struct Evaluation {
    pub best: Option<String>,
    pub score: Option<i32>,
    pub candidates: Vec<Candidate>,
}

#[derive(Clone)]
struct EvalCache {
    eval: Evaluation,
}

pub trait Engine {
    fn start_game(&mut self);
    fn apply_move(&mut self, mv: &str);
    fn evaluate(&mut self, depth: u32) -> Evaluation;
    fn evaluate_after_move(&mut self, mv: &str, depth: u32) -> Option<i32>;
}

pub struct Stockfish {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,

    cache: HashMap<ZobristKey, EvalCache>,

    position: Chess,
    current_key: ZobristKey,
}

impl Stockfish {
    pub fn new() -> Self {
        let mut child = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start stockfish");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let position = Chess::default();
        let key = hash_board(position.board(), position.turn());

        let mut s = Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
            cache: HashMap::new(),
            position,
            current_key: key,
        };

        s.init();
        s
    }

    fn init(&mut self) {
        self.send("uci");
        self.wait_for("uciok");

        self.send("isready");
        self.wait_for("readyok");
    }

    fn send(&mut self, cmd: &str) {
        writeln!(self.stdin, "{cmd}").unwrap();
    }

    fn wait_for(&mut self, token: &str) {
        let mut line = String::new();
        loop {
            line.clear();
            self.stdout.read_line(&mut line).unwrap();
            if line.contains(token) {
                break;
            }
        }
    }

    fn sync_position(&mut self) {
        let fen = Fen::from_position(self.position.clone(), EnPassantMode::Legal).to_string();
        self.send(&format!("position fen {}", fen));
    }

    fn normalize(&self, v: i32) -> i32 {
        if self.position.turn().is_white() { v } else { -v }
    }

    fn compute(&mut self, depth: u32) -> Evaluation {
        self.sync_position();

        self.send("setoption name MultiPV value 5");
        self.send(&format!("go depth {depth}"));

        let mut line = String::new();
        let mut out: Vec<Candidate> = Vec::new();

        loop {
            line.clear();
            self.stdout.read_line(&mut line).unwrap();

            if line.starts_with("info") && line.contains("multipv") {
                let parts: Vec<&str> = line.split_whitespace().collect();

                let mut rank = None;
                let mut score = None;
                let mut mv = None;

                for i in 0..parts.len() {
                    match parts[i] {
                        "multipv" => rank = parts.get(i + 1).and_then(|v| v.parse().ok()),
                        "cp" => score = parts.get(i + 1).and_then(|v| v.parse().ok()),
                        "pv" => mv = parts.get(i + 1).map(|v| v.to_string()),
                        _ => {}
                    }
                }

                if let (Some(rank), Some(mv), Some(score)) = (rank, mv, score) {
                    if !out.iter().any(|c| c.rank == rank) {
                        out.push(Candidate {
                            mv,
                            score: Some(self.normalize(score)),
                            rank,
                        });
                    }
                }
            }

            if line.starts_with("bestmove") {
                break;
            }
        }

        out.sort_by_key(|c| c.rank);

        let best = out.get(0).map(|c| c.mv.clone());
        let score = out.get(0).and_then(|c| c.score);

        Evaluation { best, score, candidates: out }
    }

    fn evaluate_after_move_internal(&mut self, mv: &str, depth: u32) -> Option<i32> {
        let original = self.position.clone();
        let original_key = self.current_key;

        self.apply_move(mv);
        let score = self.evaluate(depth).score;

        self.position = original;
        self.current_key = original_key;

        score
    }
}

impl Engine for Stockfish {
    fn start_game(&mut self) {
        self.cache.clear();
        self.position = Chess::default();
        self.current_key = hash_board(self.position.board(), self.position.turn());

        self.send("ucinewgame");
        self.send("isready");
        self.wait_for("readyok");
    }

    fn apply_move(&mut self, mv: &str) {
        let cleaned = mv.replace("-", "");

        let m = if let Ok(uci) = cleaned.parse::<shakmaty::uci::Uci>() {
            uci.to_move(&self.position).unwrap()
        } else {
            let stripped: String = mv
                .chars()
                .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                .collect();

            self.position.legal_moves().into_iter().find(|m| {
                let from = m.from().unwrap().to_string();
                let to = m.to().to_string();
                format!("{}{}", from, to) == stripped
            }).expect("unresolvable move")
        };

        let next_key = update_key(
            self.current_key,
            self.position.board(),
            &m,
            self.position.turn(),
        );

        self.position = self.position.clone().play(&m).unwrap();
        self.current_key = next_key;
    }

    fn evaluate(&mut self, depth: u32) -> Evaluation {
        let key = self.current_key;

        if let Some(cached) = self.cache.get(&key) {
            return cached.eval.clone();
        }

        let eval = self.compute(depth);
        self.cache.insert(key, EvalCache { eval: eval.clone() });

        eval
    }

    fn evaluate_after_move(&mut self, mv: &str, depth: u32) -> Option<i32> {
        self.evaluate_after_move_internal(mv, depth)
    }
}
