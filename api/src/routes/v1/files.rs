use axum::{Json, extract::State, http::StatusCode, routing::{get, delete, post, patch}};
use mithic_core::{AppError, Result};
use mithic_core::models::file::DriveFile;

use crate::state::AppState;

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(list))
        .route("/:id", delete(delete_file))
        .route("/upload-from-url", post(upload_from_url))
        .with_state(state)
}

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<DriveFile>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn delete_file(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn upload_from_url(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}