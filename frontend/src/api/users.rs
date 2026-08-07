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

pub async fn follow(token: &str, user_id: &str) -> Result<(), ApiError> {
    request::<serde_json::Value, ()>(
        "POST",
        &format!("users/{}/follow", user_id),
        Some(token),
        None,
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

#[derive(Debug, Serialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
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
