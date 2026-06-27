use axum::{Json, extract::State, routing::{get, patch, post}};
use mithic_core::{AppError, Result};
use serde::Deserialize;

use crate::dto::actor_to_user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CheckHandleRequest {
    pub username: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AvailableResponse {
    pub available: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub is_locked: Option<bool>,
    pub is_bot: Option<bool>,
}

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/:id", get(show))
        .route("/me", patch(update_me))
        .route("/search", post(search))
        .route("/check-handle", post(check_handle))
        .with_state(state)
}

pub async fn show(
    State(state): State<AppState>,
) -> Result<Json<shared::User>> {
    Ok(Json(shared::User {
        id: String::new(),
        username: String::new(),
        host: None,
        display_name: None,
        bio: None,
        avatar_url: None,
        banner_url: None,
        followers_count: 0,
        following_count: 0,
        notes_count: 0,
        is_locked: false,
        is_bot: false,
    }))
}

pub async fn update_me(
    State(state): State<AppState>,
) -> Result<Json<shared::User>> {
    let _ = state;
    Ok(Json(shared::User {
        id: String::new(),
        username: String::new(),
        host: None,
        display_name: None,
        bio: None,
        avatar_url: None,
        banner_url: None,
        followers_count: 0,
        following_count: 0,
        notes_count: 0,
        is_locked: false,
        is_bot: false,
    }))
}

pub async fn search(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::User>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn check_handle(
    State(state): State<AppState>,
    Json(request): Json<CheckHandleRequest>,
) -> Result<Json<AvailableResponse>> {
    let _ = (state, request);
    Ok(Json(AvailableResponse { available: true }))
}