use solance_core::AnnotatedMove;
use solance_engine::{Evaluation, Score, Stockfish};

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
        let s = match self {
            Self::Best       => "best",
            Self::Excellent  => "excellent",
            Self::Good       => "good",
            Self::Inaccuracy => "inaccuracy",
            Self::Mistake    => "mistake",
            Self::Blunder    => "blunder",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone)]
pub struct MoveAnalysis {
    pub played_uci:    String,
    pub played_san:    String,
    pub best_uci:      Option<String>,
    pub score_before:  Score,
    pub score_after:   Score,
    pub centipawn_loss: i32,
    pub rank:          Option<usize>,
    pub class:         Classification,
}

#[derive(Debug, Clone)]
pub struct GameSummary {
    pub moves:          Vec<MoveAnalysis>,
    pub accuracy:       f32,
    pub turning_point:  Option<usize>,
}

pub fn analyze_game(
    moves: &[AnnotatedMove],
    engine: &mut Stockfish,
    depth: u32,
) -> GameSummary {
    let mut analyses: Vec<MoveAnalysis> = Vec::with_capacity(moves.len());

    for mv in moves {
        // Evaluate before the move is applied — candidates reflect the side to move.
        let pre = engine.evaluate(depth);

        let score_before = match pre.best() {
            Some(c) => c.score,
            None    => Score::Cp(0),
        };

        let rank = pre.candidates.iter().find(|c| c.mv == mv.uci).map(|c| c.rank);

        engine.apply_move(&mv.uci).unwrap_or_else(|e| {
            panic!("move {} from game record rejected by engine: {e}", mv.uci);
        });

        let post         = engine.evaluate(depth);
        // score_after is from the opponent's perspective — negate to stay white-relative
        let score_after  = match post.best() {
            Some(c) => negate(c.score),
            None    => Score::Cp(0),
        };

        let centipawn_loss = cp_loss(score_before, score_after);
        let class          = classify(centipawn_loss, score_before, score_after);
        let best_uci       = pre.best().map(|c| c.mv.clone());

        analyses.push(MoveAnalysis {
            played_uci: mv.uci.clone(),
            played_san: mv.san.clone(),
            best_uci,
            score_before,
            score_after,
            centipawn_loss,
            rank,
            class,
        });
    }

    let accuracy       = compute_accuracy(&analyses);
    let turning_point  = find_turning_point(&analyses);

    GameSummary { moves: analyses, accuracy, turning_point }
}

fn negate(s: Score) -> Score {
    match s {
        Score::Cp(n)   => Score::Cp(-n),
        Score::Mate(n) => Score::Mate(-n),
    }
}

fn cp_loss(before: Score, after: Score) -> i32 {
    match (before, after) {
        (Score::Cp(b), Score::Cp(a)) => (b - a).max(0),
        // Throwing away a forced mate is effectively infinite loss — cap at 1000
        // so downstream math doesn't explode.
        (Score::Mate(_), Score::Cp(_)) => 1000,
        _ => 0,
    }
}

fn classify(loss: i32, before: Score, after: Score) -> Classification {
    // If a forced mate existed before and is gone after, that's a blunder
    // regardless of the centipawn floor.
    if matches!(before, Score::Mate(n) if n > 0) {
        if !matches!(after, Score::Mate(n) if n > 0) {
            return Classification::Blunder;
        }
    }

    match loss {
        0..=10  => Classification::Best,
        11..=30 => Classification::Excellent,
        31..=80 => Classification::Good,
        81..=150 => Classification::Inaccuracy,
        151..=300 => Classification::Mistake,
        _ => Classification::Blunder,
    }
}

fn compute_accuracy(analyses: &[MoveAnalysis]) -> f32 {
    if analyses.is_empty() {
        return 0.0;
    }
    // Win% accuracy model used by Lichess — maps centipawn loss to accuracy.
    // Each move contributes independently; average over the game.
    let total: f32 = analyses.iter().map(|a| {
        let loss = a.centipawn_loss.min(1000) as f32;
        (-(loss / 100.0)).exp()
    }).sum();

    (total / analyses.len() as f32) * 100.0
}

fn find_turning_point(analyses: &[MoveAnalysis]) -> Option<usize> {
    // First move where the score flips sign and the loss is >= 150cp.
    // Zero-indexed into the move list.
    analyses.windows(2).enumerate().find_map(|(i, w)| {
        let sign_before = cp_of(w[0].score_before);
        let sign_after  = cp_of(w[1].score_after);
        if sign_before * sign_after < 0 && w[0].centipawn_loss >= 150 {
            Some(i)
        } else {
            None
        }
    })
}

fn cp_of(s: Score) -> i32 {
    match s {
        Score::Cp(n)   => n,
        Score::Mate(n) => if n > 0 { 30000 } else { -30000 },
    }
}
