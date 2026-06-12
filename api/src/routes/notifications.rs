use crate::dto::{actor_to_user, note_to_dto};
use crate::state::AppState;
use axum::{Extension, Json, extract::State, http::StatusCode};
use mithic_core::models::notification::NotificationType;
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    get_actor_by_id, get_note_by_id, get_notifications, mark_all_notifications_as_read,
    mark_notification_as_read,
};
use serde::Deserialize;
use shared::{Notification as NotifDto, NotificationType as NotifTypeDto};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationListRequest {
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationReadRequest {
    pub notification_id: String,
}

pub fn notif_type_to_dto(nt: NotificationType) -> NotifTypeDto {
    match nt {
        NotificationType::Mention => NotifTypeDto::Mention,
        NotificationType::Reply => NotifTypeDto::Reply,
        NotificationType::Renote => NotifTypeDto::Renote,
        NotificationType::Quote => NotifTypeDto::Quote,
        NotificationType::Reaction => NotifTypeDto::Reaction,
        NotificationType::Follow => NotifTypeDto::Follow,
        NotificationType::FollowRequest => NotifTypeDto::FollowRequest,
        NotificationType::FollowRequestAccepted => NotifTypeDto::FollowRequestAccepted,
        NotificationType::PollEnded => NotifTypeDto::PollEnded,
        NotificationType::UserSignup => NotifTypeDto::UserSignup,
    }
}

pub async fn list(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<NotificationListRequest>,
) -> Result<Json<Vec<NotifDto>>> {
    let recipient_id = auth.user_id;
    let limit = request.limit.unwrap_or(20).min(100);

    let notifs = get_notifications(state.surreal(), &recipient_id, limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut dtos = Vec::new();
    for notif in notifs {
        let sender = if let Some(sender_id) = notif.sender_id {
            get_actor_by_id(state.surreal(), &sender_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .map(|actor| actor_to_user(&actor))
        } else {
            None
        };

        let note = if let Some(note_id) = notif.note_id {
            if let Some(n) = get_note_by_id(state.surreal(), &note_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
            {
                get_actor_by_id(state.surreal(), &n.actor_id)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?
                    .map(|author_actor| note_to_dto(&n, actor_to_user(&author_actor)))
            } else {
                None
            }
        } else {
            None
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

pub async fn read(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<NotificationReadRequest>,
) -> Result<StatusCode> {
    let recipient_id = auth.user_id;

    mark_notification_as_read(state.surreal(), &request.notification_id, &recipient_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn mark_all_as_read_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<StatusCode> {
    let recipient_id = auth.user_id;

    mark_all_notifications_as_read(state.surreal(), &recipient_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
