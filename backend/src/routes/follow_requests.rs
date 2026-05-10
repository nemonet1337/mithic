use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::actor::ActorId,
    services::follow_request::{FollowRequest, FollowRequestService},
    state::{AppState, AuthUser},
};

#[derive(Debug, Serialize)]
pub struct FollowRequestResponse {
    pub id: String,
    pub created_at: String,
    pub follower_id: String,
    pub followee_id: String,
    pub request_message: Option<String>,
}

impl From<FollowRequest> for FollowRequestResponse {
    fn from(req: FollowRequest) -> Self {
        Self {
            id: req.id.to_string(),
            created_at: req.created_at.to_rfc3339(),
            follower_id: req.follower_id.to_string(),
            followee_id: req.followee_id.to_string(),
            request_message: req.request_message,
        }
    }
}

/// 自分に届いたフォローリクエストの一覧を取得
pub async fn list_received_requests(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<Vec<FollowRequestResponse>>> {
    let requests = FollowRequestService::list_received_requests(&state, auth_user.user_id).await?;

    Ok(Json(requests.into_iter().map(FollowRequestResponse::from).collect()))
}

/// フォローリクエストを承認
pub async fn accept_request(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let follower_id = user_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

    FollowRequestService::accept_request(&state, auth_user.user_id, follower_id).await?;

    Ok(Json(serde_json::json!({})))
}

/// フォローリクエストを拒否
pub async fn reject_request(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let follower_id = user_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

    FollowRequestService::reject_request(&state, auth_user.user_id, follower_id).await?;

    Ok(Json(serde_json::json!({})))
}

/// フォローリクエストをキャンセル
pub async fn cancel_request(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let followee_id = user_id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid user ID".to_string()))?;

    FollowRequestService::cancel_request(&state, auth_user.user_id, followee_id).await?;

    Ok(Json(serde_json::json!({})))
}
