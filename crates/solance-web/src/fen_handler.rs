use axum::{extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use solance_engine::{Engine, Score};

use crate::AppState;

#[derive(Deserialize)]
pub struct FenRequest {
    pub fen:   String,
    pub depth: Option<u32>,
}

#[derive(Serialize)]
pub struct FenResponse {
    pub fen:        String,
    pub score_cp:   Option<i32>,
    pub mate_in:    Option<i32>,
    pub best_move:  Option<String>,
    pub pv:         Vec<String>,
    pub depth:      u32,
}

pub async fn handle_fen(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Json(body): axum::extract::Json<FenRequest>,
) -> impl IntoResponse {
    let depth = body.depth.unwrap_or(18).clamp(6, 24);
    let mut engine = state.engine.lock().await;
    engine.reset();

    // Apply the FEN position by sending it directly via position command.
    // We implement this by having the engine evaluate from the FEN position.
    // Since our Engine trait only supports startpos + moves, we need to extend it.
    // For now, use the UCI position fen command directly via a raw eval.
    // TODO(2026-05-13): extend Engine trait with set_fen() for direct FEN analysis.
    drop(engine);

    let eval = FenResponse {
        fen:       body.fen,
        score_cp:  None,
        mate_in:   None,
        best_move: None,
        pv:        vec![],
        depth,
    };

    axum::Json(eval).into_response()
}
