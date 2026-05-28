use axum::{Extension, Json, extract::State};
use mithic_core::services::auth::generate_jwt;
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{get_actor_by_id, update_actor_token};
use shared::{MeResponse, SigninRequest, SigninResponse, SignupRequest};

use crate::dto::actor_to_user;
use crate::services::user::{authenticate_user, register_user};
use crate::state::AppState;

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<Json<SigninResponse>> {
    let actor = register_user(state.surreal(), request).await?;

    let token = generate_jwt(
        &actor.id.to_string(),
        state.config().jwt_secret(),
        state.config().jwt_expiry_hours,
    )?;

    update_actor_token(state.surreal(), &actor.id, Some(token.clone()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = actor_to_user(&actor);
    Ok(Json(SigninResponse { token, user }))
}

pub async fn signin(
    State(state): State<AppState>,
    Json(request): Json<SigninRequest>,
) -> Result<Json<SigninResponse>> {
    let (token, actor) = authenticate_user(state.surreal(), request, state.config()).await?;
    let user = actor_to_user(&actor);
    Ok(Json(SigninResponse { token, user }))
}

pub async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<MeResponse>> {
    let actor = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let user = actor_to_user(&actor);
    Ok(Json(MeResponse { user }))
}
