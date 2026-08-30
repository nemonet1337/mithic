use crate::SurrealClient;
use mithic_core::models::activity::ActivityId;

pub async fn create_activity(
    client: &SurrealClient,
    uri: &str,
    activity_type: &str,
    actor_id: Option<&str>,
    note_id: Option<&str>,
) -> anyhow::Result<()> {
    client
        .query(
            "INSERT INTO activity { id: $id, uri: $uri, activity_type: $type, actor_id: $actor, note_id: $note, created_at: time::now() };"
        )
        .bind(("id", ActivityId::new().to_string()))
        .bind(("uri", uri.to_string()))
        .bind(("type", activity_type.to_string()))
        .bind(("actor", actor_id.map(|s| s.to_string())))
        .bind(("note", note_id.map(|s| s.to_string())))
        .await?;
    Ok(())
}

pub async fn get_activity_by_uri(client: &SurrealClient, uri: &str) -> anyhow::Result<bool> {
    let mut response = client
        .query("SELECT VALUE count() FROM activity WHERE uri = $uri")
        .bind(("uri", uri.to_string()))
        .await?;
    let counts: Vec<usize> = response.take(0)?;
    Ok(counts.first().copied().unwrap_or(0) > 0)
}
