//! Antenna API endpoints
//!
//! Provides API for managing keyword-based note monitoring.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{
        ActorId, AntennaResponse, CreateAntennaRequest, NoteId, UpdateAntennaRequest,
    },
    state::{AppState, AuthUser},
};

/// Get all antennas for the authenticated user
pub async fn get_antennas(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<AntennaResponse>>> {
    let antennas = state
        .antenna_service()
        .get_user_antennas(&auth_user.user_id.into())
        .await?;

    let responses: Vec<AntennaResponse> = antennas
        .into_iter()
        .map(AntennaResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Create a new antenna
pub async fn create_antenna(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateAntennaRequest>,
) -> Result<Json<AntennaResponse>> {
    let antenna = state
        .antenna_service()
        .create_antenna(&auth_user.user_id.into(), request)
        .await?;

    Ok(Json(AntennaResponse::from(antenna)))
}

/// Get a specific antenna
pub async fn get_antenna(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<AntennaResponse>> {
    let antenna_id = id.parse().map_err(|_| AppError::BadRequest("Invalid antenna ID".to_string()))?;
    
    let antenna = state
        .antenna_service()
        .get_antenna(&antenna_id)
        .await?;

    // Check ownership
    if antenna.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this antenna".to_string()));
    }

    Ok(Json(AntennaResponse::from(antenna)))
}

/// Update an antenna
pub async fn update_antenna(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<UpdateAntennaRequest>,
) -> Result<Json<AntennaResponse>> {
    let antenna_id = id.parse().map_err(|_| AppError::BadRequest("Invalid antenna ID".to_string()))?;
    
    // Check ownership first
    let existing = state
        .antenna_service()
        .get_antenna(&antenna_id)
        .await?;

    if existing.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this antenna".to_string()));
    }

    let antenna = state
        .antenna_service()
        .update_antenna(&antenna_id, request)
        .await?;

    Ok(Json(AntennaResponse::from(antenna)))
}

/// Delete an antenna
pub async fn delete_antenna(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<()>> {
    let antenna_id = id.parse().map_err(|_| AppError::BadRequest("Invalid antenna ID".to_string()))?;
    
    // Check ownership first
    let existing = state
        .antenna_service()
        .get_antenna(&antenna_id)
        .await?;

    if existing.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this antenna".to_string()));
    }

    state
        .antenna_service()
        .delete_antenna(&antenna_id)
        .await?;

    Ok(Json(()))
}

/// Get notes for an antenna
pub async fn get_antenna_notes(
    auth_user: AuthUser,
    Path(id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Vec<NoteId>>> {
    let antenna_id = id.parse().map_err(|_| AppError::BadRequest("Invalid antenna ID".to_string()))?;
    
    // Check ownership first
    let existing = state
        .antenna_service()
        .get_antenna(&antenna_id)
        .await?;

    if existing.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this antenna".to_string()));
    }

    let limit: u32 = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let note_ids = state
        .antenna_service()
        .get_antenna_notes(&antenna_id, limit)
        .await?;

    Ok(Json(note_ids))
}

/// Remove a note from an antenna
pub async fn remove_note_from_antenna(
    auth_user: AuthUser,
    Path((antenna_id, note_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<()>> {
    let antenna_id = antenna_id.parse().map_err(|_| AppError::BadRequest("Invalid antenna ID".to_string()))?;
    let note_id = note_id.parse().map_err(|_| AppError::BadRequest("Invalid note ID".to_string()))?;
    
    // Check ownership first
    let existing = state
        .antenna_service()
        .get_antenna(&antenna_id)
        .await?;

    if existing.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this antenna".to_string()));
    }

    state
        .antenna_service()
        .remove_note_from_antenna(&antenna_id, &note_id)
        .await?;

    Ok(Json(()))
}
