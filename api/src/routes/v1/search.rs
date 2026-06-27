use axum::{Json, extract::State, http::StatusCode, routing::get};
use mithic_core::Result;

use crate::state::AppState;

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(search))
        .with_state(state)
}

pub async fn search(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}