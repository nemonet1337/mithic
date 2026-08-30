use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use mithic_core::services::auth::verify_jwt;
use mithic_core::{AppError, AuthUser};
use mithic_db::queries::get_actor_by_id;

use crate::state::AppState;

/// Bearer トークンを検証し AuthUser を返す (middleware / streaming 共通)
pub async fn resolve_bearer(state: &AppState, token: &str) -> Result<AuthUser, AppError> {
    let claims = verify_jwt(token, &state.config().jwt_secret)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let user_id = claims
        .sub
        .parse::<ulid::Ulid>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let actor = get_actor_by_id(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    if actor.is_suspended {
        return Err(AppError::Forbidden("Account is suspended".to_string()));
    }

    match actor.token.as_deref() {
        Some(stored) if stored == token => {}
        _ => {
            return Err(AppError::Unauthorized("Token has been revoked".to_string()));
        }
    }

    Ok(AuthUser {
        user_id,
        username: actor.username,
        is_admin: actor.is_admin,
    })
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let auth = resolve_bearer(&state, token).await?;
    request.extensions_mut().insert(auth);
    Ok(next.run(request).await)
}

/// Bearer があれば AuthUser を載せる。不正トークンは無視して匿名のまま。
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(token) = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        && let Ok(auth) = resolve_bearer(&state, token).await
    {
        request.extensions_mut().insert(auth);
    }
    next.run(request).await
}
