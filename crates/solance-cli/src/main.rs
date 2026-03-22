use std::{env, fs::File, io::BufReader};

use pgn_reader::{BufferedReader, Visitor};

use solance_engine::Stockfish;
use solance_parser::GameBuilder;

fn normalize_move(m: &str) -> String {
    let s = m.replace('-', "").to_lowercase();

    match s.len() {
        4 => s,
        5 => s[1..].into(),
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
        let candidates = engine.eval_multi(&m.fen_before, 12);

        let played = normalize_move(&m.move_played.to_string());

        let mut played_rank = None;
        let mut best = String::new();

        for c in &candidates {
            let mv = c.mv.trim();

            if c.rank == 1 {
                best = mv.to_string();
            }

            if played == mv {
                played_rank = Some(c.rank);
            }
        }

        let class = match played_rank {
            Some(1) => "best",
            Some(2) => "excellent",
            Some(3) => "good",
            Some(_) => "inaccuracy",
            None => "mistake",
        };

        println!(
            "{:>3}. {:<8} | rank: {:?} | {:<10} | best: {}",
            i + 1,
            m.move_played,
            played_rank,
            class,
            best
        );
    }
}
