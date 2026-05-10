//! Mute API endpoints
//!
//! Provides API for muting and unmuting users.

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use tracing::{error, info};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{Actor, Mute, MuteResponse, CreateMuteRequest, MuteListQuery},
    state::{AppState, AuthUser},
};

/// Create a new mute
pub async fn create_mute(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateMuteRequest>,
) -> Result<Json<MuteResponse>> {
    let mutee_id = Ulid::from_string(&req.user_id)
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

    // Check if user exists
    let mutee: Option<Actor> = state
        .surreal()
        .select(("user", mutee_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    if mutee.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    let mutee = mutee.unwrap();

    // Can't mute yourself
    if auth_user.user_id == mutee.id {
        return Err(AppError::Validation("Cannot mute yourself".to_string()));
    }

    // Check if already muted
    let existing: Option<Mute> = state
        .surreal()
        .query("SELECT * FROM mute WHERE muter_id = $muter_id AND mutee_id = $mutee_id")
        .bind(("muter_id", auth_user.user_id.to_string()))
        .bind(("mutee_id", mutee.id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .ok()
        .flatten();

    if existing.is_some() {
        return Err(AppError::Validation("Already muted".to_string()));
    }

    // Calculate expiration
    let expires_at = req.expires_in.map(|seconds| {
        Utc::now() + chrono::Duration::seconds(seconds)
    });

    // Create mute relationship using RELATE
    let mute = Mute::new(auth_user.user_id, mutee.id.clone(), expires_at);
    
    state.surreal()
        .query("RELATE user:$muter->mute->user:$mutee SET created_at = $created_at, expires_at = $expires_at")
        .bind(("muter", auth_user.user_id.to_string()))
        .bind(("mutee", mutee.id.to_string()))
        .bind(("created_at", mute.created_at))
        .bind(("expires_at", mute.expires_at))
        .await
        .map_err(|e| {
            error!("Failed to create mute: {}", e);
            AppError::Database(e)
        })?;

    info!(
        "User {} muted user {} (expires: {:?})",
        auth_user.user_id, mutee.id, expires_at
    );

    Ok(Json(MuteResponse::from(mute)))
}

/// Delete a mute (unmute)
pub async fn delete_mute(
    auth_user: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(mute_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>> {
    let mute_ulid = Ulid::from_string(&mute_id)
        .map_err(|_| AppError::Validation("Invalid mute ID".to_string()))?;

    // Find the mute relationship
    let mute: Option<Mute> = state
        .surreal()
        .select(("mute", mute_ulid.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let mute = mute.ok_or_else(|| AppError::NotFound("Mute not found".to_string()))?;

    // Verify ownership
    if mute.muter_id != auth_user.user_id {
        return Err(AppError::Forbidden("Not authorized".to_string()));
    }

    // Delete the mute relationship
    state.surreal()
        .query("DELETE mute WHERE in = $muter_id AND out = $mutee_id")
        .bind(("muter_id", mute.muter_id.to_string()))
        .bind(("mutee_id", mute.mutee_id.to_string()))
        .await
        .map_err(|e| {
            error!("Failed to delete mute: {}", e);
            AppError::Database(e)
        })?;

    info!(
        "User {} unmuted user {}",
        auth_user.user_id, mute.mutee_id
    );

    Ok(Json(serde_json::json!({"success": true})))
}

/// Get list of muted users
pub async fn get_mutes(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<MuteListQuery>,
) -> Result<Json<Vec<MuteResponse>>> {
    let limit = query.limit();
    let offset = query.offset();

    let mutes: Vec<Mute> = state
        .surreal()
        .query("SELECT * FROM mute WHERE muter_id = $muter_id LIMIT $limit START $offset")
        .bind(("muter_id", auth_user.user_id.to_string()))
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    // Filter out expired mutes
    let active_mutes: Vec<Mute> = mutes
        .into_iter()
        .filter(|m| m.is_active())
        .collect();

    let responses: Vec<MuteResponse> = active_mutes
        .into_iter()
        .map(MuteResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Check if a user is muted
pub async fn is_muting(
    auth_user: AuthUser,
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>> {
    let target_id = Ulid::from_string(&user_id)
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

    let mute: Option<Mute> = state
        .surreal()
        .query("SELECT * FROM mute WHERE muter_id = $muter_id AND mutee_id = $mutee_id")
        .bind(("muter_id", auth_user.user_id.to_string()))
        .bind(("mutee_id", target_id.to_string()))
        .await
        .and_then(|mut res| res.take(0))
        .ok()
        .flatten();

    // Check if mute is active (not expired)
    let is_muting = mute.map(|m| m.is_active()).unwrap_or(false);

    Ok(Json(serde_json::json!({
        "muting": is_muting
    })))
}

/// Delete expired mutes (cleanup endpoint - can be called periodically)
pub async fn cleanup_expired_mutes(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let now = Utc::now();

    let deleted: Vec<Mute> = state
        .surreal()
        .query("DELETE mute WHERE expires_at < $now RETURN BEFORE")
        .bind(("now", now))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    info!("Cleaned up {} expired mutes", deleted.len());

    Ok(Json(serde_json::json!({
        "deleted": deleted.len()
    })))
}
