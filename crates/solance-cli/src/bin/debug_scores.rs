use std::fs::File;
use std::io::BufReader;
use pgn_reader::BufferedReader;
use solance_engine::{Engine, Stockfish};
use solance_parser::GameBuilder;

fn main() {
    let file    = File::open("/tmp/test.pgn").unwrap();
    let mut rdr = BufferedReader::new(BufReader::new(file));
    let game    = rdr.read_game(&mut GameBuilder::new()).unwrap().unwrap().unwrap();
    let mut eng = Stockfish::launch().unwrap();

    for (i, mv) in game.moves.iter().take(10).enumerate() {
        let pre       = eng.evaluate(16);
        let pre_score = pre.best().map(|c| c.score);
        let pre_best  = pre.best().map(|c| c.mv.clone());

        eng.apply_move(&mv.uci).unwrap();

        let post       = eng.evaluate(16);
        let post_score = post.best().map(|c| c.score);

        eprintln!("move {:>2} {:>6}  pre={:?}  best={:?}  post_raw={:?}",
            i + 1, mv.san, pre_score, pre_best, post_score);
    }
}
