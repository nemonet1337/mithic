//! GET /api/v1/instance — メタ + カスタム絵文字一覧

use axum::{
    extract::State,
    http::HeaderMap,
    response::Response,
};
use mithic_core::{AppError, Result};
use mithic_db::cache;
use mithic_db::queries::rows_to;
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
}

pub async fn get_instance(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response> {
    if let Some(info) = cache::get_json::<InstanceInfo>(state.dragonfly(), INSTANCE_CACHE_KEY).await
    {
        return Ok(json_with_cache(&headers, info, CC_INSTANCE));
    }

    let emojis = load_public_emojis(&state).await.unwrap_or_default();
    let info = InstanceInfo {
        name: state.config().instance_name.clone(),
        url: state.config().instance_url.clone(),
        description: String::new(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        emojis,
    };
    let _ = cache::set_json(state.dragonfly(), INSTANCE_CACHE_KEY, &info, INSTANCE_CACHE_TTL)
        .await;

    Ok(json_with_cache(&headers, info, CC_INSTANCE))
}

async fn load_public_emojis(state: &AppState) -> Result<Vec<CustomEmoji>> {
    let mut response = state
        .surreal()
        .query(
            "SELECT name, url, category, aliases FROM emoji WHERE is_public = true ORDER BY name ASC LIMIT 500;",
        )
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows: Vec<surrealdb::types::Value> = response
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    #[derive(serde::Deserialize)]
    struct Row {
        name: String,
        url: String,
        #[serde(default)]
        category: Option<String>,
        #[serde(default)]
        aliases: Vec<String>,
    }

    let parsed: Vec<Row> = rows_to(rows).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(parsed
        .into_iter()
        .map(|r| CustomEmoji {
            name: r.name,
            url: r.url,
            category: r.category,
            aliases: r.aliases,
        })
        .collect())
}
