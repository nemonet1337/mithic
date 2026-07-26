use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use mithic_core::services::auth::verify_jwt;
use mithic_core::{AppError, AuthUser};
use mithic_db::queries::get_actor_by_id;

use crate::state::AppState;

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
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let claims = verify_jwt(&token, &state.config().jwt_secret)
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let user_id = claims
        .sub
        .parse::<ulid::Ulid>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    // DB の token と突合 (signout / regenerate / password change で失効)
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
            return Err(AppError::Unauthorized(
                "Token has been revoked".to_string(),
            ));
        }
    }

    request.extensions_mut().insert(AuthUser {
        user_id,
        username: actor.username,
        is_admin: actor.is_admin,
    });

    Ok(next.run(request).await)
}

pub fn get_auth_user(request: &Request) -> Option<&AuthUser> {
    request.extensions().get::<AuthUser>()
}
