use crate::db::SurrealClient;
use crate::db::queries::rows_to;
use crate::models::relay::{Relay, RelayStatus};

pub async fn create_relay(client: &SurrealClient, inbox: &str) -> anyhow::Result<()> {
    let id = ulid::Ulid::generate().to_string();
    client
        .query(
            "INSERT INTO relay { id: $id, inbox: $inbox, status: 'requesting', created_at: time::now() };",
        )
        .bind(("id", id))
        .bind(("inbox", inbox.to_string()))
        .await?;
    Ok(())
}

pub async fn get_relay_by_id(client: &SurrealClient, id: &str) -> anyhow::Result<Option<Relay>> {
    let mut response = client
        .query("SELECT * FROM type::record('relay', $id);")
        .bind(("id", id.to_string()))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows_to::<Relay>(rows)?.into_iter().next())
}

pub async fn get_relay_by_inbox(
    client: &SurrealClient,
    inbox: &str,
) -> anyhow::Result<Option<Relay>> {
    let mut response = client
        .query("SELECT * FROM relay WHERE inbox = $inbox LIMIT 1;")
        .bind(("inbox", inbox.to_string()))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows_to::<Relay>(rows)?.into_iter().next())
}

pub async fn list_relays(client: &SurrealClient, limit: usize) -> anyhow::Result<Vec<Relay>> {
    let mut response = client
        .query("SELECT * FROM relay ORDER BY created_at DESC LIMIT $limit;")
        .bind(("limit", limit.min(100)))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<Relay>(rows)
}

pub async fn update_relay_status(
    client: &SurrealClient,
    id: &str,
    status: RelayStatus,
) -> anyhow::Result<()> {
    let status_str = match status {
        RelayStatus::Requesting => "requesting",
        RelayStatus::Accepted => "accepted",
        RelayStatus::Rejected => "rejected",
    };
    client
        .query("UPDATE type::record('relay', $id) SET status = $status, updated_at = time::now();")
        .bind(("status", status_str))
        .bind(("id", id.to_string()))
        .await?;
    Ok(())
}

pub async fn delete_relay(client: &SurrealClient, id: &str) -> anyhow::Result<()> {
    client
        .query("DELETE type::record('relay', $id);")
        .bind(("id", id.to_string()))
        .await?;
    Ok(())
}

pub async fn get_accepted_relays(client: &SurrealClient) -> anyhow::Result<Vec<Relay>> {
    let mut response = client
        .query("SELECT * FROM relay WHERE status = 'accepted';")
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<Relay>(rows)
}
