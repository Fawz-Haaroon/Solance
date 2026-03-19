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
        let (best, eval_before) = engine.eval(&m.fen_before, 12);
        let (_, eval_after) = engine.eval(&m.fen_after, 12);

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
