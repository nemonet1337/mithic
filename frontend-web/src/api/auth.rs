use serde::{Deserialize, Serialize};

use super::client::{ApiError, request};
use crate::models::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub remember: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub user: User,
}

pub async fn login(body: &LoginRequest) -> Result<SigninResponse, ApiError> {
    request("POST", "auth/signin", None, Some(body)).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

pub async fn register(body: &RegisterRequest) -> Result<TokenPair, ApiError> {
    request("POST", "auth/signup", None, Some(body)).await
}

pub async fn logout(token: &str) -> Result<(), ApiError> {
    request::<(), ()>("POST", "auth/signout", Some(token), None).await
}

pub async fn refresh(refresh_token: &str) -> Result<TokenPair, ApiError> {
    #[derive(Serialize)]
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
    request::<User, ()>("GET", "auth/me", Some(token), None).await
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
}

pub async fn setup_2fa(token: &str) -> Result<TwoFactorSetupResponse, ApiError> {
    request::<TwoFactorSetupResponse, ()>("POST", "2fa/setup", Some(token), None).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorActivateRequest {
    pub code: String,
}

pub async fn activate_2fa(token: &str, code: &str) -> Result<(), ApiError> {
    request::<(), _>(
        "POST",
        "2fa/activate",
        Some(token),
        Some(&TwoFactorActivateRequest {
            code: code.to_string(),
        }),
    )
    .await
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigninResponse {
    pub token: String,
    pub user: User,
    #[serde(default)]
    pub requires_2fa: Option<bool>,
    #[serde(default)]
    pub temp_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorVerifyRequest {
    pub username: String,
    pub temp_token: String,
    pub code: String,
}

pub async fn verify_2fa_signin(
    username: &str,
    temp_token: &str,
    code: &str,
) -> Result<SigninResponse, ApiError> {
    request::<SigninResponse, _>(
        "POST",
        "2fa/verify",
        None,
        Some(&TwoFactorVerifyRequest {
            username: username.to_string(),
            temp_token: temp_token.to_string(),
            code: code.to_string(),
        }),
    )
    .await
}
