use axum::{extract::State, routing::get, response::IntoResponse};
use mithic_core::Result;

use crate::state::AppState;

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(stream))
        .with_state(state)
}

pub async fn stream(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let _ = state;
    // Delegate to existing stream infrastructure
    Ok("Streaming endpoint - to be implemented")
}