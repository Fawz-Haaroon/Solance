use std::env;
use std::fs::File;
use std::io::BufReader;

use pgn_reader::{BufferedReader, Visitor};

use solance_engine::Engine;
use solance_parser::GameBuilder;

fn main() {
    let path = env::args().nth(1).expect("missing pgn file");

    let file = File::open(path).expect("failed to open file");
    let mut reader = BufferedReader::new(BufReader::new(file));

    let mut builder = GameBuilder::new();
    reader.read_game(&mut builder).expect("invalid pgn");

    let game = builder.end_game();

    let mut engine = Engine::spawn();

    for (i, m) in game.moves.iter().enumerate() {
        let (best, eval_before_raw) = engine.eval(&m.fen_before, 12);
        let (_, eval_after_raw) = engine.eval(&m.fen_after, 12);

        // side to move BEFORE move
        let before_white =
            m.fen_before.split_whitespace().nth(1) == Some("w");

        // side to move AFTER move
        let after_white =
            m.fen_after.split_whitespace().nth(1) == Some("w");

        let eval_before = eval_before_raw.map(|v| if before_white { v } else { -v });
        let eval_after = eval_after_raw.map(|v| if after_white { v } else { -v });

        let delta = match (eval_before, eval_after) {
            (Some(b), Some(a)) => Some(a - b),
            _ => None,
        };

        println!(
            "{:>3}. {:<8} | eval: {:>6?} → {:>6?} | Δ: {:>6?} | best: {}",
            i + 1,
            m.move_played,
            eval_before,
            eval_after,
            delta,
            best
        );
    }
}
