use argon2::password_hash::phc::PasswordHash;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Builder, Secret, Totp};

use crate::error::{AppError, Result};

/// 認証済みユーザー情報
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: ulid::Ulid,
    pub username: String,
    pub is_admin: bool,
}

fn build_totp(secret: Secret) -> Result<Totp> {
    Builder::new()
        .with_algorithm(Algorithm::SHA1)
        .with_digits(6)
        .with_skew(1)
        .with_step_duration(30)
        .with_secret(secret)
        .with_issuer(Some("mithic"))
        .with_account_name("mithic")
        .build()
        .map_err(|e| AppError::Internal(format!("TOTP error: {e}")))
}

/// TOTP シークレットを生成し、(base32シークレット, otpauth URL) を返す
pub fn generate_totp_secret() -> Result<(String, String)> {
    let totp = build_totp(Secret::default())?;
    Ok((
        totp.secret().to_base32(),
        totp.to_url()
            .map_err(|e| AppError::Internal(format!("TOTP URL error: {e}")))?,
    ))
}

/// TOTP コードを検証する
pub fn verify_totp(secret: &str, code: &str) -> Result<bool> {
    let totp = build_totp(
        Secret::try_from_base32(secret).map_err(|e| AppError::Internal(e.to_string()))?,
    )?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Internal("Time went backwards".to_string()))?
        .as_secs();
    Ok(totp.check(code, now).is_some())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub typ: String,
}

pub fn hash_password(password: &str) -> Result<String> {
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes())
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Internal(format!("Failed to parse password hash: {}", e)))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_jwt(user_id: &str, jwt_secret: &str, expiry_hours: i64) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::Internal("Time went backwards".to_string()))?
        .as_secs() as usize;
    let exp = now + (expiry_hours as usize * 3600);

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        typ: "access".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate JWT: {}", e)))?;

    Ok(token)
}

pub fn verify_jwt(token: &str, jwt_secret: &str) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    validation.set_required_spec_claims(&["exp", "sub"]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    if token_data.claims.typ != "access" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    Ok(token_data.claims)
}
