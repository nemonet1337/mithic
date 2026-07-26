use axum::{Json, extract::State, routing::get};
use mithic_core::Result;

use crate::state::AppState;

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/home", get(home_timeline))
        .route("/local", get(local_timeline))
        .route("/global", get(global_timeline))
        .route("/hashtag/{tag}", get(hashtag_timeline))
        .with_state(state)
}

pub async fn home_timeline(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn local_timeline(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn global_timeline(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn hashtag_timeline(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}