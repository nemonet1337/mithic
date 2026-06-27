use axum::{Extension, Json, extract::State, http::StatusCode, routing::post};
use mithic_core::auth::AuthUser;
use mithic_core::{AppError, Result};
use serde::Deserialize;

use crate::dto::actor_to_user;
use crate::services::user::{authenticate_user, register_user};
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorEnableRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorVerifyRequest {
    pub username: String,
    pub temp_token: String,
    pub code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct TwoFactorSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
}

pub fn router(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/signup", post(signup))
        .route("/signin", post(signin))
        .route("/signout", post(signout))
        .route("/refresh", post(refresh))
        .route("/2fa/setup", post(setup_2fa))
        .route("/2fa/activate", post(activate_2fa))
        .route("/2fa/verify", post(verify_2fa_signin))
        .with_state(state)
}

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<shared::SignupRequest>,
) -> Result<Json<shared::SigninResponse>> {
    let actor = register_user(state.surreal(), request, &state.config().instance_url).await?;

    let token = mithic_core::services::auth::generate_jwt(
        &actor.id.to_string(),
        state.config().jwt_secret(),
        state.config().jwt_expiry_hours,
    )?;

    mithic_db::queries::update_actor_token(state.surreal(), &actor.id, Some(token.clone()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = actor_to_user(&actor);
    Ok(Json(shared::SigninResponse {
        token,
        user,
        requires_2fa: None,
        temp_token: None,
    }))
}

pub async fn signin(
    State(state): State<AppState>,
    Json(request): Json<shared::SigninRequest>,
) -> Result<Json<shared::SigninResponse>> {
    let (_, actor) = authenticate_user(state.surreal(), request, state.config()).await?;
    let user = actor_to_user(&actor);

    if actor.totp_verified && actor.totp_secret.is_some() {
        let temp_token = mithic_core::services::auth::generate_jwt(
            &actor.id.to_string(),
            state.config().jwt_secret(),
            5i64 / 60, // 5 minutes expiry for temp token
        )?;
        return Ok(Json(shared::SigninResponse {
            token: String::new(),
            user,
            requires_2fa: Some(true),
            temp_token: Some(temp_token),
        }));
    }

    let token = mithic_core::services::auth::generate_jwt(
        &actor.id.to_string(),
        state.config().jwt_secret(),
        state.config().jwt_expiry_hours,
    )?;

    mithic_db::queries::update_actor_token(state.surreal(), &actor.id, Some(token.clone()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(shared::SigninResponse {
        token,
        user,
        requires_2fa: None,
        temp_token: None,
    }))
}

pub async fn signout(State(_state): State<AppState>) -> Result<StatusCode> {
    Ok(StatusCode::NO_CONTENT)
}

pub async fn refresh(State(_state): State<AppState>) -> Result<Json<TokenResponse>> {
    Ok(Json(TokenResponse {
        access_token: String::new(),
        token_type: "Bearer".to_string(),
    }))
}

/// 2FA セットアップ: シークレット生成と QR URL を返す (要認証)
pub async fn setup_2fa(
    Extension(auth): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<Json<TwoFactorSetupResponse>> {
    let (secret, url) = mithic_core::services::auth::generate_totp_secret()?;

    mithic_db::queries::enable_totp(state.surreal(), &auth.user_id, &secret)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(TwoFactorSetupResponse {
        secret,
        qr_code_url: url,
    }))
}

/// 2FA 有効化: TOTPコードを検証し、2FA を有効にする (要認証)
pub async fn activate_2fa(
    Extension(auth): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(request): Json<TwoFactorEnableRequest>,
) -> Result<StatusCode> {
    use mithic_db::queries::get_actor_by_id;

    let actor = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let secret = actor
        .totp_secret
        .as_ref()
        .ok_or_else(|| AppError::Validation("2FA not set up yet".to_string()))?;

    if !mithic_core::services::auth::verify_totp(secret, &request.code)? {
        return Err(AppError::Validation("Invalid TOTP code".to_string()));
    }

    mithic_db::queries::enable_totp(state.surreal(), &auth.user_id, secret)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// サインイン時の2FA検証: 一時トークン + TOTPコード → 本JWT発行
pub async fn verify_2fa_signin(
    State(state): State<AppState>,
    Json(request): Json<TwoFactorVerifyRequest>,
) -> Result<Json<shared::SigninResponse>> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    use mithic_core::models::actor::ActorId;

    #[derive(serde::Deserialize)]
    struct TempClaims {
        sub: String,
        exp: i64,
        #[serde(default)]
        typ: String,
    }

    let claims = decode::<TempClaims>(
        &request.temp_token,
        &DecodingKey::from_secret(state.config().jwt_secret().as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired temp token".to_string()))?
    .claims;

    let actor_id: ActorId = claims
        .sub
        .parse()
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let actor = mithic_db::queries::get_actor_by_id(state.surreal(), &actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let secret = actor
        .totp_secret
        .as_ref()
        .ok_or_else(|| AppError::Validation("2FA not enabled".to_string()))?;

    if !mithic_core::services::auth::verify_totp(secret, &request.code)? {
        return Err(AppError::Unauthorized("Invalid TOTP code".to_string()));
    }

    let token = mithic_core::services::auth::generate_jwt(
        &actor.id.to_string(),
        state.config().jwt_secret(),
        state.config().jwt_expiry_hours,
    )?;

    mithic_db::queries::update_actor_token(state.surreal(), &actor.id, Some(token.clone()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = actor_to_user(&actor);
    Ok(Json(shared::SigninResponse {
        token,
        user,
        requires_2fa: None,
        temp_token: None,
    }))
}