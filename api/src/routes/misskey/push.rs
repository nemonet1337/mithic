//! Web Push サブスクリプション管理 (TODO: web-push クレートによる配送)

use crate::state::AppState;
use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::{AppError, AuthUser, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionRequest {
    pub endpoint: String,
    pub keys: PushKeys,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscriptionResponse {
    pub endpoint: String,
    pub keys: PushKeys,
    pub created_at: String,
}

/// POST /push/subscription — 購読登録
pub async fn subscribe(
    Extension(auth): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<PushSubscriptionRequest>,
) -> Result<Json<PushSubscriptionResponse>> {
    let _ = state;
    let _ = auth;
    Ok(Json(PushSubscriptionResponse {
        endpoint: request.endpoint,
        keys: request.keys,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// GET /push/subscription — 購読状態確認
pub async fn get_subscription(
    Extension(_auth): Extension<AuthUser>,
    State(_state): State<AppState>,
) -> Result<StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /push/subscription — 購読解除
pub async fn unsubscribe(
    Extension(_auth): Extension<AuthUser>,
    State(_state): State<AppState>,
) -> Result<StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}
