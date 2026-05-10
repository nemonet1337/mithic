//! Hashtag API endpoints
//!
//! Provides API for trending hashtags, hashtag search, and hashtag timelines.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use tracing::{error, info};

use crate::{
    error::{AppError, Result},
    models::{Hashtag, TrendingHashtag, HashtagSearchQuery, HashtagTimelineQuery},
    state::{AppState, AuthUser},
};

/// Get trending hashtags
///
/// Returns the top 5 trending hashtags with usage statistics.
/// Trend is calculated based on unique users using the tag in the last 30 minutes.
pub async fn get_trending(
    State(state): State<AppState>,
) -> Result<Json<Vec<TrendingHashtag>>> {
    // Get notes with tags from last 30 minutes
    let range_minutes = 30;
    let since = Utc::now() - chrono::Duration::minutes(range_minutes);

    let notes: Vec<serde_json::Value> = state
        .surreal()
        .query("SELECT tags, actor_id FROM note WHERE created_at > $since AND tags != []")
        .bind(("since", since))
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    if notes.is_empty() {
        return Ok(Json(Vec::new()));
    }

    // Aggregate tag usage by unique users
    let mut tag_users: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();

    for note in notes {
        let tags: Vec<String> = note
            .get("tags")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let actor_id: String = note
            .get("actor_id")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        for tag in tags {
            tag_users
                .entry(tag)
                .or_default()
                .insert(actor_id.clone());
        }
    }

    // Sort by user count and take top 5
    let mut tag_counts: Vec<(String, usize)> = tag_users
        .into_iter()
        .map(|(tag, users)| (tag, users.len()))
        .collect();
    tag_counts.sort_by(|a, b| b.1.cmp(&a.1));
    tag_counts.truncate(5);

    // Build trending response with chart
    let mut trending = Vec::new();
    for (tag, user_count) in tag_counts {
        // Get chart data (last 20 intervals of 10 minutes each)
        let chart = generate_hashtag_chart(&state, &tag, 20, 10).await?;

        trending.push(TrendingHashtag {
            tag,
            chart,
            users_count: user_count as i32,
        });
    }

    Ok(Json(trending))
}

/// Generate usage chart for a hashtag
async fn generate_hashtag_chart(
    state: &AppState,
    tag: &str,
    range: i32,
    interval_minutes: i32,
) -> Result<Vec<i32>> {
    let now = Utc::now();
    let interval = chrono::Duration::minutes(interval_minutes as i64);
    let mut chart = Vec::new();

    for i in (0..range).rev() {
        let lt = now - interval * (i + 1);
        let gt = now - interval * i;

        let count: Option<i32> = state
            .surreal()
            .query("SELECT count() FROM note WHERE $tag INSIDE tags AND created_at > $gt AND created_at < $lt GROUP BY count()")
            .bind(("tag", tag.to_lowercase()))
            .bind(("gt", gt))
            .bind(("lt", lt))
            .await
            .and_then(|mut res| res.take(0))
            .ok()
            .flatten();

        chart.push(count.unwrap_or(0));
    }

    Ok(chart)
}

/// Search hashtags
pub async fn search_hashtags(
    State(state): State<AppState>,
    Query(query): Query<HashtagSearchQuery>,
) -> Result<Json<Vec<Hashtag>>> {
    let limit = query.limit();
    let offset = query.offset();

    let hashtags: Vec<Hashtag> = if let Some(q) = query.q {
        // Search by name
        state
            .surreal()
            .query("SELECT * FROM hashtag WHERE name CONTAINS $q ORDER BY count DESC LIMIT $limit START $offset")
            .bind(("q", q.to_lowercase()))
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default()
    } else {
        // Get all hashtags ordered by count
        state
            .surreal()
            .query("SELECT * FROM hashtag ORDER BY count DESC LIMIT $limit START $offset")
            .bind(("limit", limit))
            .bind(("offset", offset))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default()
    };

    Ok(Json(hashtags))
}

