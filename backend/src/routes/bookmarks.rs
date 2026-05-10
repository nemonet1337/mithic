//! Bookmark API endpoints
//!
//! Provides API for managing user bookmarks.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{BookmarkResponse, NoteId},
    state::{AppState, AuthUser},
};

/// Query parameters for bookmark list
#[derive(Debug, Deserialize)]
pub struct BookmarkListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    40
}

/// Get user's bookmarks
pub async fn get_bookmarks(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<BookmarkListQuery>,
) -> Result<Json<Vec<BookmarkResponse>>> {
    let bookmarks = state
        .bookmark_service()
        .get_bookmarks(&auth_user.user_id.into(), query.limit, query.offset)
        .await?;

    let responses: Vec<BookmarkResponse> = bookmarks
        .into_iter()
        .map(BookmarkResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Create a bookmark
pub async fn create_bookmark(
    auth_user: AuthUser,
    Path(note_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<BookmarkResponse>> {
    let note_id = note_id.parse().map_err(|_| AppError::BadRequest("Invalid note ID".to_string()))?;
    
    let bookmark = state
        .bookmark_service()
        .create_bookmark(&auth_user.user_id.into(), &note_id)
        .await?;

    info!("User {} bookmarked note {}", auth_user.user_id, note_id);

    Ok(Json(BookmarkResponse::from(bookmark)))
}

/// Delete a bookmark
pub async fn delete_bookmark(
    auth_user: AuthUser,
    Path(note_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<()>> {
    let note_id = note_id.parse().map_err(|_| AppError::BadRequest("Invalid note ID".to_string()))?;
    
    state
        .bookmark_service()
        .delete_bookmark(&auth_user.user_id.into(), &note_id)
        .await?;

    info!("User {} removed bookmark for note {}", auth_user.user_id, note_id);

    Ok(Json(()))
}
