use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::services::auth::{hash_password, verify_password};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::get_actor_by_id;
use serde::Deserialize;
use shared::User;

use crate::dto::actor_to_user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_id: Option<String>,
    pub header_id: Option<String>,
    pub is_locked: Option<bool>,
}

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<User>> {
    let user_id = auth.user_id;

    let mut update_parts = Vec::new();
    let mut bindings: Vec<(&str, serde_json::Value)> = Vec::new();

    if let Some(name) = &request.display_name {
        update_parts.push("name = $display_name");
        bindings.push(("display_name", serde_json::Value::String(name.clone())));
    }
    if let Some(bio) = &request.bio {
        update_parts.push("bio = $bio");
        bindings.push(("bio", serde_json::Value::String(bio.clone())));
    }
    if let Some(locked) = request.is_locked {
        update_parts.push("is_locked = $is_locked");
        bindings.push(("is_locked", serde_json::Value::Bool(locked)));
    }

    if update_parts.is_empty() {
        return Err(AppError::Validation("No fields to update".to_string()));
    }

    let query = format!(
        "UPDATE user SET {} WHERE id = type::record('user', $user_id);",
        update_parts.join(", ")
    );

    let mut q = state.surreal().query(&query);
    q = q.bind(("user_id", user_id.to_string()));
    for (k, v) in bindings {
        q = q.bind((k, v));
    }

    q.await.map_err(|e| AppError::Internal(e.to_string()))?;

    let actor = get_actor_by_id(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(actor_to_user(&actor)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode> {
    let user_id = auth.user_id;

    let actor = get_actor_by_id(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let hash = actor
        .password_hash
        .as_ref()
        .ok_or_else(|| AppError::Validation("No password set".to_string()))?;

    if !verify_password(&request.current_password, hash)? {
        return Err(AppError::Unauthorized(
            "Invalid current password".to_string(),
        ));
    }

    let new_hash = hash_password(&request.new_password)?;

    state
        .surreal()
        .query("UPDATE user SET password_hash = $hash WHERE id = type::record('user', $user_id);")
        .bind(("user_id", user_id.to_string()))
        .bind(("hash", new_hash))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn regenerate_token(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<String>> {
    let user_id = auth.user_id;
    let new_token = mithic_core::services::auth::generate_jwt(
        &user_id.to_string(),
        state.config().jwt_secret(),
        state.config().jwt_expiry_hours,
    )?;

    mithic_db::queries::update_actor_token(state.surreal(), &user_id, Some(new_token.clone()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(new_token))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmailRequest {
    pub email: String,
}

pub async fn update_email(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<UpdateEmailRequest>,
) -> Result<StatusCode> {
    let user_id = auth.user_id;
    let email = request.email.to_lowercase();

    state
        .surreal()
        .query("UPDATE user SET email = $email WHERE id = type::record('user', $user_id);")
        .bind(("user_id", user_id.to_string()))
        .bind(("email", email))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
