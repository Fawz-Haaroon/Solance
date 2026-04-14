use std::env;
use std::fs::File;
use std::io::BufReader;

use pgn_reader::{BufferedReader, Visitor};

use solance_engine::{Engine, Stockfish};
use solance_parser::GameBuilder;

fn main() {
    let path = env::args().nth(1).expect("missing pgn file");

    let file = File::open(path).expect("cannot open file");
    let mut reader = BufferedReader::new(BufReader::new(file));

    let mut builder = GameBuilder::new();
    reader.read_game(&mut builder).unwrap();
    let game = builder.end_game();

    let mut engine: Box<dyn Engine> = Box::new(Stockfish::new());
    engine.start_game();

    for (i, m) in game.moves.iter().enumerate() {
        // ---- BEFORE MOVE ----
        let eval_before = engine.evaluate(12);

        // apply move using UCI (engine correctness)
        engine.apply_move(&m.uci);

        // ---- AFTER MOVE ----
        let eval_after = engine.evaluate(12);

        // best move from BEFORE position
        let best_move = eval_before.best.clone().unwrap_or_default();

        // scores
        let best_score = eval_before.score;
        let played_score = eval_after.score;

        // centipawn loss
        let loss = match (best_score, played_score) {
            (Some(b), Some(p)) => Some(b - p),
            _ => None,
        };

        // classification
        let class = match loss {
            Some(l) if l <= 10   => "best",
            Some(l) if l <= 30   => "excellent",
            Some(l) if l <= 80   => "good",
            Some(l) if l <= 150  => "inaccuracy",
            Some(l) if l <= 300  => "mistake",
            Some(_)              => "blunder",
            None                 => "unknown",
        };

        println!(
            "{:>3}. {:<8} | loss: {:?} | {:<10} | best: {}",
            i + 1,
            m.mv,        // human-readable move
            loss,
            class,
            best_move
        );
    }
}
