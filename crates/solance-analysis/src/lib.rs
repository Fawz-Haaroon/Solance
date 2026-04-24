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
    pub played_uci:     String,
    pub played_san:     String,
    pub best_uci:       Option<String>,
    pub score_before:   Score,
    pub score_after:    Score,
    pub centipawn_loss: i32,
    pub win_percent_loss: f64,
    pub rank:           Option<usize>,
    pub class:          Classification,
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
    let mut analyses: Vec<MoveAnalysis> = Vec::with_capacity(moves.len());

    for mv in moves {
        let pre          = engine.evaluate(depth);
        let score_before = pre.best().map(|c| c.score).unwrap_or(Score::Cp(0));
        let best_uci     = pre.best().map(|c| c.mv.clone());
        let rank         = pre.candidates.iter().find(|c| c.mv == mv.uci).map(|c| c.rank);

        let played_score = pre.candidates.iter().find(|c| c.mv == mv.uci).map(|c| c.score);

        engine.apply_move(&mv.uci).unwrap_or_else(|e| {
            panic!("move {} from game record rejected by engine: {e}", mv.uci);
        });

        let score_after = match played_score {
            Some(s) => s,
            None => {
                let post = engine.evaluate(depth);
                match post.best() {
                    Some(c) => negate(c.score),
                    None    => Score::Cp(0),
                }
            }
        };

        let centipawn_loss  = cp_loss(score_before, score_after);
        let win_percent_loss = win_percent_loss(score_before, score_after);
        let class            = classify(centipawn_loss, score_before, score_after);

        analyses.push(MoveAnalysis {
            played_uci: mv.uci.clone(),
            played_san: mv.san.clone(),
            best_uci,
            score_before,
            score_after,
            centipawn_loss,
            win_percent_loss,
            rank,
            class,
        });
    }

    let white_accuracy = lichess_accuracy(analyses.iter().enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, a)| a));
    let black_accuracy = lichess_accuracy(analyses.iter().enumerate().filter(|(i, _)| i % 2 != 0).map(|(_, a)| a));
    let turning_point  = find_turning_point(&analyses);

    GameSummary { engine_name, moves: analyses, white_accuracy, black_accuracy, turning_point }
}

fn negate(s: Score) -> Score {
    match s {
        Score::Cp(n)   => Score::Cp(-n),
        Score::Mate(n) => Score::Mate(-n),
    }
}

fn cp_loss(before: Score, after: Score) -> i32 {
    match (before, after) {
        (Score::Cp(b), Score::Cp(a))            => (b - a).max(0),
        (Score::Mate(n), Score::Cp(_)) if n > 0 => 1000,
        _                                        => 0,
    }
}

// Win probability from centipawns — Lichess model.
fn win_percent(score: Score) -> f64 {
    let cp = match score {
        Score::Cp(n)   => n as f64,
        Score::Mate(n) => if n > 0 { 10000.0 } else { -10000.0 },
    };
    1.0 / (1.0 + (-0.00368208 * cp).exp())
}

fn win_percent_loss(before: Score, after: Score) -> f64 {
    (win_percent(before) - win_percent(after)).max(0.0)
}

// Lichess accuracy formula: maps average win% loss per move to an accuracy percentage.
// Source: https://lichess.org/page/accuracy
fn lichess_accuracy<'a>(moves: impl Iterator<Item = &'a MoveAnalysis>) -> f32 {
    let mut count = 0usize;
    let total_wpl: f64 = moves.map(|a| { count += 1; a.win_percent_loss }).sum();
    if count == 0 {
        return 0.0;
    }
    let avg_wpl = total_wpl / count as f64;
    let accuracy = 103.1668 * (-0.04354 * avg_wpl * 100.0).exp() - 3.1669;
    accuracy.clamp(0.0, 100.0) as f32
}

fn classify(loss: i32, before: Score, after: Score) -> Classification {
    if matches!(before, Score::Mate(n) if n > 0) {
        if !matches!(after, Score::Mate(n) if n > 0) {
            return Classification::Blunder;
        }
    }
    match loss {
        0..=10    => Classification::Best,
        11..=30   => Classification::Excellent,
        31..=80   => Classification::Good,
        81..=150  => Classification::Inaccuracy,
        151..=300 => Classification::Mistake,
        _         => Classification::Blunder,
    }
}

fn find_turning_point(analyses: &[MoveAnalysis]) -> Option<usize> {
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
