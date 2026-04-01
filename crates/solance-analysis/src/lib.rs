use solance_engine::Candidate;

#[derive(Debug, Clone)]
pub struct MoveEval {
    pub mv: String,
    pub score: Option<i32>,
}

#[derive(Debug, Clone)]
pub enum Classification {
    Best,
    Excellent,
    Good,
    Inaccuracy,
    Mistake,
    Blunder,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Analysis {
    pub best: MoveEval,
    pub played: MoveEval,
    pub rank: Option<usize>,
    pub loss: Option<i32>,
    pub class: Classification,
}

pub fn analyze(
    played: &str,
    candidates: &[Candidate],
    played_score: Option<i32>,
) -> Analysis {
    let mut best = MoveEval {
        mv: String::new(),
        score: None,
    };

    let mut rank = None;

    for c in candidates {
        let mv = c.mv.trim();

        if c.rank == 1 {
            best = MoveEval {
                mv: mv.to_string(),
                score: c.score,
            };
        }

        if played == mv {
            rank = Some(c.rank);
        }
    }

    let played_eval = MoveEval {
        mv: played.to_string(),
        score: played_score,
    };

    let loss = match (best.score, played_eval.score) {
        (Some(b), Some(p)) => Some(b - p),
        _ => None,
    };

    let class = match loss {
        Some(l) if l <= 10   => Classification::Best,
        Some(l) if l <= 30   => Classification::Excellent,
        Some(l) if l <= 80   => Classification::Good,
        Some(l) if l <= 150  => Classification::Inaccuracy,
        Some(l) if l <= 300  => Classification::Mistake,
        Some(_)              => Classification::Blunder,
        None                 => Classification::Unknown,
    };

    Analysis {
        best,
        played: played_eval,
        rank,
        loss,
        class,
    }
}
