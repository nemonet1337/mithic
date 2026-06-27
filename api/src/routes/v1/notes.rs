use axum::{Json, extract::State, routing::{delete, get, post}, http::StatusCode};
use mithic_core::{AppError, Result};
use serde::Deserialize;

use crate::dto::actor_to_user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub text: Option<String>,
    pub visibility: Option<shared::NoteVisibility>,
    pub cw: Option<String>,
    pub file_ids: Option<Vec<String>>,
    pub reply_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(create))
        .route("/:id", get(show))
        .route("/:id", delete(delete_note))
        .route("/:id/replies", get(replies))
        .route("/:id/quotes", get(quotes))
        .route("/:id/reactions", post(create_reaction))
        .route("/:id/reactions", delete(delete_reaction))
        .route("/:id/renote", post(renote))
        .route("/:id/bookmark", post(bookmark))
        .route("/:id/pin", post(pin))
        .with_state(state)
}

pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<shared::Note>> {
    let _ = (state, request);
    Ok(Json(shared::Note {
        id: String::new(),
        created_at: String::new(),
        author: shared::User {
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
        },
        content: String::new(),
        cw: None,
        visibility: shared::NoteVisibility::Public,
        reactions: Vec::new(),
        reply_count: 0,
        renote_count: 0,
        quote_count: 0,
        attachments: Vec::new(),
        tags: Vec::new(),
        is_nsfw: false,
    }))
}

pub async fn show(
    State(state): State<AppState>,
) -> Result<Json<shared::Note>> {
    let _ = state;
    Ok(Json(shared::Note {
        id: String::new(),
        created_at: String::new(),
        author: shared::User {
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
        },
        content: String::new(),
        cw: None,
        visibility: shared::NoteVisibility::Public,
        reactions: Vec::new(),
        reply_count: 0,
        renote_count: 0,
        quote_count: 0,
        attachments: Vec::new(),
        tags: Vec::new(),
        is_nsfw: false,
    }))
}

pub async fn delete_note(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn replies(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn quotes(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::Note>>> {
    let _ = state;
    Ok(Json(Vec::new()))
}

pub async fn create_reaction(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_reaction(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn renote(
    State(state): State<AppState>,
) -> Result<Json<shared::Note>> {
    let _ = state;
    Ok(Json(shared::Note {
        id: String::new(),
        created_at: String::new(),
        author: shared::User {
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
        },
        content: String::new(),
        cw: None,
        visibility: shared::NoteVisibility::Public,
        reactions: Vec::new(),
        reply_count: 0,
        renote_count: 0,
        quote_count: 0,
        attachments: Vec::new(),
        tags: Vec::new(),
        is_nsfw: false,
    }))
}

pub async fn bookmark(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn pin(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}