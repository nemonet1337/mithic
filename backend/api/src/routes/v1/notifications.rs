//! Notifications

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    get_actor_by_id, get_notifications, mark_all_notifications_as_read, mark_notification_as_read,
};
use serde_json::Value;
use shared::Notification as NotifDto;

use crate::dto::{actor_to_user, notif_type_to_dto};
use crate::routes::v1::common::{ok_null, PagingQuery};
use crate::routes::v1::notes::fetch_note_dto;
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

    let mut dtos = Vec::with_capacity(notifs.len());
    for notif in notifs {
        let sender = match notif.sender_id {
            Some(sender_id) => get_actor_by_id(state.surreal(), &sender_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .map(|actor| actor_to_user(&actor)),
            None => None,
        };

        let note = match notif.note_id {
            Some(note_id) => fetch_note_dto(&state, &note_id)
                .await
                .ok()
                .map(|(_, dto)| dto),
            None => None,
        };

        dtos.push(NotifDto {
            id: notif.id.to_string(),
            created_at: notif.created_at.to_rfc3339(),
            notification_type: notif_type_to_dto(notif.notification_type),
            sender,
            note,
            reaction: notif.reaction,
            is_read: notif.is_read,
        });
    }

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
