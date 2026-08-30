use crate::db::SurrealClient;
use crate::models::actor::ActorId;
use crate::models::note::NoteId;

pub async fn add_favorite(
    client: &SurrealClient,
    user_id: &ActorId,
    note_id: &NoteId,
) -> anyhow::Result<()> {
    let id_str = ulid::Ulid::generate().to_string();
    let user_str = user_id.to_string();
    let note_str = note_id.to_string();
    let created_at = chrono::Utc::now();

    client
        .query(
            "
            INSERT INTO bookmark {
                id: $id,
                user_id: type::record('user', $user),
                note_id: type::record('note', $note),
                created_at: $created_at
            };
            ",
        )
        .bind(("id", id_str))
        .bind(("user", user_str))
        .bind(("note", note_str))
        .bind(("created_at", created_at))
        .await?;

    Ok(())
}

pub async fn remove_favorite(
    client: &SurrealClient,
    user_id: &ActorId,
    note_id: &NoteId,
) -> anyhow::Result<()> {
    let user_str = user_id.to_string();
    let note_str = note_id.to_string();

    client
        .query(
            "
            DELETE bookmark WHERE user_id = type::record('user', $user) AND note_id = type::record('note', $note);
            ",
        )
        .bind(("user", user_str))
        .bind(("note", note_str))
        .await?;

    Ok(())
}

pub async fn is_favorited(
    client: &SurrealClient,
    user_id: &ActorId,
    note_id: &NoteId,
) -> anyhow::Result<bool> {
    let user_str = user_id.to_string();
    let note_str = note_id.to_string();

    let mut response = client
        .query(
            "
            SELECT VALUE count() FROM bookmark WHERE user_id = type::record('user', $user) AND note_id = type::record('note', $note);
            ",
        )
        .bind(("user", user_str))
        .bind(("note", note_str))
        .await?;

    let counts: Vec<usize> = response.take(0)?;
    Ok(counts.first().cloned().unwrap_or(0) > 0)
}
