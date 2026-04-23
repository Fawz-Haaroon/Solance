use std::env;
use std::fs::File;
use std::io::BufReader;
use std::process;

use pgn_reader::BufferedReader;

use solance_analysis::analyze_game;
use solance_engine::{Score, Stockfish};
use solance_parser::GameBuilder;

fn main() {
    let pgn_path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: solance <game.pgn>");
        process::exit(1);
    });

    let file = File::open(&pgn_path).unwrap_or_else(|e| {
        eprintln!("cannot open {pgn_path}: {e}");
        process::exit(1);
    });

    let mut reader  = BufferedReader::new(BufReader::new(file));
    let mut builder = GameBuilder::new();

    let game = match reader.read_game(&mut builder) {
        Ok(Some(Ok(g)))  => g,
        Ok(Some(Err(e))) => {
            eprintln!("pgn parse error: {e}");
            process::exit(1);
        }
        Ok(None) => {
            eprintln!("no game found in {pgn_path}");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("io error reading {pgn_path}: {e}");
            process::exit(1);
        }
    };

    let white  = game.meta.white.as_deref().unwrap_or("?");
    let black  = game.meta.black.as_deref().unwrap_or("?");
    let event  = game.meta.event.as_deref().unwrap_or("?");
    let result = game.meta.result.as_deref().unwrap_or("*");

    println!("{event}: {white} vs {black}  [{result}]");
    println!("{}", "─".repeat(60));

    let mut engine = Stockfish::launch().unwrap_or_else(|e| {
        eprintln!("engine error: {e}");
        process::exit(1);
    });

    let summary = analyze_game(&game.moves, &mut engine, 16);

    for (i, mv) in summary.moves.iter().enumerate() {
        let move_num   = i / 2 + 1;
        let side       = if i % 2 == 0 { "W" } else { "B" };
        let best_label = mv.best_uci.as_deref().unwrap_or("?");
        let rank_label = mv.rank.map(|r| format!("#{r}")).unwrap_or_else(|| "—".to_string());

        println!(
            "{move_num:>3}{side}  {:<6}  {:>4}cp loss  {:<10}  rank {rank_label}  best: {best_label}",
            mv.played_san,
            mv.centipawn_loss,
            mv.class,
        );
    }

    println!("{}", "─".repeat(60));
    println!("accuracy: {:.1}%", summary.accuracy);

    if let Some(tp) = summary.turning_point {
        let mv  = &summary.moves[tp];
        let num = tp / 2 + 1;
        let side = if tp % 2 == 0 { "W" } else { "B" };
        println!("turning point: move {num}{side} — {} ({}cp loss)", mv.played_san, mv.centipawn_loss);
    }

    print_score_graph(&summary.moves);
}

fn print_score_graph(moves: &[solance_analysis::MoveAnalysis]) {
    println!("\nscore (white perspective, capped ±5):");
    for (i, mv) in moves.iter().enumerate() {
        let cp = match mv.score_before {
            Score::Cp(n)     => n.clamp(-500, 500),
            Score::Mate(n)   => if n > 0 { 500 } else { -500 },
        };
        let bar_len = ((cp + 500) / 50) as usize;  // 0..=20
        let side    = if i % 2 == 0 { 'W' } else { 'B' };
        println!(
            "{:>3}{side} {:>+5}  {}{}",
            i / 2 + 1,
            cp,
            "█".repeat(bar_len),
            "░".repeat(20usize.saturating_sub(bar_len)),
        );
    }
}
