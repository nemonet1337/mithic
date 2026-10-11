use crate::db::Db;
use crate::models::activity::ActivityId;

pub async fn create_activity(
    db: &Db,
    uri: &str,
    activity_type: &str,
    actor_id: Option<&str>,
    note_id: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO activities (id, uri, activity_type, actor_id, note_id, created_at)
         VALUES ($1, $2, $3, $4, $5, now())",
    )
    .bind(ActivityId::generate().to_string())
    .bind(uri)
    .bind(activity_type)
    .bind(actor_id)
    .bind(note_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn get_activity_by_uri(db: &Db, uri: &str) -> anyhow::Result<bool> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM activities WHERE uri = $1)")
        .bind(uri)
        .fetch_one(db)
        .await?;
    Ok(exists)
}
