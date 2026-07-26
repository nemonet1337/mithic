use axum::{Json, extract::State, http::StatusCode, routing::{get, post}};
use mithic_core::Result;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub participant_ids: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct Conversation {
    pub id: String,
    pub participant_ids: Vec<String>,
    pub last_message: Option<String>,
}

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(list))
        .route("/", post(create))
        .route("/{id}/messages", get(messages))
        .with_state(state)
}

pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<Conversation>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn create(
    State(state): State<AppState>,
) -> Result<Json<Conversation>> {
    let _ = state;
    Ok(Json(Conversation {
        id: String::new(),
        participant_ids: Vec::new(),
        last_message: None,
    }))
}

pub async fn messages(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}