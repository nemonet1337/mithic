//! 管理API — アカウント凍結・解除・削除

use crate::state::AppState;
use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::models::actor::ActorId;
use mithic_core::{AppError, AuthUser, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountActionRequest {
    pub user_id: String,
}

pub async fn suspend(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<AccountActionRequest>,
) -> Result<StatusCode> {
    if !auth.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }
    let target_id: ActorId = request
        .user_id
        .parse()
        .map_err(|_| AppError::Validation("Invalid user id".to_string()))?;
    state
        .surreal()
        .query("UPDATE user SET is_suspended = true WHERE id = type::record('user', $id)")
        .bind(("id", target_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unsuspend(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<AccountActionRequest>,
) -> Result<StatusCode> {
    if !auth.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }
    let target_id: ActorId = request
        .user_id
        .parse()
        .map_err(|_| AppError::Validation("Invalid user id".to_string()))?;
    state
        .surreal()
        .query("UPDATE user SET is_suspended = false WHERE id = type::record('user', $id)")
        .bind(("id", target_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_account(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<AccountActionRequest>,
) -> Result<StatusCode> {
    if !auth.is_admin {
        return Err(AppError::Forbidden("Admin only".to_string()));
    }
    let target_id: ActorId = request
        .user_id
        .parse()
        .map_err(|_| AppError::Validation("Invalid user id".to_string()))?;
    state
        .surreal()
        .query("DELETE user WHERE id = type::record('user', $id)")
        .bind(("id", target_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
