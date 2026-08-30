//! Web Push subscription storage

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::SurrealClient;
use crate::queries::rows_to;
use mithic_core::models::actor::ActorId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub id: String,
    pub user_id: ActorId,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub created_at: DateTime<Utc>,
}

/// Upsert by endpoint (one device endpoint → one row). Reassigns to `user_id`.
pub async fn upsert_push_subscription(
    client: &SurrealClient,
    user_id: &ActorId,
    endpoint: &str,
    p256dh: &str,
    auth: &str,
) -> anyhow::Result<PushSubscription> {
    let id = Ulid::new().to_string();
    let created_at = Utc::now();
    let user = user_id.to_string();

    // Delete any existing row for this endpoint, then insert
    client
        .query("DELETE push_subscription WHERE endpoint = $endpoint;")
        .bind(("endpoint", endpoint.to_string()))
        .await?;

    client
        .query(
            "
            INSERT INTO push_subscription {
                id: $id,
                user_id: type::record('user', $user),
                endpoint: $endpoint,
                p256dh: $p256dh,
                auth: $auth,
                created_at: $created_at
            };
            ",
        )
        .bind(("id", id.clone()))
        .bind(("user", user))
        .bind(("endpoint", endpoint.to_string()))
        .bind(("p256dh", p256dh.to_string()))
        .bind(("auth", auth.to_string()))
        .bind(("created_at", created_at))
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
    client: &SurrealClient,
    user_id: &ActorId,
) -> anyhow::Result<Vec<PushSubscription>> {
    let mut response = client
        .query(
            "
            SELECT id, user_id.id AS user_id, endpoint, p256dh, auth, created_at
            FROM push_subscription
            WHERE user_id = type::record('user', $user);
            ",
        )
        .bind(("user", user_id.to_string()))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to(rows)
}

pub async fn delete_push_subscriptions_for_user(
    client: &SurrealClient,
    user_id: &ActorId,
) -> anyhow::Result<()> {
    client
        .query("DELETE push_subscription WHERE user_id = type::record('user', $user);")
        .bind(("user", user_id.to_string()))
        .await?;
    Ok(())
}

pub async fn delete_push_subscription_by_endpoint(
    client: &SurrealClient,
    endpoint: &str,
) -> anyhow::Result<()> {
    client
        .query("DELETE push_subscription WHERE endpoint = $endpoint;")
        .bind(("endpoint", endpoint.to_string()))
        .await?;
    Ok(())
}
