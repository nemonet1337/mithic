use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use ulid::Ulid;

use crate::{
    error::{AppError, Result},
    models::{Notification, NotificationType},
    state::{AppState, AuthUser},
};

/// 通知一覧クエリ
#[derive(Debug, Deserialize)]
pub struct NotificationsQuery {
    pub limit: Option<i64>,
    pub since_id: Option<String>,
    pub until_id: Option<String>,
    pub unread_only: Option<bool>,
    pub following: Option<bool>,
    #[serde(default)]
    pub mark_as_read: bool,
    #[serde(default)]
    pub include_types: Vec<String>,
    #[serde(default)]
    pub exclude_types: Vec<String>,
}

/// 通知レスポンス
#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: String,
    pub notification_type: String,
    pub created_at: String,
    pub sender_id: Option<String>,
    pub note_id: Option<String>,
    pub reaction: Option<String>,
    pub is_read: bool,
}

impl From<&Notification> for NotificationResponse {
    fn from(n: &Notification) -> Self {
        Self {
            id: n.id.to_string(),
            notification_type: format!("{:?}", n.notification_type).to_lowercase(),
            created_at: n.created_at.to_rfc3339(),
            sender_id: n.sender_id.map(|id| id.to_string()),
            note_id: n.note_id.map(|id| id.to_string()),
            reaction: n.reaction.clone(),
            is_read: n.is_read,
        }
    }
}

/// 通知一覧取得
pub async fn get_notifications(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    Query(query): Query<NotificationsQuery>,
) -> Result<Json<Vec<NotificationResponse>>> {
    let limit = query.limit.unwrap_or(20).min(100);

    let mut sql = String::from(
        "SELECT * FROM notification WHERE recipient_id = $recipient_id"
    );

    if query.unread_only.unwrap_or(false) {
        sql.push_str(" AND is_read = false");
    }

    if let Some(since_id) = &query.since_id {
        sql.push_str(&format!(" AND id > '{}'", since_id));
    }

    if let Some(until_id) = &query.until_id {
        sql.push_str(&format!(" AND id < '{}'", until_id));
    }

    // followingフィルタ（フォロー中のユーザーからの通知のみ）
    if query.following.unwrap_or(false) {
        sql.push_str(" AND sender_id IN (SELECT out FROM follow WHERE in = $recipient_id)");
    }

    // 通知タイプフィルタ
    if !query.include_types.is_empty() {
        let types: Vec<String> = query.include_types.iter().map(|t| format!("'{}'", t)).collect();
        sql.push_str(&format!(" AND notification_type IN ({})", types.join(", ")));
    } else if !query.exclude_types.is_empty() {
        let types: Vec<String> = query.exclude_types.iter().map(|t| format!("'{}'", t)).collect();
        sql.push_str(&format!(" AND notification_type NOT IN ({})", types.join(", ")));
    }

    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit));

    let mut result = state
        .surreal()
        .query(&sql)
        .bind(("recipient_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| {
            error!("Failed to fetch notifications: {}", e);
            AppError::Database(e)
        })?;

    let notifications: Vec<Notification> = result.take(0).unwrap_or_default();

    // markAsReadがtrueの場合、取得した通知を既読にする
    if query.mark_as_read && !notifications.is_empty() {
        let notification_ids: Vec<String> = notifications.iter().map(|n| n.id.to_string()).collect();
        for id in notification_ids {
            state.surreal()
                .query("UPDATE notification:$id SET is_read = true")
                .bind(("id", id))
                .await
                .ok();
        }
    }

    let responses: Vec<NotificationResponse> = notifications.iter().map(NotificationResponse::from).collect();

    Ok(Json(responses))
}

/// 未読通知数取得
pub async fn get_unread_count(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<serde_json::Value>> {
    let query = r#"
        SELECT count() FROM notification 
        WHERE recipient_id = $recipient_id AND is_read = false
    "#;

    let mut result = state
        .surreal()
        .query(query)
        .bind(("recipient_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let count: Option<i64> = result.take(0).ok().flatten();

    Ok(Json(serde_json::json!({
        "count": count.unwrap_or(0)
    })))
}

/// 通知を既読にする
pub async fn mark_as_read(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
) -> Result<Json<serde_json::Value>> {
    let query = r#"
        UPDATE notification SET is_read = true 
        WHERE recipient_id = $recipient_id AND is_read = false
    "#;

    state
        .surreal()
        .query(query)
        .bind(("recipient_id", auth_user.user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(Json(serde_json::json!({"success": true})))
}

/// 特定の通知を既読にする
pub async fn mark_one_as_read(
    State(state): State<AppState>,
    auth_user: axum::Extension<AuthUser>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>> {
    let notification_id = id.parse::<Ulid>()
        .map_err(|_| AppError::Validation("Invalid notification ID".to_string()))?;

    // 所有者確認
    let check_query = r#"
        SELECT recipient_id FROM notification WHERE id = $id
    "#;
    let mut check_result = state
        .surreal()
        .query(check_query)
        .bind(("id", notification_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;

    let recipient_id: Option<String> = check_result.take(0).ok().flatten();

    match recipient_id {
        Some(rid) if rid == auth_user.user_id.to_string() => {
            let update_query = r#"
                UPDATE notification:$id SET is_read = true
            "#;
            state
                .surreal()
                .query(update_query)
                .bind(("id", notification_id.to_string()))
                .await
                .map_err(|e| AppError::Database(e))?;

            Ok(Json(serde_json::json!({"success": true})))
        }
        _ => Err(AppError::NotFound("Notification not found".to_string())),
    }
}
