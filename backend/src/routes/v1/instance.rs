//! GET /api/v1/instance — メタ + カスタム絵文字一覧

use crate::db::cache;
use crate::{AppError, Result};
use axum::{extract::State, http::HeaderMap, response::Response};
use serde::Serialize;

use crate::http_cache::{CC_INSTANCE, json_with_cache};
use crate::state::AppState;

const INSTANCE_CACHE_KEY: &str = "instance:meta:v1";
const INSTANCE_CACHE_TTL: u64 = 120;

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomEmoji {
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    pub name: String,
    pub url: String,
    pub description: String,
    pub version: String,
    pub emojis: Vec<CustomEmoji>,
    /// URL-safe base64 VAPID public key when Web Push is configured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vapid_public_key: Option<String>,
}

pub async fn get_instance(State(state): State<AppState>, headers: HeaderMap) -> Result<Response> {
    if let Some(mut info) = cache::get_json::<InstanceInfo>(state.redis(), INSTANCE_CACHE_KEY).await
    {
        // VAPID is process config, not in DB — always overlay from live state
        info.vapid_public_key = state.vapid_public_key().map(|s| s.to_string());
        return Ok(json_with_cache(&headers, info, CC_INSTANCE));
    }

    let emojis = load_public_emojis(&state).await.unwrap_or_default();
    let info = InstanceInfo {
        name: state.config().instance_name.clone(),
        url: state.config().instance_url.clone(),
        description: String::new(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        emojis,
        vapid_public_key: state.vapid_public_key().map(|s| s.to_string()),
    };
    let _ = cache::set_json(state.redis(), INSTANCE_CACHE_KEY, &info, INSTANCE_CACHE_TTL).await;

    Ok(json_with_cache(&headers, info, CC_INSTANCE))
}

async fn load_public_emojis(state: &AppState) -> Result<Vec<CustomEmoji>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        name: String,
        url: String,
        category: Option<String>,
        aliases: Vec<String>,
    }

    let rows: Vec<Row> = sqlx::query_as(
        "SELECT name, url, category, aliases FROM emojis
         WHERE is_public
         ORDER BY name ASC
         LIMIT 500",
    )
    .fetch_all(state.db())
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| CustomEmoji {
            name: r.name,
            url: r.url,
            category: r.category,
            aliases: r.aliases,
        })
        .collect())
}
