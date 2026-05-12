use std::sync::Arc;

use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use solance_analysis::{analyze_game, MoveAnalysis};
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
    games: Vec<GameResponse>,
}

#[derive(Serialize)]
struct GameResponse {
    event:          String,
    white:          String,
    black:          String,
    result:         String,
    eco:            Option<String>,
    opening:        Option<String>,
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

    // Parse all games from the PGN, not just the first.
    let mut games = Vec::new();
    let mut reader = BufferedReader::new(body.pgn.as_bytes());
    loop {
        let mut builder = GameBuilder::new();
        match reader.read_game(&mut builder) {
            Ok(Some(Ok(g)))  => games.push(g),
            Ok(Some(Err(e))) => return (StatusCode::UNPROCESSABLE_ENTITY, format!("pgn error: {e}")).into_response(),
            Ok(None)         => break,
            Err(e)           => return (StatusCode::UNPROCESSABLE_ENTITY, format!("io error: {e}")).into_response(),
        }
    }

    if games.is_empty() {
        return (StatusCode::UNPROCESSABLE_ENTITY, "no games found in pgn".to_owned()).into_response();
    }

    let mut engine = state.engine.lock().await;
    let mut game_responses = Vec::with_capacity(games.len());

    for game in &games {
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

        game_responses.push(GameResponse {
            event:          game.meta.event.clone().unwrap_or_else(|| "?".into()),
            white:          game.meta.white.clone().unwrap_or_else(|| "?".into()),
            black:          game.meta.black.clone().unwrap_or_else(|| "?".into()),
            result:         game.meta.result.clone().unwrap_or_else(|| "*".into()),
            eco:            game.meta.eco.clone(),
            opening:        game.meta.opening.clone(),
            engine:         summary.engine_name,
            depth,
            white_accuracy: summary.white_accuracy,
            black_accuracy: summary.black_accuracy,
            turning_point:  summary.turning_point,
            moves,
        });
    }

    axum::Json(AnalyzeResponse { games: game_responses }).into_response()
}
