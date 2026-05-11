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
    pub score_before:     Score,
    pub score_after:      Score,
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

    // Collect N+1 evaluations. Each is white-relative (positive = white winning).
    let mut evals = Vec::with_capacity(moves.len() + 1);
    for mv in moves {
        evals.push(engine.evaluate(depth));
        engine.apply_move(&mv.uci).unwrap_or_else(|e| {
            panic!("move {} rejected by engine: {e}", mv.uci);
        });
    }
    evals.push(engine.evaluate(depth));

    let mut analyses: Vec<MoveAnalysis> = Vec::with_capacity(moves.len());

    for (i, mv) in moves.iter().enumerate() {
        let pre  = &evals[i];
        let post = &evals[i + 1];

        let score_before = pre.best().map(|c| c.score).unwrap_or(Score::Cp(0));
        let score_after  = post.best().map(|c| c.score).unwrap_or(Score::Cp(0));
        let best_uci     = pre.best().map(|c| c.mv.clone());
        let rank         = pre.candidates.iter().find(|c| c.mv == mv.uci).map(|c| c.rank);

        let white_to_move = i % 2 == 0;

        // Win probability before and after, both white-relative.
        let wp_before = win_prob(score_before);
        let wp_after  = win_prob(score_after);

        // For white moves: loss = drop in white's win probability.
        // For black moves: loss = rise in white's win probability (black wants it to drop).
        let wpl = if white_to_move {
            (wp_before - wp_after).max(0.0)
        } else {
            (wp_after - wp_before).max(0.0)
        };

        // A position is decided when one side has forced mate — exclude from accuracy.
        let decided = matches!(score_before, Score::Mate(_));

        let centipawn_loss = cp_loss(score_before, score_after, white_to_move);
        let class          = classify(wpl, score_before, score_after, decided, white_to_move);

        analyses.push(MoveAnalysis {
            played_uci: mv.uci.clone(),
            played_san: mv.san.clone(),
            best_uci,
            score_before,
            score_after,
            centipawn_loss,
            win_percent_loss: wpl,
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

// Lichess sigmoid — maps centipawns to win probability [0, 1], white-relative.
fn win_prob(s: Score) -> f64 {
    let cp: f64 = match s {
        Score::Cp(n)   => n as f64,
        Score::Mate(n) => if n > 0 { 10_000.0 } else { -10_000.0 },
    };
    1.0 / (1.0 + (-0.00368208 * cp).exp())
}

fn cp_loss(before: Score, after: Score, white_to_move: bool) -> i32 {
    let sign: i32 = if white_to_move { 1 } else { -1 };
    let b = match before {
        Score::Cp(n)   => n * sign,
        Score::Mate(n) => if n > 0 { 3000 * sign } else { -3000 * sign },
    };
    let a = match after {
        Score::Cp(n)   => n * sign,
        Score::Mate(n) => if n > 0 { 3000 * sign } else { -3000 * sign },
    };
    (b - a).max(0).min(1000)
}

fn classify(wpl: f64, before: Score, after: Score, decided: bool, white_to_move: bool) -> Classification {
    if decided { return Classification::Best; }

    // Throwing away a forced mate = blunder regardless of wpl magnitude.
    let had_mate = if white_to_move {
        matches!(before, Score::Mate(n) if n > 0)
    } else {
        matches!(before, Score::Mate(n) if n < 0)
    };
    let kept_mate = if white_to_move {
        matches!(after, Score::Mate(n) if n > 0)
    } else {
        matches!(after, Score::Mate(n) if n < 0)
    };
    if had_mate && !kept_mate { return Classification::Blunder; }

    // Lichess win-percent loss thresholds.
    if wpl < 0.02  { return Classification::Best; }
    if wpl < 0.05  { return Classification::Excellent; }
    if wpl < 0.10  { return Classification::Good; }
    if wpl < 0.175 { return Classification::Inaccuracy; }
    if wpl < 0.30  { return Classification::Mistake; }
    Classification::Blunder
}

// Lichess accuracy formula — maps average win% loss to accuracy percentage.
fn lichess_accuracy<'a>(moves: impl Iterator<Item = &'a MoveAnalysis>) -> f32 {
    let mut count      = 0usize;
    let total_wpl: f64 = moves.map(|a| { count += 1; a.win_percent_loss }).sum();
    if count == 0 { return 0.0; }
    let avg_wpl  = total_wpl / count as f64;
    let accuracy = 103.1668 * (-0.04354 * avg_wpl * 100.0).exp() - 3.1669;
    accuracy.clamp(0.0, 100.0) as f32
}

fn find_turning_point(analyses: &[MoveAnalysis]) -> Option<usize> {
    analyses.windows(2).enumerate().find_map(|(i, w)| {
        let wp0 = win_prob(w[0].score_before);
        let wp1 = win_prob(w[1].score_after);
        // Score crossed 0.5 (equal position) and the loss was significant.
        if (wp0 - 0.5) * (wp1 - 0.5) < 0.0 && w[0].win_percent_loss > 0.15 {
            Some(i)
        } else {
            None
        }
    })
}
