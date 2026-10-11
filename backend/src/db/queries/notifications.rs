use crate::db::Db;
use crate::db::queries::actors::parse_actor_id;
use crate::models::actor::ActorId;
use crate::models::notification::{Notification, notif_type_from_str, notif_type_str};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
struct NotificationRow {
    id: String,
    created_at: DateTime<Utc>,
    notification_type: String,
    user_id: String,
    notifier_id: Option<String>,
    note_id: Option<String>,
    reaction: Option<String>,
    is_read: bool,
}

impl TryFrom<NotificationRow> for Notification {
    type Error = anyhow::Error;

    fn try_from(row: NotificationRow) -> anyhow::Result<Self> {
        Ok(Notification {
            id: row
                .id
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid notification id '{}': {e}", row.id))?,
            created_at: row.created_at,
            notification_type: notif_type_from_str(&row.notification_type)?,
            recipient_id: parse_actor_id(&row.user_id)?,
            sender_id: row.notifier_id.as_deref().map(parse_actor_id).transpose()?,
            note_id: row
                .note_id
                .as_deref()
                .map(|raw| {
                    raw.parse()
                        .map_err(|e| anyhow::anyhow!("invalid note id '{raw}': {e}"))
                })
                .transpose()?,
            reaction: row.reaction,
            is_read: row.is_read,
        })
    }
}

pub async fn create_notification(db: &Db, notif: &Notification) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO notifications (
            id, created_at, notification_type, user_id, notifier_id, note_id, reaction, is_read
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(notif.id.to_string())
    .bind(notif.created_at)
    .bind(notif_type_str(notif.notification_type))
    .bind(notif.recipient_id.to_string())
    .bind(notif.sender_id.map(|id| id.to_string()))
    .bind(notif.note_id.map(|id| id.to_string()))
    .bind(&notif.reaction)
    .bind(notif.is_read)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_notifications(
    db: &Db,
    recipient_id: &ActorId,
    limit: usize,
) -> anyhow::Result<Vec<Notification>> {
    let rows = sqlx::query_as::<_, NotificationRow>(
        "SELECT id, created_at, notification_type, user_id, notifier_id, note_id, reaction, is_read
         FROM notifications
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT $2",
    )
    .bind(recipient_id.to_string())
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(Notification::try_from).collect()
}

pub async fn mark_notification_as_read(
    db: &Db,
    id: &str,
    recipient_id: &ActorId,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE notifications SET is_read = true
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(recipient_id.to_string())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_all_notifications_as_read(db: &Db, recipient_id: &ActorId) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE notifications SET is_read = true
         WHERE user_id = $1 AND is_read = false",
    )
    .bind(recipient_id.to_string())
    .execute(db)
    .await?;
    Ok(())
}
