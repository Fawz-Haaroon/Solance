use std::fs::File;
use std::io::BufReader;
use std::process;

use pgn_reader::BufferedReader;

use solance_analysis::{analyze_game, MoveAnalysis};
use solance_engine::{Engine, Score, Stockfish};
use solance_parser::GameBuilder;

struct Args {
    pgn_path:    String,
    depth:       u32,
    engine_bin:  String,
}

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();

    if raw.is_empty() || raw.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("usage: solance <game.pgn> [--depth N] [--engine PATH]");
        eprintln!("  --depth N      search depth (default: 16)");
        eprintln!("  --engine PATH  path to UCI engine binary (default: stockfish)");
        process::exit(0);
    }

    let mut pgn_path   = String::new();
    let mut depth      = 16u32;
    let mut engine_bin = "stockfish".to_owned();

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--depth" => {
                depth = raw.get(i + 1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| {
                        eprintln!("--depth requires a number");
                        process::exit(1);
                    });
                i += 2;
            }
            "--engine" => {
                engine_bin = raw.get(i + 1).cloned().unwrap_or_else(|| {
                    eprintln!("--engine requires a path");
                    process::exit(1);
                });
                i += 2;
            }
            path => {
                pgn_path = path.to_owned();
                i += 1;
            }
        }
    }

    if pgn_path.is_empty() {
        eprintln!("error: no pgn file specified");
        process::exit(1);
    }

    Args { pgn_path, depth, engine_bin }
}

fn main() {
    let args = parse_args();

    let file = File::open(&args.pgn_path).unwrap_or_else(|e| {
        eprintln!("cannot open {}: {e}", args.pgn_path);
        process::exit(1);
    });

    let mut reader  = BufferedReader::new(BufReader::new(file));
    let mut builder = GameBuilder::new();

    let game = match reader.read_game(&mut builder) {
        Ok(Some(Ok(g)))  => g,
        Ok(Some(Err(e))) => { eprintln!("pgn parse error: {e}"); process::exit(1); }
        Ok(None)         => { eprintln!("no game found in {}", args.pgn_path); process::exit(1); }
        Err(e)           => { eprintln!("io error reading {}: {e}", args.pgn_path); process::exit(1); }
    };

    let mut engine: Box<dyn Engine> = Box::new(
        Stockfish::launch_from(&args.engine_bin).unwrap_or_else(|e| {
            eprintln!("engine error: {e}");
            process::exit(1);
        })
    );

    let white  = game.meta.white.as_deref().unwrap_or("?");
    let black  = game.meta.black.as_deref().unwrap_or("?");
    let event  = game.meta.event.as_deref().unwrap_or("?");
    let result = game.meta.result.as_deref().unwrap_or("*");

    println!("{event}: {white} vs {black}  [{result}]");
    println!("engine: {}  depth: {}", engine.name(), args.depth);
    println!("{}", "─".repeat(64));

    let summary = analyze_game(&game.moves, engine.as_mut(), args.depth);

    for (i, mv) in summary.moves.iter().enumerate() {
        let move_num   = i / 2 + 1;
        let side       = if i % 2 == 0 { 'W' } else { 'B' };
        let rank_label = mv.rank.map(|r| format!("#{r}")).unwrap_or_else(|| " —".to_string());
        let best_label = mv.best_uci.as_deref().unwrap_or("?");

        println!(
            "{move_num:>3}{side}  {:<7} {:>4}cp  {:<10}  rank {:<3}  best: {best_label}",
            mv.played_san,
            mv.centipawn_loss,
            mv.class,
            rank_label,
        );
    }

    println!("{}", "─".repeat(64));
    println!(
        "accuracy — white: {:.1}%  black: {:.1}%",
        summary.white_accuracy,
        summary.black_accuracy,
    );

    if let Some(tp) = summary.turning_point {
        let mv   = &summary.moves[tp];
        let num  = tp / 2 + 1;
        let side = if tp % 2 == 0 { 'W' } else { 'B' };
        println!("turning point: move {num}{side} — {} ({}cp loss)", mv.played_san, mv.centipawn_loss);
    }

    print_score_graph(&summary.moves);
}

fn print_score_graph(moves: &[MoveAnalysis]) {
    println!("\nscore (white perspective, capped ±500cp):");
    for (i, mv) in moves.iter().enumerate() {
        let cp = match mv.score_before {
            Score::Cp(n)   => n.clamp(-500, 500),
            Score::Mate(n) => if n > 0 { 500 } else { -500 },
        };
        // 0..=20 bar, midpoint at 10 = equal position
        let filled  = ((cp + 500) / 50) as usize;
        let side    = if i % 2 == 0 { 'W' } else { 'B' };
        println!(
            "{:>3}{side} {:>+5}cp  {}{}",
            i / 2 + 1,
            cp,
            "█".repeat(filled),
            "░".repeat(20usize.saturating_sub(filled)),
        );
    }
}

fn debug_run(pgn_path: &str) {
    use std::fs::File;
    use std::io::BufReader;
    use pgn_reader::BufferedReader;
    use solance_parser::GameBuilder;
    use solance_engine::Stockfish;
    use solance_analysis::debug_first_moves;

    let file    = File::open(pgn_path).unwrap();
    let mut rdr = BufferedReader::new(BufReader::new(file));
    let game    = rdr.read_game(&mut GameBuilder::new()).unwrap().unwrap().unwrap();
    let mut eng = Stockfish::launch().unwrap();
    debug_first_moves(&game.moves, &mut eng, 16);
}
