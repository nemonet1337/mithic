use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use mithic_core::{AppError, AuthUser};

use crate::state::AppState;

#[derive(Debug, serde::Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
    #[serde(default)]
    typ: String,
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
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let claims = validate_token(&token, state.config().jwt_secret())
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let user_id = claims
        .sub
        .parse::<ulid::Ulid>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    request.extensions_mut().insert(AuthUser {
        user_id,
        username: String::new(),
        is_admin: false,
    });

    Ok(next.run(request).await)
}

fn validate_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    let now = Utc::now().timestamp();
    if token_data.claims.exp < now {
        return Err(anyhow::anyhow!("Token expired"));
    }
    if token_data.claims.typ != "access" {
        return Err(anyhow::anyhow!("Invalid token type"));
    }

    Ok(token_data.claims)
}

pub fn get_auth_user(request: &Request) -> Option<&AuthUser> {
    request.extensions().get::<AuthUser>()
}
