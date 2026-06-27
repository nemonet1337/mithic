use axum::{Json, extract::State, http::StatusCode, routing::{delete, get, post}};
use mithic_core::Result;

use crate::state::AppState;

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(create_follow))
        .route("/:id", delete(delete_follow))
        .route("/requests", get(follow_requests))
        .route("/requests/:id/accept", post(accept_follow_request))
        .route("/requests/:id/reject", post(reject_follow_request))
        .with_state(state)
}

pub async fn create_follow(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_follow(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn follow_requests(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::User>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn accept_follow_request(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reject_follow_request(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}