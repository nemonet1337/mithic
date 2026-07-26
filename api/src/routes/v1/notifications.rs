use axum::{Json, extract::State, http::StatusCode, routing::{get, post}};
use mithic_core::Result;

use crate::state::AppState;

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(list))
        .route("/read-all", post(read_all))
        .route("/{id}/read", post(read))
        .with_state(state)
}

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Notification>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn read_all(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn read(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}