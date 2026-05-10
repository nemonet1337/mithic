use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct FollowersQuery {
    pub page: Option<bool>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderedCollection {
    #[serde(rename = "@context")]
    pub context: String,
    pub id: String,
    #[serde(rename = "type")]
    pub collection_type: String,
    pub total_items: i32,
    pub first: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderedCollectionPage {
    #[serde(rename = "@context")]
    pub context: String,
    pub id: String,
    #[serde(rename = "type")]
    pub page_type: String,
    pub part_of: String,
    pub total_items: i32,
    pub ordered_items: Vec<String>,
    pub next: Option<String>,
    pub prev: Option<String>,
}

/// Followersエンドポイント
pub async fn followers(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<FollowersQuery>,
) -> Result<Json<serde_json::Value>> {
    let instance_url = &state.config().instance_url;
    let actor_url = format!("{}/users/{}", instance_url, username);
    let followers_url = format!("{}/followers", actor_url);

    // ユーザー存在確認
    let mut result = state
        .surreal()
        .query("SELECT id, followers_count FROM user WHERE username_lower = $username")
        .bind(("username", username.to_lowercase()))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Database(e)
        })?;

    let user_data: Option<(String, i32)> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    let (_, followers_count) = user_data
        .ok_or_else(|| AppError::NotFound(crate::t!("actor-not-found")))?;

    let page = query.page.unwrap_or(false);

    if page {
        // Pageリクエスト - 実際のフォロワー一覧を返す
        let limit = 10;
        let cursor = query.cursor.as_ref().and_then(|c| c.parse::<Ulid>().ok());

        // フォロワー取得クエリ
        let followers_query = if let Some(cursor_id) = cursor {
            r#"
                SELECT in as follower_id FROM follow 
                WHERE out = user:$user_id AND id < $cursor
                ORDER BY id DESC
                LIMIT $limit
            "#
        } else {
            r#"
                SELECT in as follower_id FROM follow 
                WHERE out = user:$user_id
                ORDER BY id DESC
                LIMIT $limit
            "#
        };

        let mut result = state
            .surreal()
            .query(followers_query)
            .bind(("user_id", username.to_lowercase()))
            .bind(("limit", limit + 1))
            .bind(("cursor", cursor.map(|c| c.to_string()).unwrap_or_default()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let follower_ids: Vec<(String,)> = result.take(0).unwrap_or_default();

        // 次ページがあるか
        let has_more = follower_ids.len() > limit;
        let follower_ids: Vec<_> = follower_ids.into_iter().take(limit).collect();

        // フォロワーのActor URLを取得
        let mut ordered_items = Vec::new();
        for (follower_username,) in follower_ids {
            let follower_url = format!("{}/users/{}", instance_url, follower_username);
            ordered_items.push(follower_url);
        }

        let next = if has_more && !ordered_items.is_empty() {
            Some(format!(
                "{}?page=true&cursor={}",
                followers_url,
                ordered_items.last().unwrap().split('/').last().unwrap()
            ))
        } else {
            None
        };

        Ok(Json(serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}?page=true", followers_url),
            "type": "OrderedCollectionPage",
            "partOf": followers_url,
            "totalItems": followers_count,
            "orderedItems": ordered_items,
            "next": next,
        })))
    } else {
        // Collectionリクエスト - メタ情報のみ
        Ok(Json(serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": followers_url,
            "type": "OrderedCollection",
            "totalItems": followers_count,
            "first": format!("{}?page=true", followers_url),
        })))
    }
}
