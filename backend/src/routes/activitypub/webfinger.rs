use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    error::{AppError, Result},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct WebfingerQuery {
    pub resource: String,
}

#[derive(Debug, Serialize)]
pub struct WebfingerResponse {
    pub subject: String,
    pub aliases: Vec<String>,
    pub links: Vec<WebfingerLink>,
}

#[derive(Debug, Serialize)]
pub struct WebfingerLink {
    pub rel: String,
    #[serde(rename = "type")]
    pub link_type: Option<String>,
    pub href: Option<String>,
    pub template: Option<String>,
}

/// Webfingerエンドポイント
pub async fn webfinger(
    State(state): State<AppState>,
    Query(query): Query<WebfingerQuery>,
) -> Result<Json<WebfingerResponse>> {
    // acct:user@example.com 形式をパース
    let resource = query.resource;
    if !resource.starts_with("acct:") {
        return Err(AppError::Validation("Invalid resource format".to_string()));
    }

    let acct = resource.trim_start_matches("acct:");
    let parts: Vec<&str> = acct.split('@').collect();
    if parts.len() != 2 {
        return Err(AppError::Validation("Invalid acct format".to_string()));
    }

    let username = parts[0];
    let host = parts[1];

    // インスタンスドメイン確認
    let instance_url = &state.config().instance_url;
    let instance_host = instance_url
        .trim_start_matches("http://")
        .trim_start_matches("https://");

    if host != instance_host {
        return Err(AppError::NotFound("Actor not found on this instance".to_string()));
    }

    // ユーザー存在確認
    let mut result = state
        .surreal()
        .query("SELECT id FROM user WHERE username_lower = $username")
        .bind(("username", username.to_lowercase()))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Database(e)
        })?;

    let actor_id: Option<String> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    if actor_id.is_none() {
        return Err(AppError::NotFound("Actor not found".to_string()));
    }

    let actor_url = format!("{}/users/{}", instance_url, username);

    Ok(Json(WebfingerResponse {
        subject: format!("acct:{}@{}", username, host),
        aliases: vec![actor_url.clone()],
        links: vec![
            WebfingerLink {
                rel: "self".to_string(),
                link_type: Some("application/activity+json".to_string()),
                href: Some(actor_url.clone()),
                template: None,
            },
            WebfingerLink {
                rel: "http://webfinger.net/rel/profile-page".to_string(),
                link_type: Some("text/html".to_string()),
                href: Some(actor_url),
                template: None,
            },
        ],
    }))
}
