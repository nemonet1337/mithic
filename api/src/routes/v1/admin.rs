use axum::{Json, extract::State, http::StatusCode, routing::{get, delete, post, patch}};
use mithic_core::{AppError, Result};

use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct StatsResponse {
    pub users_count: u64,
    pub notes_count: u64,
    pub instances_count: u64,
    pub online_users: u64,
}

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/stats", get(stats))
        .with_state(state)
}

pub async fn stats(
    State(state): State<AppState>,
) -> Result<Json<StatsResponse>> {
    let _ = state;
    Ok(Json(StatsResponse {
        users_count: 0,
        notes_count: 0,
        instances_count: 0,
        online_users: 0,
    }))
}