use mithic_config::AppConfig;
use mithic_db::SurrealClient;
use mithic_db::queries::{create_actor, get_actor_by_username, update_actor_token};
use shared::{SignupRequest, SigninRequest};

use crate::error::{AppError, Result};
use crate::models::actor::Actor;
use super::auth::{hash_password, verify_password, generate_jwt};

pub async fn register_user(
    surreal: &SurrealClient,
    request: SignupRequest,
) -> Result<Actor> {
    if get_actor_by_username(surreal, &request.username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .is_some()
    {
        return Err(AppError::Validation("Username is already taken".to_string()));
    }

    let password_hash = hash_password(&request.password)?;
    let mut actor = Actor::new_local(request.username, request.name);
    actor.password_hash = Some(password_hash);
    actor.email = request.email;

    let created = create_actor(surreal, &actor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(created)
}

pub async fn authenticate_user(
    surreal: &SurrealClient,
    request: SigninRequest,
    config: &AppConfig,
) -> Result<(String, Actor)> {
    let mut actor = get_actor_by_username(surreal, &request.username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    let hash = actor
        .password_hash
        .as_ref()
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    if !verify_password(&request.password, hash)? {
        return Err(AppError::Unauthorized("Invalid username or password".to_string()));
    }

    let token = generate_jwt(&actor.id.to_string(), &config.jwt_secret, config.jwt_expiry_hours)?;

    update_actor_token(surreal, &actor.id, Some(token.clone()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    actor.token = Some(token.clone());

    Ok((token, actor))
}