/// Get notes by hashtag
pub async fn get_hashtag_timeline(
    auth_user: Option<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<HashtagTimelineQuery>,
) -> Result<Json<Vec<crate::models::Note>>> {
    let limit = query.limit();
    let tag = query.tag.to_lowercase();

    // Build the base query
    let mut query_builder = "SELECT * FROM note WHERE $tag INSIDE tags".to_string();

    // Add pagination
    if let Some(since_id) = &query.since_id {
        query_builder.push_str(" AND id > $since_id");
    }
    if let Some(until_id) = &query.until_id {
        query_builder.push_str(" AND id < $until_id");
    }

    // Add visibility filtering
    query_builder.push_str(" AND visibility IN ['public', 'home']");

    // Add block/mute exclusion if user is authenticated
    if let Some(auth) = &auth_user {
        // Exclude blocked users
        query_builder.push_str(" AND actor_id NOT IN (SELECT out FROM block WHERE in = $user_id)");
        // Exclude muted users
        query_builder.push_str(" AND actor_id NOT IN (SELECT out FROM mute WHERE in = $user_id AND (expires_at IS NONE OR expires_at > time::now()))");
    }

    // Add file filter
    if query.with_files.unwrap_or(false) {
        query_builder.push_str(" AND file_ids != []");
    }

    query_builder.push_str(" ORDER BY created_at DESC LIMIT $limit");

    // Execute query
    let mut surreal_query = state.surreal().query(&query_builder);
    surreal_query = surreal_query.bind(("tag", tag));

    if let Some(auth) = &auth_user {
        surreal_query = surreal_query.bind(("user_id", auth.user_id.to_string()));
    }
    if let Some(since_id) = &query.since_id {
        surreal_query = surreal_query.bind(("since_id", since_id.clone()));
    }
    if let Some(until_id) = &query.until_id {
        surreal_query = surreal_query.bind(("until_id", until_id.clone()));
    }
    surreal_query = surreal_query.bind(("limit", limit));

    let notes: Vec<crate::models::Note> = surreal_query
        .await
        .and_then(|mut res| res.take(0))
        .map_err(|e| AppError::Database(e))?
        .unwrap_or_default();

    Ok(Json(notes))
}

/// Get hashtag by name
pub async fn get_hashtag(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<Hashtag>> {
    let hashtag: Option<Hashtag> = state
        .surreal()
        .query("SELECT * FROM hashtag WHERE name = $name")
        .bind(("name", name.to_lowercase()))
        .await
        .and_then(|mut res| res.take(0))
        .ok()
        .flatten();

    let hashtag = hashtag.ok_or_else(|| AppError::NotFound("Hashtag not found".to_string()))?;

    Ok(Json(hashtag))
}

/// Get users using a hashtag
pub async fn get_tag_users(
    State(state): State<AppState>,
    Path(tag): Path<String>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let limit: usize = query.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(50);

    let query_str = r#"
        SELECT DISTINCT actor_id FROM note WHERE $tag INSIDE tags ORDER BY created_at DESC LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(query_str)
        .bind(("tag", tag.to_lowercase()))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let notes: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

    let mut user_ids = Vec::new();
    for note in notes {
        if let Some(actor_id) = note.get("actor_id").and_then(|v| v.as_str()) {
            user_ids.push(actor_id.to_string());
        }
    }

    let users: Vec<serde_json::Value> = if user_ids.is_empty() {
        Vec::new()
    } else {
        let users_query = r#"
            SELECT * FROM user WHERE id IN $user_ids
        "#;
        let mut users_result = state
            .surreal()
            .query(users_query)
            .bind(("user_ids", user_ids))
            .await
            .map_err(|e| AppError::Database(e))?;

        users_result.take(0).unwrap_or_default()
    };

    Ok(Json(users))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashtag_search_query_defaults() {
        let query = HashtagSearchQuery {
            q: None,
            limit: None,
            offset: None,
        };
        assert_eq!(query.limit(), 20);
        assert_eq!(query.offset(), 0);
    }

    #[test]
    fn test_hashtag_timeline_query_defaults() {
        let query = HashtagTimelineQuery {
            tag: "test".to_string(),
            since_id: None,
            until_id: None,
            limit: None,
            with_files: None,
        };
        assert_eq!(query.limit(), 10);
    }
}
