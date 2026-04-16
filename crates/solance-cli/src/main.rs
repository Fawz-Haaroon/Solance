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
        // ---- BEFORE POSITION ----
        let eval_before = engine.evaluate(12);

        // best from BEFORE
        let best_move = eval_before.best.clone().unwrap_or_default();
        let best_score = eval_before.score;

        // evaluate PLAYED MOVE from SAME position:
        // apply → eval → revert by reinitializing and replaying moves up to i, excluding current
        // (cheap enough for now; we’ll optimize later)

        // snapshot: rebuild engine to the same pre-move state
        let mut probe: Box<dyn Engine> = Box::new(Stockfish::new());
        probe.start_game();
        for j in 0..i {
            probe.apply_move(&game.moves[j].uci);
        }

        // apply current move on probe and evaluate
        probe.apply_move(&m.uci);
        let played_after = probe.evaluate(12).score;

        // convert played_after (after move) to comparable delta:
        // loss = best_before - played_after (but now both from SAME ROOT via probe)
        let loss = match (best_score, played_after) {
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

        // advance main engine AFTER computing loss
        engine.apply_move(&m.uci);

        println!(
            "{:>3}. {:<8} | loss: {:?} | {:<10} | best: {}",
            i + 1,
            m.mv,
            loss,
            class,
            best_move
        );
    }
}
