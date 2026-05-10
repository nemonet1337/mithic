//! Admin API endpoints
//!
//! Admin-only endpoints for monitoring and management.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::actor::ActorId,
    services::federation::{QueueJob, QueueStats},
    state::{AppState, AuthUser},
};

#[derive(Debug, Deserialize)]
pub struct QueueJobsQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminDriveFilesQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserFilesRequest {
    pub user_id: String,
}

/// Get queue statistics (admin only)
pub async fn get_queue_stats(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<QueueStats>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let stats = state.federation_service().get_queue_stats().await
        .map_err(|e| AppError::Internal(format!("Failed to get queue stats: {}", e)))?;

    Ok(Json(stats))
}

/// Get queue jobs (admin only)
pub async fn get_queue_jobs(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<QueueJobsQuery>,
) -> Result<Json<Vec<QueueJob>>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let limit = query.limit.unwrap_or(50);
    let jobs = state.federation_service().get_queue_jobs(limit).await
        .map_err(|e| AppError::Internal(format!("Failed to get queue jobs: {}", e)))?;

    Ok(Json(jobs))
}

/// Show users (admin only)
pub async fn show_users(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<UsersQuery>,
) -> Result<Json<serde_json::Value>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let search_query = query.query.clone();

    let mut query_str = "SELECT * FROM user LIMIT $limit START $offset".to_string();
    let mut result = state.surreal()
        .query(&query_str)
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await
        .map_err(|e| AppError::Database(e))?;

    let users: Vec<serde_json::Value> = result.take(0).map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "users": users,
        "count": users.len()
    })))
}

/// Delete user account (admin only)
pub async fn delete_user_account(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let actor_id = ActorId::from_str(&user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Delete user
    state.surreal()
        .query("DELETE FROM user WHERE id = $id")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User deleted successfully"
    })))
}

/// Suspend user (admin only)
pub async fn suspend_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let actor_id = ActorId::from_str(&user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Update user to set is_suspended flag
    state.surreal()
        .query("UPDATE user:$id SET is_suspended = true")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    // TODO: Remove all following relationships
    // TODO: Send Delete activity via federation

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User suspended successfully"
    })))
}

/// Unsuspend user (admin only)
pub async fn unsuspend_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let actor_id = ActorId::from_str(&user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Update user to unset is_suspended flag
    state.surreal()
        .query("UPDATE user:$id SET is_suspended = false")
        .bind(("id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    // TODO: Send Update activity via federation

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User unsuspended successfully"
    })))
}

/// Clear federation queue (admin only)
pub async fn clear_queue(
    auth_user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // Clear the federation queue in Dragonfly
    redis::cmd("DEL")
        .arg("federation:queue")
        .query_async::<_, ()>(&mut state.dragonfly().clone())
        .await
        .ok();

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Queue cleared successfully"
    })))
}

/// Get all drive files (admin only)
pub async fn get_all_drive_files(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<AdminDriveFilesQuery>,
) -> Result<Json<serde_json::Value>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let mut query_str = "SELECT * FROM file LIMIT $limit START $offset".to_string();

    if let Some(user_id) = query.user_id {
        query_str = format!("{} WHERE user_id = $user_id", query_str);
        let mut result = state
            .surreal()
            .query(&query_str)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .bind(("user_id", user_id))
            .await
            .map_err(|e| AppError::Database(e))?;

        let files: Vec<serde_json::Value> = result.take(0).map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(Json(serde_json::json!({
            "files": files,
            "count": files.len()
        })))
    } else {
        let mut result = state
            .surreal()
            .query(&query_str)
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await
            .map_err(|e| AppError::Database(e))?;

        let files: Vec<serde_json::Value> = result.take(0).map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(Json(serde_json::json!({
            "files": files,
            "count": files.len()
        })))
    }
}

/// Delete all files of a user (admin only)
pub async fn delete_all_files_of_a_user(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<DeleteUserFilesRequest>,
) -> Result<Json<serde_json::Value>> {
    if !auth_user.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    let actor_id = ActorId::from_str(&req.user_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid user ID: {}", e)))?;

    // Delete all files of the user
    state.surreal()
        .query("DELETE FROM file WHERE user_id = $user_id")
        .bind(("user_id", actor_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "All files deleted successfully"
    })))
}
