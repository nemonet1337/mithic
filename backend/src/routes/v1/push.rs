//! Web Push subscription CRUD

use crate::db::queries::{
    delete_push_subscriptions_for_user, list_push_subscriptions, upsert_push_subscription,
};
use crate::{AppError, AuthUser, Result};
use axum::{Extension, Json, extract::State, http::StatusCode};
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
    Extension(auth): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<PushSubscriptionRequest>,
) -> Result<Json<PushSubscriptionResponse>> {
    if request.endpoint.is_empty() || request.keys.p256dh.is_empty() || request.keys.auth.is_empty()
    {
        return Err(AppError::Validation(
            "endpoint, keys.p256dh and keys.auth are required".to_string(),
        ));
    }
    if !(request.endpoint.starts_with("https://") || request.endpoint.starts_with("http://")) {
        return Err(AppError::Validation(
            "endpoint must be an http(s) URL".to_string(),
        ));
    }

    let sub = upsert_push_subscription(
        state.surreal(),
        &auth.user_id,
        &request.endpoint,
        &request.keys.p256dh,
        &request.keys.auth,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(PushSubscriptionResponse {
        endpoint: sub.endpoint,
        keys: PushKeys {
            p256dh: sub.p256dh,
            auth: sub.auth,
        },
        created_at: sub.created_at.to_rfc3339(),
    }))
}

pub async fn get_subscription(
    Extension(auth): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PushSubscriptionResponse>>> {
    let subs = list_push_subscriptions(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(
        subs.into_iter()
            .map(|s| PushSubscriptionResponse {
                endpoint: s.endpoint,
                keys: PushKeys {
                    p256dh: s.p256dh,
                    auth: s.auth,
                },
                created_at: s.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

pub async fn unsubscribe(
    Extension(auth): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<StatusCode> {
    delete_push_subscriptions_for_user(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
