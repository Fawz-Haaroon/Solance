use std::env;
use std::fs::File;
use std::io::BufReader;

use pgn_reader::{BufferedReader, Visitor};

use solance_engine::{Engine, Stockfish};
use solance_parser::GameBuilder;

fn normalize_move(m: &str) -> String {
    m.replace('-', "")
}

fn main() {
    let path = env::args().nth(1).expect("missing pgn file");

    let file = File::open(path).expect("cannot open file");
    let mut reader = BufferedReader::new(BufReader::new(file));

    let mut builder = GameBuilder::new();
    reader.read_game(&mut builder).unwrap();
    let game = builder.end_game();

    let mut engine: Box<dyn Engine> = Box::new(Stockfish::new());

    // NEW: initialize engine state
    engine.start_game();

    for (i, m) in game.moves.iter().enumerate() {
        let played = normalize_move(&m.move_played.to_string());

        // BEFORE applying move → evaluate position
        let candidates = engine.eval_current_multi(12);

        let mut played_rank = None;
        let mut best = String::new();
        let mut best_score = None;

        for c in &candidates {
            let mv = c.mv.trim();

            if c.rank == 1 {
                best = mv.to_string();
                best_score = c.score;
            }

            if played == mv {
                played_rank = Some(c.rank);
            }
        }

        // APPLY MOVE (state mutation)
        engine.apply_move(&played);

        // AFTER move → evaluate new position
        let played_score = engine.eval_current_single(12);

        let loss = match (best_score, played_score) {
            (Some(b), Some(p)) => Some(b - p),
            _ => None,
        };

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
            "{:>3}. {:<8} | rank: {:?} | loss: {:?} | {:<10} | best: {}",
            i + 1,
            m.move_played,
            played_rank,
            loss,
            class,
            best
        );
    }
}
