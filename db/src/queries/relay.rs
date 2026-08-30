use crate::SurrealClient;
use mithic_core::models::relay::{Relay, RelayStatus};

pub async fn create_relay(client: &SurrealClient, inbox: &str) -> anyhow::Result<()> {
    let id = ulid::Ulid::new().to_string();
    let status_str = "requesting";
    client
        .query(
            "INSERT INTO relay { id: $id, inbox: $inbox, status: $status, created_at: time::now() };"
        )
        .bind(("id", id))
        .bind(("inbox", inbox.to_string()))
        .bind(("status", status_str))
        .await?;
    Ok(())
}

pub async fn get_relay_by_id(client: &SurrealClient, id: &str) -> anyhow::Result<Option<Relay>> {
    let mut response = client
        .query("SELECT * FROM relay WHERE id = $id LIMIT 1")
        .bind(("id", id.to_string()))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    let mut result: Vec<serde_json::Value> =
        rows.into_iter().map(|v| v.into_json_value()).collect();
    Ok(result.pop().and_then(|mut v| {
        crate::queries::strip_record_prefixes(&mut v);
        serde_json::from_value(v).ok()
    }))
}

pub async fn get_relay_by_inbox(
    client: &SurrealClient,
    inbox: &str,
) -> anyhow::Result<Option<Relay>> {
    let mut response = client
        .query("SELECT * FROM relay WHERE inbox = $inbox LIMIT 1")
        .bind(("inbox", inbox.to_string()))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    let mut result: Vec<serde_json::Value> =
        rows.into_iter().map(|v| v.into_json_value()).collect();
    Ok(result.pop().and_then(|mut v| {
        crate::queries::strip_record_prefixes(&mut v);
        serde_json::from_value(v).ok()
    }))
}

pub async fn list_relays(client: &SurrealClient, limit: usize) -> anyhow::Result<Vec<Relay>> {
    let mut response = client
        .query(format!(
            "SELECT * FROM relay ORDER BY created_at DESC LIMIT {}",
            limit.min(100)
        ))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    let mut values: Vec<serde_json::Value> =
        rows.into_iter().map(|v| v.into_json_value()).collect();
    let mut relays = Vec::new();
    for mut v in values.drain(..) {
        crate::queries::strip_record_prefixes(&mut v);
        if let Ok(relay) = serde_json::from_value::<Relay>(v) {
            relays.push(relay);
        }
    }
    Ok(relays)
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
        .query("UPDATE relay SET status = $status, updated_at = time::now() WHERE id = $id")
        .bind(("status", status_str))
        .bind(("id", id.to_string()))
        .await?;
    Ok(())
}

pub async fn delete_relay(client: &SurrealClient, id: &str) -> anyhow::Result<()> {
    client
        .query("DELETE relay WHERE id = $id")
        .bind(("id", id.to_string()))
        .await?;
    Ok(())
}

pub async fn get_accepted_relays(client: &SurrealClient) -> anyhow::Result<Vec<Relay>> {
    let mut response = client
        .query("SELECT * FROM relay WHERE status = 'accepted'")
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    let mut values: Vec<serde_json::Value> =
        rows.into_iter().map(|v| v.into_json_value()).collect();
    let mut relays = Vec::new();
    for mut v in values.drain(..) {
        crate::queries::strip_record_prefixes(&mut v);
        if let Ok(relay) = serde_json::from_value::<Relay>(v) {
            relays.push(relay);
        }
    }
    Ok(relays)
}
