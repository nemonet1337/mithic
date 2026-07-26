//! Web Push subscription

use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::{AuthUser, Result};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

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

pub async fn subscribe(
    Extension(_auth): Extension<AuthUser>,
    State(_state): State<AppState>,
    Json(request): Json<PushSubscriptionRequest>,
) -> Result<Json<PushSubscriptionResponse>> {
    // ponytail: web-push 配送は未実装。登録レスポンスのみ返す
    Ok(Json(PushSubscriptionResponse {
        endpoint: request.endpoint,
        keys: request.keys,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn get_subscription(
    Extension(_auth): Extension<AuthUser>,
    State(_state): State<AppState>,
) -> Result<StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unsubscribe(
    Extension(_auth): Extension<AuthUser>,
    State(_state): State<AppState>,
) -> Result<StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}
