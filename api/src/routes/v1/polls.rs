use axum::{Json, extract::State, http::StatusCode, routing::post};
use mithic_core::Result;

use crate::state::AppState;

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/:id/vote", post(vote))
        .with_state(state)
}

pub async fn vote(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}