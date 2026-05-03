use solance_core::AnnotatedMove;
use solance_engine::{Engine, Score};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Best       => "best",
            Self::Excellent  => "excellent",
            Self::Good       => "good",
            Self::Inaccuracy => "inaccuracy",
            Self::Mistake    => "mistake",
            Self::Blunder    => "blunder",
        })
    }
}

#[derive(Debug, Clone)]
pub struct MoveAnalysis {
    pub played_uci:       String,
    pub played_san:       String,
    pub best_uci:         Option<String>,
    pub eval_before:      Score,
    pub eval_after:       Score,
    pub centipawn_loss:   i32,
    pub win_percent_loss: f64,
    pub rank:             Option<usize>,
    pub class:            Classification,
    pub decided:          bool,
}

#[derive(Debug, Clone)]
pub struct GameSummary {
    pub engine_name:    String,
    pub moves:          Vec<MoveAnalysis>,
    pub white_accuracy: f32,
    pub black_accuracy: f32,
    pub turning_point:  Option<usize>,
}

pub fn analyze_game(
    moves: &[AnnotatedMove],
    engine: &mut dyn Engine,
    depth: u32,
) -> GameSummary {
    let engine_name = engine.name().to_owned();

    let mut raw_evals = Vec::with_capacity(moves.len() + 1);
    for mv in moves {
        raw_evals.push(engine.evaluate(depth));
        engine.apply_move(&mv.uci).unwrap_or_else(|e| {
            panic!("move {} rejected by engine: {e}", mv.uci);
        });
    }
    raw_evals.push(engine.evaluate(depth));

    // parse_info_line in solance-engine normalizes scores to white-relative already.
    let evals = raw_evals;

    let mut analyses: Vec<MoveAnalysis> = Vec::with_capacity(moves.len());

    for (i, mv) in moves.iter().enumerate() {
        let pre  = &evals[i];
        let post = &evals[i + 1];

        let eval_before = pre.best().map(|c| c.score).unwrap_or(Score::Cp(0));
        let eval_after  = post.best().map(|c| c.score).unwrap_or(Score::Cp(0));
        let best_uci    = pre.best().map(|c| c.mv.clone());
        let rank        = pre.candidates.iter().find(|c| c.mv == mv.uci).map(|c| c.rank);

        let white_to_move    = i % 2 == 0;
        let decided          = (white_to_move  && matches!(eval_before, Score::Mate(n) if n > 0))
                            || (!white_to_move && matches!(eval_before, Score::Mate(n) if n < 0));
        let centipawn_loss   = cp_loss(eval_before, eval_after, white_to_move);
        let win_percent_loss = if decided { 0.0 } else { wpl(eval_before, eval_after) };
        let class            = classify(win_percent_loss, eval_before, eval_after, decided, white_to_move);

        analyses.push(MoveAnalysis {
            played_uci: mv.uci.clone(),
            played_san: mv.san.clone(),
            best_uci,
            eval_before,
            eval_after,
            centipawn_loss,
            win_percent_loss,
            rank,
            class,
            decided,
        });
    }

    let white_accuracy = lichess_accuracy(
        analyses.iter().enumerate()
            .filter(|(i, a)| i % 2 == 0 && !a.decided)
            .map(|(_, a)| a)
    );
    let black_accuracy = lichess_accuracy(
        analyses.iter().enumerate()
            .filter(|(i, a)| i % 2 != 0 && !a.decided)
            .map(|(_, a)| a)
    );
    let turning_point = find_turning_point(&analyses);

    GameSummary { engine_name, moves: analyses, white_accuracy, black_accuracy, turning_point }
}

fn win_prob(s: Score) -> f64 {
    let cp: f64 = match s {
        Score::Cp(n)   => n as f64,
        Score::Mate(n) => if n > 0 { 10_000.0 } else { -10_000.0 },
    };
    1.0 / (1.0 + (-0.005756 * cp).exp())
}

fn wpl(before: Score, after: Score) -> f64 {
    (win_prob(before) - win_prob(after)).max(0.0)
}

fn cp_loss(before: Score, after: Score, white_to_move: bool) -> i32 {
    let sign = if white_to_move { 1i32 } else { -1i32 };
    let b = match before {
        Score::Cp(n)   => n * sign,
        Score::Mate(n) => if (n > 0) == white_to_move { 5000 } else { -5000 },
    };
    let a = match after {
        Score::Cp(n)   => n * sign,
        Score::Mate(n) => if (n > 0) == white_to_move { 5000 } else { -5000 },
    };
    (b - a).max(0).min(2000)
}

fn classify(wpl: f64, before: Score, after: Score, decided: bool, white_to_move: bool) -> Classification {
    if decided { return Classification::Best; }

    let had_mate = if white_to_move {
        matches!(before, Score::Mate(n) if n > 0)
    } else {
        matches!(before, Score::Mate(n) if n < 0)
    };
    let still_mate = if white_to_move {
        matches!(after, Score::Mate(n) if n > 0)
    } else {
        matches!(after, Score::Mate(n) if n < 0)
    };
    if had_mate && !still_mate { return Classification::Blunder; }

    if wpl < 0.02  { return Classification::Best; }
    if wpl < 0.05  { return Classification::Excellent; }
    if wpl < 0.10  { return Classification::Good; }
    if wpl < 0.175 { return Classification::Inaccuracy; }
    if wpl < 0.30  { return Classification::Mistake; }
    Classification::Blunder
}

fn lichess_accuracy<'a>(moves: impl Iterator<Item = &'a MoveAnalysis>) -> f32 {
    let mut count = 0usize;
    let total_cpl: i64 = moves.map(|a| { count += 1; a.centipawn_loss as i64 }).sum();
    if count == 0 { return 0.0; }
    let acpl = total_cpl as f64 / count as f64;
    let accuracy = 103.1668 * (-0.04354 * acpl).exp() - 3.1669;
    accuracy.clamp(0.0, 100.0) as f32
}

fn find_turning_point(analyses: &[MoveAnalysis]) -> Option<usize> {
    analyses.windows(3).enumerate().find_map(|(i, w)| {
        let before = win_prob(w[0].eval_before);
        let after  = win_prob(w[1].eval_after);
        if (before - 0.5) * (after - 0.5) < 0.0 && w[1].win_percent_loss > 0.15 {
            Some(i + 1)
        } else {
            None
        }
    })
}
