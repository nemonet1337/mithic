use crate::db::Db;
use crate::models::actor::ActorId;
use crate::models::note::NoteId;

pub async fn add_favorite(db: &Db, user_id: &ActorId, note_id: &NoteId) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO bookmarks (id, user_id, note_id, created_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, note_id) DO NOTHING",
    )
    .bind(ulid::Ulid::generate().to_string())
    .bind(user_id.to_string())
    .bind(note_id.to_string())
    .bind(chrono::Utc::now())
    .execute(db)
    .await?;

    Ok(())
}

pub async fn remove_favorite(db: &Db, user_id: &ActorId, note_id: &NoteId) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 AND note_id = $2")
        .bind(user_id.to_string())
        .bind(note_id.to_string())
        .execute(db)
        .await?;
    Ok(())
}

pub async fn is_favorited(db: &Db, user_id: &ActorId, note_id: &NoteId) -> anyhow::Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE user_id = $1 AND note_id = $2)",
    )
    .bind(user_id.to_string())
    .bind(note_id.to_string())
    .fetch_one(db)
    .await?;
    Ok(exists)
}
