use std::{env, fs::File, io::BufReader};

use pgn_reader::{BufferedReader, Visitor};

use solance_engine::Stockfish;
use solance_parser::GameBuilder;

fn normalize_move(m: &str) -> String {
    let s = m.replace('-', "").to_lowercase();

    match s.len() {
        4 => s,            // e2e4
        5 => s[1..].into(), // ng1f3 -> g1f3
        _ => s,
    }
}

fn main() {
    let path = env::args().nth(1).expect("missing pgn file");

    let file = File::open(path).expect("cannot open file");
    let mut reader = BufferedReader::new(BufReader::new(file));

    let mut builder = GameBuilder::new();
    reader.read_all(&mut builder).unwrap();

    let game = builder.end_game();

    let mut engine = Stockfish::new();

    for (i, m) in game.moves.iter().enumerate() {
        let (best_raw, eval_before_raw) = engine.eval(&m.fen_before, 12);
        let best = best_raw.trim().to_string();

        let (_, eval_after_raw) = engine.eval(&m.fen_after, 12);

        let before_white =
            m.fen_before.split_whitespace().nth(1) == Some("w");
        let after_white =
            m.fen_after.split_whitespace().nth(1) == Some("w");

        let eval_before: Option<i32> =
            eval_before_raw.map(|v| if before_white { v } else { -v });

        let eval_after: Option<i32> =
            eval_after_raw.map(|v| if after_white { v } else { -v });

        let delta = match (eval_before, eval_after) {
            (Some(b), Some(a)) => Some(a - b),
            _ => None,
        };

        let played = normalize_move(&m.move_played.to_string());

        let class = if played == best {
            "best"
        } else {
            match delta {
                Some(d) if d > -20   => "good",
                Some(d) if d > -50   => "inaccuracy",
                Some(d) if d > -100  => "mistake",
                Some(_)              => "blunder",
                None                 => "unknown",
            }
        };

        println!(
            "{:>3}. {:<8} | eval: {:>6?} → {:>6?} | Δ: {:>5?} | {:<11} | best: {}",
            i + 1,
            m.move_played,
            eval_before,
            eval_after,
            delta,
            class,
            best
        );
    }
}
