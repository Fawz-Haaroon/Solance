use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

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

        let mut engine = Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        };

        engine.init();
        engine
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

    pub fn evaluate(&mut self, fen: &str, depth: u32) -> (String, Option<i32>) {
        self.send(&format!("position fen {fen}"));
        self.send(&format!("go depth {depth}"));

        let mut score: Option<i32> = None;
        let mut line = String::new();

        loop {
            line.clear();
            self.stdout.read_line(&mut line).unwrap();

            // parse score
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

            // return immediately when bestmove arrives
            if line.starts_with("bestmove") {
                let best_move = line
                    .split_whitespace()
                    .nth(1)
                    .expect("no bestmove returned")
                    .to_string();

                return (best_move, score);
            }
        }
    }
}
