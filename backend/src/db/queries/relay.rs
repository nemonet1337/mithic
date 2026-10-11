use crate::db::Db;
use crate::models::relay::{Relay, RelayStatus, relay_status_from_str};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
struct RelayRow {
    id: String,
    inbox: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
}

impl TryFrom<RelayRow> for Relay {
    type Error = anyhow::Error;

    fn try_from(row: RelayRow) -> anyhow::Result<Self> {
        Ok(Relay {
            id: row
                .id
                .parse()
                .map_err(|e| anyhow::anyhow!("invalid relay id '{}': {e}", row.id))?,
            inbox: row.inbox,
            status: relay_status_from_str(&row.status)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn rows_to_relays(rows: Vec<RelayRow>) -> anyhow::Result<Vec<Relay>> {
    rows.into_iter().map(Relay::try_from).collect()
}

pub async fn create_relay(db: &Db, inbox: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO relays (id, inbox, status, created_at) VALUES ($1, $2, 'requesting', now())",
    )
    .bind(ulid::Ulid::generate().to_string())
    .bind(inbox)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_relay_by_id(db: &Db, id: &str) -> anyhow::Result<Option<Relay>> {
    let row = sqlx::query_as::<_, RelayRow>(
        "SELECT id, inbox, status, created_at, updated_at FROM relays WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    row.map(Relay::try_from).transpose()
}

pub async fn get_relay_by_inbox(db: &Db, inbox: &str) -> anyhow::Result<Option<Relay>> {
    let row = sqlx::query_as::<_, RelayRow>(
        "SELECT id, inbox, status, created_at, updated_at FROM relays WHERE inbox = $1 LIMIT 1",
    )
    .bind(inbox)
    .fetch_optional(db)
    .await?;

    row.map(Relay::try_from).transpose()
}

pub async fn list_relays(db: &Db, limit: usize) -> anyhow::Result<Vec<Relay>> {
    let rows = sqlx::query_as::<_, RelayRow>(
        "SELECT id, inbox, status, created_at, updated_at FROM relays
         ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit.min(100) as i64)
    .fetch_all(db)
    .await?;

    rows_to_relays(rows)
}

fn relay_status_str(status: RelayStatus) -> &'static str {
    match status {
        RelayStatus::Requesting => "requesting",
        RelayStatus::Accepted => "accepted",
        RelayStatus::Rejected => "rejected",
    }
}

pub async fn update_relay_status(db: &Db, id: &str, status: RelayStatus) -> anyhow::Result<()> {
    sqlx::query("UPDATE relays SET status = $1, updated_at = now() WHERE id = $2")
        .bind(relay_status_str(status))
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn delete_relay(db: &Db, id: &str) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM relays WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn get_accepted_relays(db: &Db) -> anyhow::Result<Vec<Relay>> {
    let rows = sqlx::query_as::<_, RelayRow>(
        "SELECT id, inbox, status, created_at, updated_at FROM relays WHERE status = 'accepted'",
    )
    .fetch_all(db)
    .await?;

    rows_to_relays(rows)
}
