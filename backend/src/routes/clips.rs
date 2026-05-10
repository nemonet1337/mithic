//! Clip API endpoints
//!
//! Provides API for managing user clips.

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::{info, warn};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{CreateClipRequest, UpdateClipRequest, AddNoteToClipRequest, Clip, ClipNote, ClipResponse, ClipWithNotes, PublicClipResponse, ClipId},
    state::{AppState, AuthUser},
};

/// List user's clips
///
/// Returns all clips owned by the authenticated user.
pub async fn get_clips(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ClipResponse>>> {
    let user_id = auth_user.user_id;
    
    let clips: Vec<Clip> = state.surreal()
        .query("SELECT * FROM clip WHERE user_id = $user_id ORDER BY created_at DESC")
        .bind(("user_id", user_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();
    
    let responses: Vec<ClipResponse> = clips
        .into_iter()
        .map(|c| c.into())
        .collect();
    
    Ok(Json(responses))
}

/// Create a new clip
///
/// Creates a new clip with the given name and visibility.
pub async fn create_clip(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateClipRequest>,
) -> Result<Json<ClipResponse>> {
    let user_id = auth_user.user_id;
    
    if request.name.is_empty() {
        return Err(AppError::Validation("Clip name cannot be empty".to_string()));
    }
    
    if request.name.len() > 100 {
        return Err(AppError::Validation("Clip name too long (max 100 chars)".to_string()));
    }
    
    let clip = Clip::new(user_id, request.name, request.is_public, request.description);
    
    state.surreal()
        .create::<Option<Clip>>(
            ("clip", clip.id.to_string()),
        )
        .content(clip.clone())
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} created clip: {}", auth_user.username, clip.name);
    
    Ok(Json(clip.into()))
}

/// Get a specific clip
///
/// Returns a single clip with its notes.
pub async fn get_clip(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClipWithNotes>> {
    let user_id = auth_user.user_id;
    let clip_id = id.parse::<ClipId>()
        .map_err(|_| AppError::Validation("Invalid clip ID".to_string()))?;
    
    let clip: Clip = state.surreal()
        .select(("clip", clip_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("Clip not found".to_string()))?;
    
    // Check ownership or public visibility
    if clip.user_id != user_id && !clip.is_public {
        return Err(AppError::Forbidden("You don't have access to this clip".to_string()));
    }
    
    // Get notes in the clip
    let clip_notes: Vec<ClipNote> = state.surreal()
        .query("SELECT * FROM clip_note WHERE clip_id = $clip_id ORDER BY created_at DESC")
        .bind(("clip_id", clip_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();
    
    let note_ids: Vec<String> = clip_notes
        .into_iter()
        .map(|cn| cn.note_id.to_string())
        .collect();
    
    let response = ClipWithNotes {
        id: clip.id.to_string(),
        name: clip.name,
        is_public: clip.is_public,
        description: clip.description,
        notes: note_ids,
        created_at: clip.created_at,
        updated_at: clip.updated_at,
    };
    
    Ok(Json(response))
}

/// Update a clip
///
/// Updates the name, visibility, or description of an existing clip.
pub async fn update_clip(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateClipRequest>,
) -> Result<Json<ClipResponse>> {
    let user_id = auth_user.user_id;
    let clip_id = id.parse::<ClipId>()
        .map_err(|_| AppError::Validation("Invalid clip ID".to_string()))?;
    
    // Validate name if provided
    if let Some(ref name) = request.name {
        if name.is_empty() {
            return Err(AppError::Validation("Clip name cannot be empty".to_string()));
        }
        if name.len() > 100 {
            return Err(AppError::Validation("Clip name too long (max 100 chars)".to_string()));
        }
    }
    
    let mut clip: Clip = state.surreal()
        .select(("clip", clip_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("Clip not found".to_string()))?;
    
    // Check ownership
    if clip.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this clip".to_string()));
    }
    
    clip.update(request.name, request.is_public, request.description);
    
    state.surreal()
        .update::<Option<Clip>>(
            ("clip", clip_id.to_string()),
        )
        .merge(clip.clone())
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} updated clip: {}", auth_user.username, clip.name);
    
    Ok(Json(clip.into()))
}

/// Delete a clip
///
/// Deletes a clip and all its notes.
pub async fn delete_clip(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let user_id = auth_user.user_id;
    let clip_id = id.parse::<ClipId>()
        .map_err(|_| AppError::Validation("Invalid clip ID".to_string()))?;
    
    let clip: Clip = state.surreal()
        .select(("clip", clip_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("Clip not found".to_string()))?;
    
    // Check ownership
    if clip.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this clip".to_string()));
    }
    
    // Delete clip notes first
    state.surreal()
        .query("DELETE clip_note WHERE clip_id = $clip_id")
        .bind(("clip_id", clip_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;
    
    // Delete the clip
    state.surreal()
        .delete(("clip", clip_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} deleted clip: {}", auth_user.username, clip.name);
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// Add note to clip
///
/// Adds a note to the specified clip.
pub async fn add_note_to_clip(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(clip_id): Path<String>,
    Json(request): Json<AddNoteToClipRequest>,
) -> Result<Json<serde_json::Value>> {
    let user_id = auth_user.user_id;
    let clip_id = clip_id.parse::<ClipId>()
        .map_err(|_| AppError::Validation("Invalid clip ID".to_string()))?;
    
    let note_id = request.note_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;
    
    let clip: Clip = state.surreal()
        .select(("clip", clip_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("Clip not found".to_string()))?;
    
    // Check ownership
    if clip.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this clip".to_string()));
    }
    
    // Check if already in clip
    let existing: Option<ClipNote> = state.surreal()
        .query("SELECT * FROM clip_note WHERE clip_id = $clip_id AND note_id = $note_id")
        .bind(("clip_id", clip_id.to_string()))
        .bind(("note_id", note_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default()
        .into_iter()
        .next();
    
    if existing.is_some() {
        return Err(AppError::Conflict("Note is already in this clip".to_string()));
    }
    
    let clip_note = ClipNote::new(clip_id, note_id);
    
    state.surreal()
        .create::<Option<ClipNote>>(
            ("clip_note", clip_note.id.to_string()),
        )
        .content(clip_note)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} added note {} to clip {}", auth_user.username, note_id, clip_id);
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// Remove note from clip
///
/// Removes a note from the specified clip.
pub async fn remove_note_from_clip(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path((clip_id, note_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let user_id = auth_user.user_id;
    let clip_id = clip_id.parse::<ClipId>()
        .map_err(|_| AppError::Validation("Invalid clip ID".to_string()))?;
    
    let note_id = note_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid note ID".to_string()))?;
    
    let clip: Clip = state.surreal()
        .select(("clip", clip_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or_else(|| AppError::NotFound("Clip not found".to_string()))?;
    
    // Check ownership
    if clip.user_id != user_id {
        return Err(AppError::Forbidden("You don't own this clip".to_string()));
    }
    
    state.surreal()
        .query("DELETE clip_note WHERE clip_id = $clip_id AND note_id = $note_id")
        .bind(("clip_id", clip_id.to_string()))
        .bind(("note_id", note_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;
    
    info!("User {} removed note {} from clip {}", auth_user.username, note_id, clip_id);
    
    Ok(Json(serde_json::json!({
        "success": true
    })))
}

/// Get public clips for a user
///
/// Returns public clips for a specific user.
pub async fn get_user_public_clips(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<PublicClipResponse>>> {
    let target_user_id = user_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;
    
    let clips: Vec<Clip> = state.surreal()
        .query("SELECT * FROM clip WHERE user_id = $user_id AND is_public = true ORDER BY created_at DESC")
        .bind(("user_id", target_user_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();
    
    let responses: Vec<PublicClipResponse> = clips
        .into_iter()
        .map(|c| PublicClipResponse {
            id: c.id.to_string(),
            name: c.name,
            description: c.description,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect();
    
    Ok(Json(responses))
}
