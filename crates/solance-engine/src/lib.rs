use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Debug, Clone)]
pub struct Candidate {
    pub mv: String,
    pub score: Option<i32>, // ALWAYS white-centric now
    pub rank: usize,
}

#[derive(Clone)]
struct EvalCache {
    multi: Vec<Candidate>,
    single: Option<i32>,
}

pub trait Engine {
    fn start_game(&mut self);
    fn apply_move(&mut self, mv: &str);
    fn eval_current_multi(&mut self, depth: u32) -> Vec<Candidate>;
    fn eval_current_single(&mut self, depth: u32) -> Option<i32>;
}

pub struct Stockfish {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,

    moves: Vec<String>,
    cache: HashMap<String, EvalCache>,

    white_to_move: bool, // ← NEW
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

        let mut s = Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
            moves: Vec::new(),
            cache: HashMap::new(),
            white_to_move: true,
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

    fn current_key(&self) -> String {
        self.moves.join(" ")
    }

    fn sync_position(&mut self) {
        let mut cmd = String::from("position startpos");

        if !self.moves.is_empty() {
            cmd.push_str(" moves ");
            cmd.push_str(&self.moves.join(" "));
        }

        self.send(&cmd);
    }

    fn normalize(&self, v: i32) -> i32 {
        if self.white_to_move {
            v
        } else {
            -v
        }
    }

    fn compute_multi(&mut self, depth: u32) -> Vec<Candidate> {
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
                        "multipv" => {
                            rank = parts.get(i + 1).and_then(|v| v.parse::<usize>().ok())
                        }
                        "cp" => {
                            score = parts.get(i + 1).and_then(|v| v.parse::<i32>().ok())
                        }
                        "pv" => {
                            mv = parts.get(i + 1).map(|v| v.to_string())
                        }
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
        out
    }

    fn compute_single(&mut self, depth: u32) -> Option<i32> {
        self.sync_position();
        self.send(&format!("go depth {depth}"));

        let mut line = String::new();
        let mut score = None;

        loop {
            line.clear();
            self.stdout.read_line(&mut line).unwrap();

            if line.starts_with("info") && line.contains("score cp") {
                let parts: Vec<&str> = line.split_whitespace().collect();

                for i in 0..parts.len() {
                    if parts[i] == "cp" {
                        if let Ok(v) = parts[i + 1].parse::<i32>() {
                            score = Some(self.normalize(v));
                        }
                    }
                }
            }

            if line.starts_with("bestmove") {
                return score;
            }
        }
    }
}

impl Engine for Stockfish {
    fn start_game(&mut self) {
        self.moves.clear();
        self.cache.clear();
        self.white_to_move = true;

        self.send("ucinewgame");
        self.send("isready");
        self.wait_for("readyok");
    }

    fn apply_move(&mut self, mv: &str) {
        self.moves.push(mv.to_string());
        self.white_to_move = !self.white_to_move; // ← CRITICAL
    }

    fn eval_current_multi(&mut self, depth: u32) -> Vec<Candidate> {
        let key = self.current_key();

        if let Some(cached) = self.cache.get(&key) {
            return cached.multi.clone();
        }

        let multi = self.compute_multi(depth);
        let single = multi.get(0).and_then(|c| c.score);

        self.cache.insert(
            key,
            EvalCache {
                multi: multi.clone(),
                single,
            },
        );

        multi
    }

    fn eval_current_single(&mut self, depth: u32) -> Option<i32> {
        let key = self.current_key();

        if let Some(cached) = self.cache.get(&key) {
            return cached.single;
        }

        let single = self.compute_single(depth);

        self.cache.insert(
            key,
            EvalCache {
                multi: Vec::new(),
                single,
            },
        );

        single
    }
}
