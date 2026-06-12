use crate::SurrealClient;
use crate::queries::rows_to;
use mithic_core::models::actor::ActorId;
use mithic_core::models::notification::{Notification, NotificationType};

fn notif_type_str(nt: NotificationType) -> &'static str {
    match nt {
        NotificationType::Mention => "mention",
        NotificationType::Reply => "reply",
        NotificationType::Renote => "renote",
        NotificationType::Quote => "quote",
        NotificationType::Reaction => "reaction",
        NotificationType::Follow => "follow",
        NotificationType::FollowRequest => "follow_request",
        NotificationType::FollowRequestAccepted => "follow_request_accepted",
        NotificationType::PollEnded => "poll_ended",
        NotificationType::UserSignup => "user_signup",
    }
}

pub async fn create_notification(
    client: &SurrealClient,
    notif: &Notification,
) -> anyhow::Result<()> {
    let id_str = notif.id.to_string();
    let recipient_str = notif.recipient_id.to_string();
    let sender_str = notif.sender_id.map(|id| id.to_string());
    let note_str = notif.note_id.map(|id| id.to_string());
    let type_str = notif_type_str(notif.notification_type);

    client
        .query(
            "
            INSERT INTO notification {
                id: $id,
                created_at: $created_at,
                user_id: type::record('user', $recipient),
                notification_type: $notification_type,
                notifier_id: if $sender != None { type::record('user', $sender) } else { None },
                note_id: if $note != None { type::record('note', $note) } else { None },
                reaction: $reaction,
                is_read: $is_read
            };
            ",
        )
        .bind(("id", id_str))
        .bind(("created_at", notif.created_at))
        .bind(("recipient", recipient_str))
        .bind(("notification_type", type_str))
        .bind(("sender", sender_str))
        .bind(("note", note_str))
        .bind(("reaction", notif.reaction.clone()))
        .bind(("is_read", notif.is_read))
        .await?;

    Ok(())
}

pub async fn get_notifications(
    client: &SurrealClient,
    recipient_id: &ActorId,
    limit: usize,
) -> anyhow::Result<Vec<Notification>> {
    let recipient_str = recipient_id.to_string();

    let mut response = client
        .query(
            "
            SELECT 
                id,
                created_at,
                notification_type,
                user_id.id AS recipient_id,
                notifier_id.id AS sender_id,
                note_id.id AS note_id,
                reaction,
                is_read
            FROM notification
            WHERE user_id = type::record('user', $recipient)
            ORDER BY created_at DESC
            LIMIT $limit;
            ",
        )
        .bind(("recipient", recipient_str))
        .bind(("limit", limit))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<Notification>(rows)
}

pub async fn mark_notification_as_read(
    client: &SurrealClient,
    id: &str,
    recipient_id: &ActorId,
) -> anyhow::Result<()> {
    let recipient_str = recipient_id.to_string();

    client
        .query(
            "
            UPDATE notification SET is_read = true WHERE id = type::record('notification', $id) AND user_id = type::record('user', $recipient);
            ",
        )
        .bind(("id", id.to_string()))
        .bind(("recipient", recipient_str))
        .await?;

    Ok(())
}

pub async fn mark_all_notifications_as_read(
    client: &SurrealClient,
    recipient_id: &ActorId,
) -> anyhow::Result<()> {
    let recipient_str = recipient_id.to_string();

    client
        .query(
            "
            UPDATE notification SET is_read = true WHERE user_id = type::record('user', $recipient) AND is_read = false;
            ",
        )
        .bind(("recipient", recipient_str))
        .await?;

    Ok(())
}
