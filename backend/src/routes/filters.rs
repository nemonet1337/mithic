//! Filter API endpoints
//!
//! Content filtering (word mute, regex filters).

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{CreateFilterRequest, FilterResponse, FilterId, UpdateFilterRequest},
    state::{AppState, AuthUser},
};

/// Get user's filters
pub async fn get_filters(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<FilterResponse>>> {
    let filters = state.filter_service().get_filters(auth_user.user_id.into()).await?;

    Ok(Json(filters.into_iter().map(FilterResponse::from).collect()))
}

/// Create filter
pub async fn create_filter(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateFilterRequest>,
) -> Result<Json<FilterResponse>> {
    let filter = state
        .filter_service()
        .create_filter(auth_user.user_id.into(), req)
        .await?;

    info!("Filter {} created by user {}", filter.id, auth_user.user_id);

    Ok(Json(FilterResponse::from(filter)))
}

/// Get filter by ID
pub async fn get_filter(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<FilterResponse>> {
    let filter_id = FilterId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid filter ID".to_string()))?;

    let filter = state
        .filter_service()
        .get_filter(filter_id, auth_user.user_id.into())
        .await?;

    Ok(Json(FilterResponse::from(filter)))
}

/// Update filter
pub async fn update_filter(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFilterRequest>,
) -> Result<Json<FilterResponse>> {
    let filter_id = FilterId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid filter ID".to_string()))?;

    let filter = state
        .filter_service()
        .update_filter(filter_id, auth_user.user_id.into(), req)
        .await?;

    info!("Filter {} updated by user {}", filter_id, auth_user.user_id);

    Ok(Json(FilterResponse::from(filter)))
}

/// Delete filter
pub async fn delete_filter(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<()> {
    let filter_id = FilterId::from_string(&id)
        .map_err(|_| AppError::BadRequest("Invalid filter ID".to_string()))?;

    state
        .filter_service()
        .delete_filter(filter_id, auth_user.user_id.into())
        .await?;

    info!("Filter {} deleted by user {}", filter_id, auth_user.user_id);

    Ok(())
}
