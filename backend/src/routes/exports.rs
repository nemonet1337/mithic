//! Import/Export API endpoints
//!
//! Provides API for data export and import.

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{CreateExportRequest, ExportResponse, ImportRequest, ImportResponse},
    state::{AppState, AuthUser},
};

/// Get user's exports
pub async fn get_exports(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ExportResponse>>> {
    let exports = state
        .export_service()
        .get_user_exports(&auth_user.user_id.into())
        .await?;

    let responses: Vec<ExportResponse> = exports
        .into_iter()
        .map(ExportResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Create a new export
pub async fn create_export(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateExportRequest>,
) -> Result<Json<ExportResponse>> {
    let export = state
        .export_service()
        .create_export(&auth_user.user_id.into(), request)
        .await?;

    // Process export asynchronously (in production, use a background job)
    let export_id = export.id;
    let service = state.export_service().clone();
    tokio::spawn(async move {
        if let Err(e) = service.process_export(&export_id).await {
            tracing::error!("Failed to process export {}: {}", export_id, e);
        }
    });

    Ok(Json(ExportResponse::from(export)))
}

/// Get a specific export
pub async fn get_export(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ExportResponse>> {
    let export_id = id.parse().map_err(|_| AppError::BadRequest("Invalid export ID".to_string()))?;
    
    let export = state
        .export_service()
        .get_export(&export_id)
        .await?;

    // Check ownership
    if export.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this export".to_string()));
    }

    Ok(Json(ExportResponse::from(export)))
}

/// Delete an export
pub async fn delete_export(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<()>> {
    let export_id = id.parse().map_err(|_| AppError::BadRequest("Invalid export ID".to_string()))?;
    
    // Check ownership first
    let existing = state
        .export_service()
        .get_export(&export_id)
        .await?;

    if existing.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this export".to_string()));
    }

    state
        .export_service()
        .delete_export(&export_id)
        .await?;

    Ok(Json(()))
}

/// Get user's imports
pub async fn get_imports(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ImportResponse>>> {
    let imports = state
        .export_service()
        .get_user_imports(&auth_user.user_id.into())
        .await?;

    let responses: Vec<ImportResponse> = imports
        .into_iter()
        .map(ImportResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Create a new import
pub async fn create_import(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<ImportRequest>,
) -> Result<Json<ImportResponse>> {
    let import = state
        .export_service()
        .create_import(&auth_user.user_id.into(), request)
        .await?;

    // Process import asynchronously (in production, use a background job)
    let import_id = import.id;
    let service = state.export_service().clone();
    tokio::spawn(async move {
        if let Err(e) = service.process_import(&import_id).await {
            tracing::error!("Failed to process import {}: {}", import_id, e);
        }
    });

    Ok(Json(ImportResponse::from(import)))
}

/// Get a specific import
pub async fn get_import(
    auth_user: AuthUser,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ImportResponse>> {
    let import_id = id.parse().map_err(|_| AppError::BadRequest("Invalid import ID".to_string()))?;
    
    let import = state
        .export_service()
        .get_import(&import_id)
        .await?;

    // Check ownership
    if import.user_id != auth_user.user_id.into() {
        return Err(AppError::Forbidden("You don't own this import".to_string()));
    }

    Ok(Json(ImportResponse::from(import)))
}
