use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    error::{AppError, Result},
    models::{Note, NoteVisibility, UserListId},
    state::{AppState, AuthUser},
};

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub max_id: Option<String>,
    pub since_id: Option<String>,
    pub min_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub id: String,
    pub uri: String,
    pub url: Option<String>,
    pub account_id: String,
    pub content: String,
    pub created_at: String,
    pub in_reply_to_id: Option<String>,
    pub in_reply_to_account_id: Option<String>,
    pub reblog: Option<Box<StatusResponse>>,
    pub sensitive: bool,
    pub spoiler_text: String,
    pub visibility: String,
    pub replies_count: i32,
    pub reblogs_count: i32,
    pub favourites_count: i32,
    pub favourited: bool,
    pub reblogged: bool,
    pub muted: bool,
    pub bookmarked: bool,
    pub pinned: bool,
    pub media_attachments: Vec<serde_json::Value>,
    pub mentions: Vec<serde_json::Value>,
    pub tags: Vec<serde_json::Value>,
    pub emojis: Vec<serde_json::Value>,
}

/// ホームタイムライン
///
/// ブロック・ミュートしたユーザーの投稿を除外します
pub async fn home_timeline(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let limit = query.limit.unwrap_or(20).min(40);

    // フォローしているユーザーと自分の投稿を取得（ブロック・ミュート除外付き）
    // ->follow->user でフォロー先ユーザーを取得し、ブロック・ミュートユーザーを除外
    let timeline_query = r#"
        SELECT * FROM note WHERE (
            actor_id IN (
                SELECT VALUE ->follow->user FROM ONLY user:$user_id
            )
            OR actor_id = $user_id
        )
        AND visibility IN ['public', 'home', 'followers']
        AND actor_id NOT IN (SELECT VALUE out FROM block WHERE in = user:$user_id)
        AND actor_id NOT IN (SELECT VALUE out FROM mute WHERE in = user:$user_id AND (expires_at IS NONE OR expires_at > time::now()))
        ORDER BY created_at DESC
        LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(timeline_query)
        .bind(("user_id", auth_user.user_id.to_string()))
        .bind(("limit", limit))
        .await
        .map_err(|e| {
            error!("Failed to fetch home timeline: {}", e);
            AppError::Database(e)
        })?;

    let notes: Vec<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize notes: {}", e))
    })?;

    let statuses: Vec<StatusResponse> = notes
        .iter()
        .map(|note| to_status_response(note, &state.config().instance_url))
        .collect();

    Ok(Json(statuses))
}

/// リストタイムライン
///
/// ユーザーリストに含まれるユーザーの投稿を取得
pub async fn list_timeline(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(list_id): Path<String>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let list_id = list_id.parse().map_err(|_| AppError::BadRequest("Invalid list ID".to_string()))?;
    
    let limit = query.limit.unwrap_or(20).min(40) as i64;
    
    let since_id = query.since_id.as_ref().and_then(|id| {
        id.parse::<crate::models::NoteId>().ok()
    });
    
    let until_id = query.max_id.as_ref().and_then(|id| {
        id.parse::<crate::models::NoteId>().ok()
    });

    let notes = state
        .timeline_service()
        .get_list_timeline(&auth_user.user_id.into(), &list_id, limit, since_id, until_id)
        .await?;

    let statuses: Vec<StatusResponse> = notes
        .iter()
        .map(|note| to_status_response(note, &state.config().instance_url))
        .collect();

    info!("Fetched {} notes for list {} user {}", statuses.len(), list_id, auth_user.user_id);

    Ok(Json(statuses))
}

/// パブリックタイムライン
pub async fn public_timeline(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let limit = query.limit.unwrap_or(20).min(40);

    // 公開投稿のみ取得
    let timeline_query = r#"
        SELECT * FROM note
        WHERE visibility = 'public'
        ORDER BY created_at DESC
        LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(timeline_query)
        .bind(("limit", limit))
        .await
        .map_err(|e| {
            error!("Failed to fetch public timeline: {}", e);
            AppError::Database(e)
        })?;

    let notes: Vec<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize notes: {}", e))
    })?;

    let statuses: Vec<StatusResponse> = notes
        .iter()
        .map(|note| to_status_response(note, &state.config().instance_url))
        .collect();

    Ok(Json(statuses))
}

/// グローバルタイムライン（連合タイムライン）
pub async fn global_timeline(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    let limit = query.limit.unwrap_or(20).min(40);

    // 公開投稿のみ取得（ローカル・リモート問わず）
    let timeline_query = r#"
        SELECT * FROM note
        WHERE visibility = 'public'
        ORDER BY created_at DESC
        LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(timeline_query)
        .bind(("limit", limit))
        .await
        .map_err(|e| {
            error!("Failed to fetch global timeline: {}", e);
            AppError::Database(e)
        })?;

    let notes: Vec<Note> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize notes: {}", e))
    })?;

    let statuses: Vec<StatusResponse> = notes
        .iter()
        .map(|note| to_status_response(note, &state.config().instance_url))
        .collect();

    Ok(Json(statuses))
}

/// Note を StatusResponse に変換
fn to_status_response(note: &Note, instance_url: &str) -> StatusResponse {
    let visibility_str = match note.visibility {
        NoteVisibility::Public => "public",
        NoteVisibility::Home => "unlisted",
        NoteVisibility::Followers => "private",
        NoteVisibility::Specified => "direct",
    };

    StatusResponse {
        id: note.id.to_string(),
        uri: note.uri.clone().unwrap_or_else(|| {
            format!("{}/notes/{}", instance_url, note.id)
        }),
        url: note.uri.clone().or_else(|| {
            Some(format!("{}/notes/{}", instance_url, note.id))
        }),
        account_id: note.actor_id.to_string(),
        content: note.text.clone().unwrap_or_default(),
        created_at: note.created_at.to_rfc3339(),
        in_reply_to_id: note.reply_id.map(|id| id.to_string()),
        in_reply_to_account_id: note.reply_actor_id.map(|id| id.to_string()),
        reblog: note.renote_id.and_then(|renote_id| {
            // リノート先の取得は別途必要
            None
        }),
        sensitive: note.cw.is_some(),
        spoiler_text: note.cw.clone().unwrap_or_default(),
        visibility: visibility_str.to_string(),
        replies_count: note.replies_count,
        reblogs_count: note.renote_count,
        favourites_count: note.total_reactions(),
        favourited: false,
        reblogged: false,
        muted: false,
        bookmarked: false,
        pinned: false,
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: note.tags.iter().map(|tag| {
            serde_json::json!({
                "name": tag,
                "url": format!("{}/tags/{}", instance_url, tag),
            })
        }).collect(),
        emojis: Vec::new(),
    }
}
