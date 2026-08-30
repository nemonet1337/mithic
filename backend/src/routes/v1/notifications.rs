//! Notifications

use crate::db::queries::{
    get_actors_by_ids, get_notes_with_authors_by_ids, get_notifications,
    mark_all_notifications_as_read, mark_notification_as_read,
};
use crate::{AppError, AuthUser, Result};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use serde_json::Value;
use shared::Notification as NotifDto;

use crate::dto::{actor_to_user, notes_to_dtos};
use crate::routes::v1::common::{PagingQuery, ok_null};
use crate::state::AppState;

pub async fn list_notifications(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(paging): Query<PagingQuery>,
) -> Result<Json<Vec<NotifDto>>> {
    let limit = paging.limit.unwrap_or(20).min(100);
    let notifs = get_notifications(state.surreal(), &auth.user_id, limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let sender_ids: Vec<String> = notifs
        .iter()
        .filter_map(|n| n.sender_id.map(|id| id.to_string()))
        .collect();
    let senders = get_actors_by_ids(state.surreal(), &sender_ids)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let sender_map: std::collections::HashMap<_, _> = senders
        .into_iter()
        .map(|a| (a.id.to_string(), actor_to_user(&a)))
        .collect();

    let note_ids: Vec<String> = notifs
        .iter()
        .filter_map(|n| n.note_id.map(|id| id.to_string()))
        .collect();
    let note_rows = get_notes_with_authors_by_ids(state.surreal(), &note_ids)
        .await
        .unwrap_or_default();
    let note_dtos = notes_to_dtos(&state, &note_rows, Some(&auth.user_id.to_string())).await;
    let note_map: std::collections::HashMap<_, _> = note_dtos
        .into_iter()
        .map(|dto| (dto.id.clone(), dto))
        .collect();

    let dtos = notifs
        .into_iter()
        .map(|notif| NotifDto {
            id: notif.id.to_string(),
            created_at: notif.created_at.to_rfc3339(),
            notification_type: notif.notification_type,
            sender: notif
                .sender_id
                .and_then(|id| sender_map.get(&id.to_string()).cloned()),
            note: notif
                .note_id
                .and_then(|id| note_map.get(&id.to_string()).cloned()),
            reaction: notif.reaction,
            is_read: notif.is_read,
        })
        .collect();

    Ok(Json(dtos))
}

pub async fn read_all_notifications(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Value>> {
    mark_all_notifications_as_read(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

pub async fn read_notification(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    mark_notification_as_read(state.surreal(), &id, &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}
