use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use surrealdb::opt::PatchOp;
use tracing::error;

use crate::{
    config::AppConfig,
    error::{AppError, Result},
    models::Actor,
    state::AppState,
};

/// サインインリクエスト
#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub username: String,
    pub password: String,
}

/// サインアップリクエスト
#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// 認証レスポンス
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub actor: ActorResponse,
}

#[derive(Debug, Serialize)]
pub struct ActorResponse {
    pub id: String,
    pub username: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

/// JWTクレーム
#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
    typ: String,
}

/// サインインハンドラー
pub async fn signin(
    State(state): State<AppState>,
    Json(req): Json<SigninRequest>,
) -> Result<Json<AuthResponse>> {
    let username_lower = req.username.to_lowercase();

    // SurrealDBからユーザー検索
    let mut result = state
        .surreal()
        .query("SELECT * FROM user WHERE username_lower = $username")
        .bind(("username", username_lower))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Internal(format!("Database error: {}", e))
        })?;

    let actor: Option<Actor> = result.take(0).map_err(|e| {
        error!("Failed to deserialize actor: {}", e);
        AppError::Internal("Failed to deserialize actor".to_string())
    })?;

    let actor = actor.ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // パスワード検証
    let password_hash = actor.password_hash.as_ref()
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify_password(&req.password, password_hash)? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // JWT生成
    let token = generate_token(&actor.id.to_string(), &state.config())?;

    Ok(Json(AuthResponse {
        token,
        actor: ActorResponse {
            id: actor.id.to_string(),
            username: actor.username,
            name: actor.name,
            avatar_url: actor.avatar_url,
        },
    }))
}

/// パスワードをハッシュ化
fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            error!("Password hashing error: {}", e);
            AppError::Internal(format!("Password hashing error: {}", e))
        })?
        .to_string();
    Ok(password_hash)
}

/// パスワードを検証
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        error!("Password hash parsing error: {}", e);
        AppError::Internal(format!("Password hash parsing error: {}", e))
    })?;

    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// サインアップハンドラー
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<AuthResponse>> {
    let username_lower = req.username.to_lowercase();

    // ユーザー名重複チェック
    let mut result = state
        .surreal()
        .query("SELECT id FROM user WHERE username_lower = $username")
        .bind(("username", username_lower.clone()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let existing: Option<String> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Database error: {}", e))
    })?;

    if existing.is_some() {
        return Err(AppError::Validation("Username already taken".to_string()));
    }

    // パスワードをハッシュ化
    let password_hash = hash_password(&req.password)?;

    // アクターを作成
    let mut actor = Actor::new_local(req.username, req.name);
    actor.password_hash = Some(password_hash);
    actor.email = req.email;

    // SurrealDBに保存
    let created: Actor = state
        .surreal()
        .create(("user", actor.id.to_string()))
        .content(actor)
        .await
        .map_err(|e| {
            error!("Failed to create actor: {}", e);
            AppError::Database(e)
        })?;

    // JWT生成
    let token = generate_token(&created.id.to_string(), &state.config())?;

    Ok(Json(AuthResponse {
        token,
        actor: ActorResponse {
            id: created.id.to_string(),
            username: created.username,
            name: created.name,
            avatar_url: created.avatar_url,
        },
    }))
}

/// JWTトークン生成
fn generate_token(user_id: &str, config: &AppConfig) -> Result<String> {
    let now = Utc::now();
    let exp = now + Duration::hours(config.jwt_expiry_hours);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        typ: "access".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret().as_bytes()),
    )
    .map_err(|e| {
        error!("JWT encoding error: {}", e);
        AppError::Internal(format!("JWT encoding error: {}", e))
    })?;

    Ok(token)
}
