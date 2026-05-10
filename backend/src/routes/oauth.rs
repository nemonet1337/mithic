//! OAuth API endpoints
//!
//! OAuth 2.0 application registration and token management.

use axum::{
    extract::State,
    Json,
};
use tracing::info;

use crate::{
    error::{AppError, Result},
    models::{CreateOAuthAppRequest, OAuthAppResponse, OAuthTokenResponse, Ulid},
    state::{AppState, AuthUser},
};

/// Register OAuth application
pub async fn create_app(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateOAuthAppRequest>,
) -> Result<Json<OAuthAppResponse>> {
    let app = state
        .oauth_service()
        .create_app(req, Some(auth_user.user_id.into()))
        .await?;

    info!("OAuth app {} created by user {}", app.id, auth_user.user_id);

    Ok(Json(OAuthAppResponse::from(app)))
}

/// Verify OAuth application credentials
pub async fn verify_app_credentials(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<OAuthAppResponse>> {
    let client_id = req.get("client_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing client_id".to_string()))?;

    let client_secret = req.get("client_secret")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing client_secret".to_string()))?;

    let app = state
        .oauth_service()
        .get_app_by_client_id(client_id)
        .await?
        .ok_or_else(|| AppError::NotFound("App not found".to_string()))?;

    if app.client_secret != client_secret {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    Ok(Json(OAuthAppResponse::from(app)))
}

/// Create OAuth token (simplified authorization code flow)
pub async fn create_token(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<OAuthTokenResponse>> {
    let client_id = req.get("client_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing client_id".to_string()))?;

    let client_secret = req.get("client_secret")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing client_secret".to_string()))?;

    let scopes = req.get("scope")
        .and_then(|v| v.as_str())
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_else(|| vec!["read".to_string(), "write".to_string()]);

    let app = state
        .oauth_service()
        .get_app_by_client_id(client_id)
        .await?
        .ok_or_else(|| AppError::NotFound("App not found".to_string()))?;

    if app.client_secret != client_secret {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = state
        .oauth_service()
        .create_token(app.id, auth_user.user_id.into(), scopes)
        .await?;

    info!("OAuth token created for user {} app {}", auth_user.user_id, app.id);

    Ok(Json(OAuthTokenResponse {
        access_token: token.access_token,
        token_type: "Bearer".to_string(),
        scope: token.scopes.join(" "),
        created_at: token.created_at.timestamp(),
    }))
}
