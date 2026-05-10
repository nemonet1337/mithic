//! Custom emoji API endpoints
//!
//! Provides API for managing instance custom emojis.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{CreateEmojiRequest, EmojiCategory, EmojiResponse, UpdateEmojiRequest},
    state::{AppState, AuthUser},
};

/// Get all public emojis
pub async fn get_public_emojis(
    State(state): State<AppState>,
) -> Result<Json<Vec<EmojiResponse>>> {
    let emojis = state
        .emoji_service()
        .get_public_emojis()
        .await?;

    let responses: Vec<EmojiResponse> = emojis
        .into_iter()
        .map(EmojiResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Get all emojis (admin only)
pub async fn get_all_emojis(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<EmojiResponse>>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let emojis = state
        .emoji_service()
        .get_all_emojis()
        .await?;

    let responses: Vec<EmojiResponse> = emojis
        .into_iter()
        .map(EmojiResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Get emojis by category
pub async fn get_emojis_by_category(
    Path(category): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<EmojiResponse>>> {
    let category = match category.as_str() {
        "general" => EmojiCategory::General,
        "activities" => EmojiCategory::Activities,
        "animals" => EmojiCategory::Animals,
        "flags" => EmojiCategory::Flags,
        "food" => EmojiCategory::Food,
        "objects" => EmojiCategory::Objects,
        "people" => EmojiCategory::People,
        "nature" => EmojiCategory::Nature,
        "symbols" => EmojiCategory::Symbols,
        _ => EmojiCategory::Uncategorized,
    };

    let emojis = state
        .emoji_service()
        .get_emojis_by_category(category)
        .await?;

    let responses: Vec<EmojiResponse> = emojis
        .into_iter()
        .map(EmojiResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Search emojis
pub async fn search_emojis(
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Vec<EmojiResponse>>> {
    let query = params.get("q").cloned().unwrap_or_default();

    let emojis = state
        .emoji_service()
        .search_emojis(&query)
        .await?;

    let responses: Vec<EmojiResponse> = emojis
        .into_iter()
        .map(EmojiResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Create a new emoji (admin only)
pub async fn create_emoji(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateEmojiRequest>,
) -> Result<Json<EmojiResponse>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let emoji = state
        .emoji_service()
        .create_emoji(request)
        .await?;

    Ok(Json(EmojiResponse::from(emoji)))
}

/// Get a specific emoji
pub async fn get_emoji(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<EmojiResponse>> {
    let emoji_id = id.parse().map_err(|_| AppError::BadRequest("Invalid emoji ID".to_string()))?;
    
    let emoji = state
        .emoji_service()
        .get_emoji(&emoji_id)
        .await?;

    Ok(Json(EmojiResponse::from(emoji)))
}

/// Update an emoji (admin only)
pub async fn update_emoji(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<UpdateEmojiRequest>,
) -> Result<Json<EmojiResponse>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let emoji_id = id.parse().map_err(|_| AppError::BadRequest("Invalid emoji ID".to_string()))?;
    
    let emoji = state
        .emoji_service()
        .update_emoji(&emoji_id, request)
        .await?;

    Ok(Json(EmojiResponse::from(emoji)))
}

/// Delete an emoji (admin only)
pub async fn delete_emoji(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<()>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let emoji_id = id.parse().map_err(|_| AppError::BadRequest("Invalid emoji ID".to_string()))?;

    state
        .emoji_service()
        .delete_emoji(&emoji_id)
        .await?;

    Ok(Json(()))
}

/// Copy a remote emoji to local (admin only)
pub async fn copy_emoji(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<EmojiResponse>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let emoji_id = id.parse().map_err(|_| AppError::BadRequest("Invalid emoji ID".to_string()))?;

    let emoji = state
        .emoji_service()
        .get_emoji(&emoji_id)
        .await?;

    // Copy the emoji as a local emoji
    let copied = state
        .emoji_service()
        .copy_emoji(&emoji_id)
        .await?;

    Ok(Json(EmojiResponse::from(copied)))
}

/// List remote emojis (admin only)
pub async fn list_remote_emojis(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<EmojiResponse>>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }

    let host = params.get("host").cloned();
    let limit: usize = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(10);

    let emojis = state
        .emoji_service()
        .list_remote_emojis(host, limit)
        .await?;

    let responses: Vec<EmojiResponse> = emojis
        .into_iter()
        .map(EmojiResponse::from)
        .collect();

    Ok(Json(responses))
}
