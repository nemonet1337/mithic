use serde::{Deserialize, Serialize};

use super::client::{ApiError, request, urlencoding_loose};
use crate::models::{Note, User};

#[derive(Debug, Deserialize)]
pub struct HandleAvailability {
    pub available: bool,
}

pub async fn fetch_user(token: &str, username: &str) -> Result<User, ApiError> {
    request::<User, ()>("GET", &format!("users/{}", username), Some(token), None).await
}

pub async fn fetch_user_notes(token: &str, username: &str) -> Result<Vec<Note>, ApiError> {
    request::<Vec<Note>, ()>(
        "GET",
        &format!("users/{}/notes", username),
        Some(token),
        None,
    )
    .await
}

pub async fn check_handle(username: &str) -> Result<HandleAvailability, ApiError> {
    request::<HandleAvailability, ()>(
        "GET",
        &format!("users/check-handle?handle={}", urlencoding_loose(username)),
        None,
        None,
    )
    .await
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FollowResponse {
    #[serde(default)]
    pub followed_message: Option<String>,
}

pub async fn follow(token: &str, user_id: &str) -> Result<FollowResponse, ApiError> {
    request::<FollowResponse, ()>(
        "POST",
        &format!("users/{}/follow", user_id),
        Some(token),
        None,
    )
    .await
}

pub async fn block(token: &str, user_id: &str) -> Result<(), ApiError> {
    request::<serde_json::Value, ()>("POST", &format!("users/{user_id}/block"), Some(token), None)
        .await
        .map(|_| ())
}

pub async fn mute(token: &str, user_id: &str) -> Result<(), ApiError> {
    request::<serde_json::Value, ()>("POST", &format!("users/{user_id}/mute"), Some(token), None)
        .await
        .map(|_| ())
}

pub async fn list_blocks(token: &str) -> Result<Vec<User>, ApiError> {
    request::<Vec<User>, ()>("GET", "blocks", Some(token), None).await
}

pub async fn list_mutes(token: &str) -> Result<Vec<User>, ApiError> {
    request::<Vec<User>, ()>("GET", "mutes", Some(token), None).await
}

pub async fn change_password(
    token: &str,
    current_password: &str,
    new_password: &str,
) -> Result<(), ApiError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body<'a> {
        current_password: &'a str,
        new_password: &'a str,
    }
    request::<serde_json::Value, Body>(
        "POST",
        "users/me/password",
        Some(token),
        Some(&Body {
            current_password,
            new_password,
        }),
    )
    .await
    .map(|_| ())
}

pub async fn unfollow(token: &str, user_id: &str) -> Result<(), ApiError> {
    request::<serde_json::Value, ()>(
        "DELETE",
        &format!("users/{}/follow", user_id),
        Some(token),
        None,
    )
    .await
    .map(|_| ())
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub is_locked: Option<bool>,
    pub is_bot: Option<bool>,
    pub is_cat: Option<bool>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub location: Option<String>,
    pub birthday: Option<String>,
    pub lang: Option<String>,
    pub fields: Option<Vec<shared::ProfileField>>,
    pub followed_message: Option<String>,
    pub reaction_acceptance: Option<String>,
}

pub async fn update_me(token: &str, body: &UpdateProfileRequest) -> Result<User, ApiError> {
    request("PATCH", "users/me", Some(token), Some(body)).await
}

pub async fn search_users(token: Option<&str>, q: &str) -> Result<Vec<User>, ApiError> {
    request::<Vec<User>, ()>(
        "GET",
        &format!("users/search?q={}", urlencoding_loose(q)),
        token,
        None,
    )
    .await
}
