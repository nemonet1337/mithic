//! OAuth — アプリ登録・認可・トークン発行・失効 (TODO: DB永続化)

use crate::state::AppState;
use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::models::oauth::{
    CreateOAuthAppRequest, OAuthAppResponse, OAuthTokenResponse,
};
use mithic_core::{AppError, AuthUser, Result};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeRequest {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub redirect_uri: String,
    pub grant_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeResponse {
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeRequest {
    pub client_id: String,
    pub client_secret: String,
    pub token: String,
}

/// POST /api/apps — アプリ登録
pub async fn create_app(
    State(state): State<AppState>,
    Json(request): Json<CreateOAuthAppRequest>,
) -> Result<Json<OAuthAppResponse>> {
    let _ = state;
    let id = Ulid::new();
    Ok(Json(OAuthAppResponse {
        id,
        name: request.client_name,
        website: request.website,
        scopes: request.scopes.unwrap_or_default(),
        redirect_uris: request.redirect_uris.clone(),
        client_id: hex::encode(&id.to_bytes()[..8]),
        client_secret: hex::encode(&id.to_bytes()[8..]),
    }))
}

/// POST /oauth/authorize — 認可コード発行 (要認証)
pub async fn authorize(
    Extension(_auth): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(_request): Json<AuthorizeRequest>,
) -> Result<Json<AuthorizeResponse>> {
    let _ = state;
    let code = Ulid::new().to_string();
    Ok(Json(AuthorizeResponse { code }))
}

/// POST /oauth/token — アクセストークン発行
pub async fn token(
    State(state): State<AppState>,
    Json(_request): Json<TokenRequest>,
) -> Result<Json<OAuthTokenResponse>> {
    let _ = state;
    Ok(Json(OAuthTokenResponse {
        access_token: Ulid::new().to_string(),
        token_type: "Bearer".to_string(),
        scope: "read write".to_string(),
        created_at: chrono::Utc::now().timestamp(),
    }))
}

/// GET /oauth/revoke — トークン失効
pub async fn revoke(
    State(state): State<AppState>,
    Json(_request): Json<RevokeRequest>,
) -> Result<StatusCode> {
    let _ = state;
    Ok(StatusCode::NO_CONTENT)
}
