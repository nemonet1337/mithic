use serde::{Deserialize, Serialize};

use crate::models::User;

use super::client::api_base;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

pub async fn login(body: &LoginRequest) -> Result<LoginResponse, reqwest::Error> {
    reqwest::Client::new()
        .post(format!("{}/v1/auth/login", api_base()))
        .json(body)
        .send()
        .await?
        .error_for_status()?
        .json::<LoginResponse>()
        .await
}
