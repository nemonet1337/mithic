use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use tracing::error;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct FollowingQuery {
    pub page: Option<bool>,
    pub cursor: Option<String>,
}

/// Followingエンドポイント
pub async fn following(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<FollowingQuery>,
) -> Result<Json<serde_json::Value>> {
    let instance_url = &state.config().instance_url;
    let actor_url = format!("{}/users/{}", instance_url, username);
    let following_url = format!("{}/following", actor_url);

    // ユーザー存在確認
    let mut result = state
        .surreal()
        .query("SELECT id, following_count FROM user WHERE username_lower = $username")
        .bind(("username", username.to_lowercase()))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Database(e)
        })?;

    let user_data: Option<(String, i32)> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize: {}", e))
    })?;

    let (_, following_count) = user_data
        .ok_or_else(|| AppError::NotFound(crate::t!("actor-not-found")))?;

    let page = query.page.unwrap_or(false);

    if page {
        // Pageリクエスト
        let limit = 10;
        let cursor = query.cursor.as_ref().and_then(|c| c.parse::<Ulid>().ok());

        let following_query = if cursor.is_some() {
            r#"
                SELECT out as followee_id FROM follow 
                WHERE in = user:$user_id AND id < $cursor
                ORDER BY id DESC
                LIMIT $limit
            "#
        } else {
            r#"
                SELECT out as followee_id FROM follow 
                WHERE in = user:$user_id
                ORDER BY id DESC
                LIMIT $limit
            "#
        };

        let mut result = state
            .surreal()
            .query(following_query)
            .bind(("user_id", username.to_lowercase()))
            .bind(("limit", limit + 1))
            .bind(("cursor", cursor.map(|c| c.to_string()).unwrap_or_default()))
            .await
            .map_err(|e| AppError::Database(e))?;

        let following_ids: Vec<(String,)> = result.take(0).unwrap_or_default();

        let has_more = following_ids.len() > limit;
        let following_ids: Vec<_> = following_ids.into_iter().take(limit).collect();

        let mut ordered_items = Vec::new();
        for (followee_username,) in following_ids {
            let followee_url = format!("{}/users/{}", instance_url, followee_username);
            ordered_items.push(followee_url);
        }

        let next = if has_more && !ordered_items.is_empty() {
            Some(format!(
                "{}?page=true&cursor={}",
                following_url,
                ordered_items.last().unwrap().split('/').last().unwrap()
            ))
        } else {
            None
        };

        Ok(Json(serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}?page=true", following_url),
            "type": "OrderedCollectionPage",
            "partOf": following_url,
            "totalItems": following_count,
            "orderedItems": ordered_items,
            "next": next,
        })))
    } else {
        // Collectionリクエスト
        Ok(Json(serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": following_url,
            "type": "OrderedCollection",
            "totalItems": following_count,
            "first": format!("{}?page=true", following_url),
        })))
    }
}
