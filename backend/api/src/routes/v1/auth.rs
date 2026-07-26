//! Auth: register / login / refresh / logout

use axum::{Extension, Json, extract::State};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::{LoginRequest, RefreshRequest, SigninRequest, SignupRequest, TokenPair};

use mithic_core::models::actor::ActorId;
use mithic_core::services::auth::generate_jwt;
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{get_actor_by_id, update_actor_token};

use crate::dto::actor_to_user;
use crate::routes::v1::common::{normalize_handle, ok_null};
use crate::services::user::{authenticate_user, register_user};
use crate::state::AppState;

const REFRESH_EXPIRY_HOURS: i64 = 24 * 30;

#[derive(Debug, Serialize)]
struct RefreshClaims {
    sub: String,
    exp: usize,
    typ: String,
}

fn generate_refresh_token(user_id: &str, secret: &str) -> Result<String> {
    let exp = (Utc::now().timestamp() + REFRESH_EXPIRY_HOURS * 3600) as usize;
    let claims = RefreshClaims {
        sub: user_id.to_string(),
        exp,
        typ: "refresh".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate refresh token: {e}")))
}

async fn token_pair(
    state: &AppState,
    actor: &mithic_core::models::actor::Actor,
) -> Result<TokenPair> {
    let access = generate_jwt(
        &actor.id.to_string(),
        &state.config().jwt_secret,
        state.config().jwt_expiry_hours,
    )?;
    let refresh = generate_refresh_token(&actor.id.to_string(), &state.config().jwt_secret)?;
    update_actor_token(state.surreal(), &actor.id, Some(access.clone()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(TokenPair {
        access_token: access,
        refresh_token: refresh,
        user: actor_to_user(actor),
    })
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<TokenPair>> {
    let signin = SigninRequest {
        username: normalize_handle(&request.handle),
        password: request.password,
    };
    let (_token, actor) = authenticate_user(state.surreal(), signin, state.config()).await?;
    Ok(Json(token_pair(&state, &actor).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    handle: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    email: Option<String>,
    password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<TokenPair>> {
    let username = normalize_handle(&request.handle);
    if username.len() < 3
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::Validation("Invalid handle".to_string()));
    }
    if request.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let signup = SignupRequest {
        username,
        password: request.password,
        name: request.display_name,
        email: request.email,
    };
    let actor = register_user(state.surreal(), signup, &state.config().instance_url).await?;
    Ok(Json(token_pair(&state, &actor).await?))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<TokenPair>> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

    #[derive(Debug, Deserialize)]
    struct Claims {
        sub: String,
        #[allow(dead_code)]
        exp: i64,
        #[serde(default)]
        typ: String,
    }

    let data = decode::<Claims>(
        &request.refresh_token,
        &DecodingKey::from_secret(state.config().jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    if data.claims.typ != "refresh" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    let user_id = data
        .claims
        .sub
        .parse::<ActorId>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let actor = get_actor_by_id(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    Ok(Json(token_pair(&state, &actor).await?))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Value>> {
    update_actor_token(state.surreal(), &auth.user_id, None)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}
