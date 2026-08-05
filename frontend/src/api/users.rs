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
    // バックエンドは GET /api/v1/users/check-handle?handle=
    request::<HandleAvailability, ()>(
        "GET",
        &format!("users/check-handle?handle={}", urlencoding_loose(username)),
        None,
        None,
    )
    .await
}

fn urlencoding_loose(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct UserRelation {
    pub id: String,
    pub is_following: bool,
    pub is_followed: bool,
    pub is_blocking: bool,
    pub is_blocked: bool,
    pub is_muted: bool,
}

#[allow(dead_code)]
pub async fn relation(token: &str, user_id: &str) -> Result<UserRelation, ApiError> {
    request::<UserRelation, ()>(
        "GET",
        &format!("users/{user_id}/relation"),
        Some(token),
        None,
    )
    .await
}

#[allow(dead_code)]
pub async fn block(token: &str, user_id: &str) -> Result<UserRelation, ApiError> {
    request::<UserRelation, ()>(
        "POST",
        &format!("users/{user_id}/block"),
        Some(token),
        None,
    )
    .await
}

#[allow(dead_code)]
pub async fn unblock(token: &str, user_id: &str) -> Result<UserRelation, ApiError> {
    request::<UserRelation, ()>(
        "DELETE",
        &format!("users/{user_id}/block"),
        Some(token),
        None,
    )
    .await
}

#[allow(dead_code)]
pub async fn mute(token: &str, user_id: &str) -> Result<UserRelation, ApiError> {
    request::<UserRelation, ()>(
        "POST",
        &format!("users/{user_id}/mute"),
        Some(token),
        None,
    )
    .await
}

#[allow(dead_code)]
pub async fn unmute(token: &str, user_id: &str) -> Result<UserRelation, ApiError> {
    request::<UserRelation, ()>(
        "DELETE",
        &format!("users/{user_id}/mute"),
        Some(token),
        None,
    )
    .await
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

pub async fn fetch_suggested(token: Option<&str>, limit: usize) -> Result<Vec<User>, ApiError> {
    request::<Vec<User>, ()>(
        "GET",
        &format!("users/suggested?limit={limit}"),
        token,
        None,
    )
    .await
}
