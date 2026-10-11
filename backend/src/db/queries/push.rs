//! Web Push subscription storage

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::db::Db;
use crate::models::actor::ActorId;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub id: String,
    pub user_id: ActorId,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PushSubscriptionRow {
    id: String,
    user_id: String,
    endpoint: String,
    p256dh: String,
    auth: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<PushSubscriptionRow> for PushSubscription {
    type Error = anyhow::Error;

    fn try_from(row: PushSubscriptionRow) -> anyhow::Result<Self> {
        Ok(PushSubscription {
            id: row.id,
            user_id: row
                .user_id
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid user id '{}': {e}", row.user_id))?,
            endpoint: row.endpoint,
            p256dh: row.p256dh,
            auth: row.auth,
            created_at: row.created_at,
        })
    }
}

/// Upsert by endpoint (one device endpoint → one row). Reassigns to `user_id`.
pub async fn upsert_push_subscription(
    db: &Db,
    user_id: &ActorId,
    endpoint: &str,
    p256dh: &str,
    auth: &str,
) -> anyhow::Result<PushSubscription> {
    let id = Ulid::generate().to_string();
    let created_at = Utc::now();
    let user = user_id.to_string();

    sqlx::query(
        "INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh, auth, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (endpoint)
         DO UPDATE SET user_id = EXCLUDED.user_id, p256dh = EXCLUDED.p256dh, auth = EXCLUDED.auth",
    )
    .bind(&id)
    .bind(&user)
    .bind(endpoint)
    .bind(p256dh)
    .bind(auth)
    .bind(created_at)
    .execute(db)
    .await?;

    Ok(PushSubscription {
        id,
        user_id: *user_id,
        endpoint: endpoint.to_string(),
        p256dh: p256dh.to_string(),
        auth: auth.to_string(),
        created_at,
    })
}

pub async fn list_push_subscriptions(
    db: &Db,
    user_id: &ActorId,
) -> anyhow::Result<Vec<PushSubscription>> {
    let rows = sqlx::query_as::<_, PushSubscriptionRow>(
        "SELECT id, user_id, endpoint, p256dh, auth, created_at
         FROM push_subscriptions
         WHERE user_id = $1",
    )
    .bind(user_id.to_string())
    .fetch_all(db)
    .await?;

    rows.into_iter().map(PushSubscription::try_from).collect()
}

pub async fn delete_push_subscriptions_for_user(db: &Db, user_id: &ActorId) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM push_subscriptions WHERE user_id = $1")
        .bind(user_id.to_string())
        .execute(db)
        .await?;
    Ok(())
}

pub async fn delete_push_subscription_by_endpoint(db: &Db, endpoint: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM push_subscriptions WHERE endpoint = $1")
        .bind(endpoint)
        .execute(db)
        .await?;
    Ok(())
}
