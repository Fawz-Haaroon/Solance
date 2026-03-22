use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Debug, Clone)]
pub struct Candidate {
    pub mv: String,
    pub score: Option<i32>,
    pub rank: usize,
}

/*
    This is the boundary.

    Everything above CLI talks to THIS, not Stockfish.
*/
pub trait Engine {
    fn eval_multi(&mut self, fen: &str, depth: u32) -> Vec<Candidate>;
    fn eval_single(&mut self, fen: &str, depth: u32) -> Option<i32>;
}

/*
    Concrete implementation: Stockfish (UCI)
*/
pub struct Stockfish {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
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
}

/*
    Engine trait implementation (THIS is what CLI uses)
*/
impl Engine for Stockfish {
    fn eval_multi(&mut self, fen: &str, depth: u32) -> Vec<Candidate> {
        self.send(&format!("position fen {fen}"));
        self.send("setoption name MultiPV value 5");
        self.send(&format!("go depth {depth}"));

        let mut line = String::new();
        let mut out = Vec::new();

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
                            rank = parts.get(i + 1).and_then(|v| v.parse::<usize>().ok());
                        }
                        "cp" => {
                            score = parts.get(i + 1).and_then(|v| v.parse::<i32>().ok());
                        }
                        "pv" => {
                            mv = parts.get(i + 1).map(|v| v.to_string());
                        }
                        _ => {}
                    }
                }

                if let (Some(rank), Some(mv), Some(score)) = (rank, mv, score) {
                    if !out.iter().any(|c| c.rank == rank) {
                        out.push(Candidate {
                            mv,
                            score: Some(score),
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

    fn eval_single(&mut self, fen: &str, depth: u32) -> Option<i32> {
        self.send(&format!("position fen {fen}"));
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
                            score = Some(v);
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
