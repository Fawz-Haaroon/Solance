use std::env;
use std::fs::File;
use std::io::BufReader;

use pgn_reader::{BufferedReader, Visitor};

use solance_engine::{Engine, Stockfish};
use solance_parser::GameBuilder;
use solance_analysis::{analyze, Classification};

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
        let played = &m.uci;

        let candidates = engine.eval_current_multi(12);

        engine.apply_move(played);

        let played_score = engine.eval_current_single(12);

        let analysis = analyze(played, &candidates, played_score);

        let class_str = match analysis.class {
            Classification::Best => "best",
            Classification::Excellent => "excellent",
            Classification::Good => "good",
            Classification::Inaccuracy => "inaccuracy",
            Classification::Mistake => "mistake",
            Classification::Blunder => "blunder",
            Classification::Unknown => "unknown",
        };

        println!(
            "{:>3}. {:<8} | rank: {:?} | loss: {:?} | {:<10} | best: {}",
            i + 1,
            m.mv,
            analysis.rank,
            analysis.loss,
            class_str,
            analysis.best.mv
        );
    }
}
