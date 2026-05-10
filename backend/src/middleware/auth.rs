use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    headers::{authorization::Bearer, Authorization, HeaderMapExt},
};
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

use crate::{
    config::AppConfig,
    error::AppError,
    state::{AppState, AuthUser},
};

/// JWTクレーム
#[derive(Debug, serde::Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
    #[serde(default)]
    typ: String,
}

/// 認証ミドルウェア
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Authorizationヘッダー取得
    let token = request
        .headers()
        .typed_get::<Authorization<Bearer>>()
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?
        .token()
        .to_string();

    // JWT検証
    let claims = validate_token(&token, &state.config())
        .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    // ユーザーIDをパース
    let user_id = claims.sub.parse::<ulid::Ulid>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    // リクエストに認証情報を追加
    request.extensions_mut().insert(AuthUser {
        user_id,
        username: String::new(),
        is_admin: false,
    });

    Ok(next.run(request).await)
}

/// トークン検証
fn validate_token(token: &str, config: &AppConfig) -> anyhow::Result<Claims> {
    let decoding_key = DecodingKey::from_secret(config.jwt_secret().as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    // 有効期限チェック
    let now = Utc::now().timestamp();
    if token_data.claims.exp < now {
        return Err(anyhow::anyhow!("Token expired"));
    }

    // アクセストークンか確認
    if token_data.claims.typ != "access" {
        return Err(anyhow::anyhow!("Invalid token type"));
    }

    Ok(token_data.claims)
}

/// 認証情報を取得するヘルパー
pub fn get_auth_user(request: &Request) -> Option<&AuthUser> {
    request.extensions().get::<AuthUser>()
}
