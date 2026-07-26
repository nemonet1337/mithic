use serde::{Deserialize, Serialize};

use super::client::{ApiError, request};
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
    let body = serde_json::json!({ "username": username });
    request::<HandleAvailability, serde_json::Value>(
        "POST",
        "users/check-handle",
        None,
        Some(&body),
    )
    .await
}

pub async fn follow(token: &str, user_id: &str) -> Result<(), ApiError> {
    let body = serde_json::json!({ "user_id": user_id });
    request::<(), serde_json::Value>(
        "POST",
        "follows",
        Some(token),
        Some(&body),
    )
    .await
}

pub async fn unfollow(token: &str, user_id: &str) -> Result<(), ApiError> {
    request::<(), ()>(
        "DELETE",
        &format!("follows/{}", user_id),
        Some(token),
        None,
    )
    .await
}

#[derive(Debug, Serialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

pub async fn update_me(token: &str, body: &UpdateProfileRequest) -> Result<User, ApiError> {
    request("PATCH", "users/me", Some(token), Some(body)).await
}
