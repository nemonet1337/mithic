use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{Actor, Note},
    routes::timeline::StatusResponse,
    state::{AppState, AuthUser},
};

/// 検索クエリ
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(rename = "type")]
    pub search_type: Option<String>, // accounts, statuses, hashtags
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 検索結果
#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub accounts: Vec<AccountResult>,
    pub statuses: Vec<StatusResponse>,
    pub hashtags: Vec<String>,
}

/// アカウント検索結果
#[derive(Debug, Serialize)]
pub struct AccountResult {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub is_bot: bool,
}

impl From<&Actor> for AccountResult {
    fn from(actor: &Actor) -> Self {
        Self {
            id: actor.id.to_string(),
            username: actor.username.clone(),
            display_name: actor.name.clone(),
            avatar: actor.avatar_url.clone(),
            is_bot: actor.is_bot,
        }
    }
}

/// 検索実行
pub async fn search(
    State(state): State<AppState>,
    _auth_user: axum::Extension<AuthUser>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResults>> {
    if query.q.is_empty() {
        return Err(AppError::Validation("Search query is required".to_string()));
    }

    let limit = query.limit.unwrap_or(20).min(40);
    let search_type = query.search_type.as_deref();

    let mut results = SearchResults {
        accounts: Vec::new(),
        statuses: Vec::new(),
        hashtags: Vec::new(),
    };

    // ユーザー検索
    if search_type.is_none() || search_type == Some("accounts") {
        let user_query = r#"
            SELECT * FROM user 
            WHERE username_lower CONTAINS $query 
            OR name CONTAINS $query
            ORDER BY followers_count DESC
            LIMIT $limit
        "#;
        let mut user_result = state
            .surreal()
            .query(user_query)
            .bind(("query", query.q.to_lowercase()))
            .bind(("limit", limit))
            .await
            .map_err(|e| {
                error!("Failed to search users: {}", e);
                AppError::Database(e)
            })?;

        let users: Vec<Actor> = user_result.take(0).unwrap_or_default();
        results.accounts = users.iter().map(AccountResult::from).collect();
    }

    // ノート検索（公開投稿のみ）
    if search_type.is_none() || search_type == Some("statuses") {
        let note_query = r#"
            SELECT * FROM note 
            WHERE text CONTAINS $query 
            AND visibility IN ['public', 'home']
            ORDER BY created_at DESC
            LIMIT $limit
        "#;
        let mut note_result = state
            .surreal()
            .query(note_query)
            .bind(("query", query.q.clone()))
            .bind(("limit", limit))
            .await
            .map_err(|e| {
                error!("Failed to search notes: {}", e);
                AppError::Database(e)
            })?;

        let notes: Vec<Note> = note_result.take(0).unwrap_or_default();
        results.statuses = notes
            .iter()
            .map(|n| to_status_response_search(n, &state.config().instance_url))
            .collect();
    }

    // ハッシュタグ抽出
    if search_type.is_none() || search_type == Some("hashtags") {
        let tag_query = r#"
            SELECT array::flatten(tags) as tags FROM note 
            WHERE tags CONTAINS $query
            AND visibility IN ['public', 'home']
            LIMIT $limit
        "#;
        let mut tag_result = state
            .surreal()
            .query(tag_query)
            .bind(("query", query.q.trim_start_matches('#').to_string()))
            .bind(("limit", limit))
            .await
            .ok();

        if let Some(mut result) = tag_result {
            if let Ok(tags) = result.take::<Vec<String>>(0) {
                results.hashtags = tags.into_iter().take(limit as usize).collect();
            }
        }
    }

    Ok(Json(results))
}

/// ハッシュタグ検索
pub async fn search_hashtag(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<StatusResponse>>> {
    if query.q.is_empty() {
        return Err(AppError::Validation("Hashtag is required".to_string()));
    }

    let hashtag = query.q.trim_start_matches('#').to_lowercase();
    let limit = query.limit.unwrap_or(20).min(40);

    let note_query = r#"
        SELECT * FROM note 
        WHERE tags CONTAINS $hashtag
        AND visibility IN ['public', 'home']
        ORDER BY created_at DESC
        LIMIT $limit
    "#;

    let mut result = state
        .surreal()
        .query(note_query)
        .bind(("hashtag", hashtag))
        .bind(("limit", limit))
        .await
        .map_err(|e| AppError::Database(e))?;

    let notes: Vec<Note> = result.take(0).unwrap_or_default();
    let statuses: Vec<StatusResponse> = notes
        .iter()
        .map(|n| to_status_response_search(n, &state.config().instance_url))
        .collect();

    Ok(Json(statuses))
}

/// NoteをStatusResponseに変換（検索用簡易版）
fn to_status_response_search(note: &Note, instance_url: &str) -> StatusResponse {
    use crate::models::NoteVisibility;

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
        reblog: None,
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
