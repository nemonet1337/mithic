use serde::Serialize;

use super::client::{ApiError, request};
use crate::models::User;
pub use shared::{LoginRequest, TokenPair};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub handle: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

pub async fn login(body: &LoginRequest) -> Result<TokenPair, ApiError> {
    request("POST", "auth/login", None, Some(body)).await
}

pub async fn register(body: &RegisterRequest) -> Result<TokenPair, ApiError> {
    request("POST", "auth/register", None, Some(body)).await
}

pub async fn logout(token: &str) -> Result<(), ApiError> {
    request::<(), ()>("POST", "auth/logout", Some(token), None).await
}

pub async fn fetch_me(token: &str) -> Result<User, ApiError> {
    request::<User, ()>("GET", "users/me", Some(token), None).await
}
