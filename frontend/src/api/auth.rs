use serde::{Deserialize, Serialize};

use super::client::{ApiError, request};
use crate::models::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub handle: String,
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    pub access_token: String,
    #[allow(dead_code)]
    pub refresh_token: String,
    pub user: User,
}

pub async fn login(body: &LoginRequest) -> Result<TokenPair, ApiError> {
    request("POST", "auth/login", None, Some(body)).await
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
    request("POST", "auth/register", None, Some(body)).await
}

#[allow(dead_code)]
pub async fn logout(token: &str) -> Result<(), ApiError> {
    request::<(), ()>("POST", "auth/logout", Some(token), None).await
}

#[allow(dead_code)]
pub async fn refresh(refresh_token: &str) -> Result<TokenPair, ApiError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body<'a> {
        refresh_token: &'a str,
    }
    request(
        "POST",
        "auth/refresh",
        None,
        Some(&Body { refresh_token }),
    )
    .await
}

pub async fn fetch_me(token: &str) -> Result<User, ApiError> {
    request::<User, ()>("GET", "users/me", Some(token), None).await
}
