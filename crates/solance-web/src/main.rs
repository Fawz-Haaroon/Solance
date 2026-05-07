use std::sync::Arc;

use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use solance_analysis::analyze_game;
use solance_engine::{Engine, Score, Stockfish};
use solance_parser::GameBuilder;
use pgn_reader::BufferedReader;

#[derive(Deserialize)]
struct AnalyzeRequest {
    pgn:   String,
    depth: Option<u32>,
    #[allow(dead_code)]
    engine: Option<String>,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    event:          String,
    white:          String,
    black:          String,
    result:         String,
    engine:         String,
    depth:          u32,
    white_accuracy: f32,
    black_accuracy: f32,
    turning_point:  Option<usize>,
    moves:          Vec<MoveResponse>,
}

#[derive(Serialize)]
struct MoveResponse {
    move_number:      usize,
    side:             String,
    san:              String,
    uci:              String,
    fen_before:       String,
    best_uci:         Option<String>,
    score_cp:         Option<i32>,
    loss_cp:          i32,
    win_percent_loss: f64,
    rank:             Option<usize>,
    class:            String,
    decided:          bool,
}

#[derive(Clone)]
struct AppState {
    engine: Arc<Mutex<Box<dyn Engine>>>,
}

#[tokio::main]
async fn main() {
    let engine: Box<dyn Engine> = Box::new(
        Stockfish::launch().expect("stockfish not found")
    );
    let state = AppState { engine: Arc::new(Mutex::new(engine)) };
    let app = Router::new()
        .route("/analyze", post(handle_analyze))
        .layer(CorsLayer::permissive())
        .with_state(state);
    println!("solance-web listening on 0.0.0.0:4242");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_analyze(
    State(state): State<AppState>,
    axum::extract::Json(body): axum::extract::Json<AnalyzeRequest>,
) -> impl IntoResponse {
    let depth = body.depth.unwrap_or(16).clamp(6, 24);

    let game = {
        let mut reader  = BufferedReader::new(body.pgn.as_bytes());
        let mut builder = GameBuilder::new();
        match reader.read_game(&mut builder) {
            Ok(Some(Ok(g)))  => g,
            Ok(Some(Err(e))) => return (StatusCode::UNPROCESSABLE_ENTITY, format!("pgn parse error: {e}")).into_response(),
            _                => return (StatusCode::UNPROCESSABLE_ENTITY, "no game found in pgn".to_owned()).into_response(),
        }
    };

    let mut engine = state.engine.lock().await;
    engine.reset();
    let summary = analyze_game(&game.moves, engine.as_mut(), depth);

    let moves = summary.moves.iter().enumerate().zip(game.moves.iter()).map(|((i, mv), annotated)| {
        MoveResponse {
            move_number:      i / 2 + 1,
            side:             if i % 2 == 0 { "white".into() } else { "black".into() },
            san:              mv.played_san.clone(),
            uci:              mv.played_uci.clone(),
            fen_before:       annotated.fen_before.clone(),
            best_uci:         mv.best_uci.clone(),
            score_cp:         match mv.score_before { Score::Cp(n) => Some(n), Score::Mate(_) => None },
            loss_cp:          mv.centipawn_loss,
            win_percent_loss: mv.win_percent_loss,
            rank:             mv.rank,
            class:            mv.class.to_string(),
            decided:          mv.decided,
        }
    }).collect();

    axum::Json(AnalyzeResponse {
        event:          game.meta.event.unwrap_or_else(|| "?".into()),
        white:          game.meta.white.unwrap_or_else(|| "?".into()),
        black:          game.meta.black.unwrap_or_else(|| "?".into()),
        result:         game.meta.result.unwrap_or_else(|| "*".into()),
        engine:         summary.engine_name,
        depth,
        white_accuracy: summary.white_accuracy,
        black_accuracy: summary.black_accuracy,
        turning_point:  summary.turning_point,
        moves,
    }).into_response()
}
