use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Engine {
    _proc: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl Engine {
    pub fn spawn() -> Self {
        let mut proc = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("stockfish not found in PATH");

        let input = proc.stdin.take().unwrap();
        let output = proc.stdout.take().unwrap();

        let mut this = Self {
            _proc: proc,
            input,
            output: BufReader::new(output),
        };

        this.bootstrap();
        this
    }

    fn bootstrap(&mut self) {
        self.cmd("uci");
        self.wait("uciok");

        self.cmd("isready");
        self.wait("readyok");
    }

    fn cmd(&mut self, s: &str) {
        writeln!(self.input, "{s}").unwrap();
        self.input.flush().unwrap();
    }

    fn wait(&mut self, needle: &str) {
        let mut buf = String::new();

        loop {
            buf.clear();
            self.output.read_line(&mut buf).unwrap();

            if buf.contains(needle) {
                return;
            }
        }
    }

    pub fn eval(&mut self, fen: &str, depth: u32) -> (String, Option<i32>) {
        self.cmd(&format!("position fen {fen}"));
        self.cmd(&format!("go depth {depth}"));

        let mut score = None;
        let mut line = String::new();

        loop {
            line.clear();
            self.output.read_line(&mut line).unwrap();

            if line.starts_with("info") {
                if let Some(s) = parse_score(&line) {
                    score = Some(s);
                }
            }

            if line.starts_with("bestmove") {
                let mv = line.split_whitespace().nth(1).unwrap().to_string();
                return (mv, score);
            }
        }
    }
}

fn parse_score(line: &str) -> Option<i32> {
    let mut it = line.split_whitespace();

    while let Some(tok) = it.next() {
        match tok {
            "cp" => {
                return it.next()?.parse().ok();
            }
            "mate" => {
                let v: i32 = it.next()?.parse().ok()?;
                return Some(if v > 0 { 10_000 - v } else { -10_000 - v });
            }
            _ => {}
        }
    }

    None
}
