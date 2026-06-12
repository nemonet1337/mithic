use serde::{Deserialize, Serialize};

use super::client::{ApiError, request};
use crate::models::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub handle: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

pub async fn login(body: &LoginRequest) -> Result<TokenPair, ApiError> {
    request("POST", "v1/auth/login", None, Some(body)).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub handle: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

pub async fn register(body: &RegisterRequest) -> Result<TokenPair, ApiError> {
    request("POST", "v1/auth/register", None, Some(body)).await
}

pub async fn logout(token: &str) -> Result<(), ApiError> {
    request::<(), ()>("POST", "v1/auth/logout", Some(token), None).await
}

pub async fn refresh(refresh_token: &str) -> Result<TokenPair, ApiError> {
    #[derive(Serialize)]
    struct Body<'a> {
        refresh_token: &'a str,
    }
    request(
        "POST",
        "v1/auth/refresh",
        None,
        Some(&Body { refresh_token }),
    )
    .await
}

pub async fn fetch_me(token: &str) -> Result<User, ApiError> {
    request::<User, ()>("GET", "v1/users/me", Some(token), None).await
}
