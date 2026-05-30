use crate::SurrealClient;

pub async fn add_reaction(
    client: &SurrealClient,
    note_id: &str,
    actor_id: &str,
    reaction: &str,
) -> anyhow::Result<()> {
    let id_str = ulid::Ulid::new().to_string();
    let created_at = chrono::Utc::now();

    client
        .query(
            "
            BEGIN TRANSACTION;
            INSERT INTO note_reaction {
                id: $id,
                note_id: type::thing('note', $note_id),
                actor_id: type::thing('user', $actor_id),
                reaction: $reaction,
                created_at: $created_at
            };
            UPDATE note SET reactions[$reaction] = <int>(reactions[$reaction] OR 0) + 1 WHERE id = $note_id;
            COMMIT TRANSACTION;
            ",
        )
        .bind(("id", id_str))
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", actor_id.to_string()))
        .bind(("reaction", reaction.to_string()))
        .bind(("created_at", created_at))
        .await?;

    Ok(())
}

pub async fn remove_reaction(
    client: &SurrealClient,
    note_id: &str,
    actor_id: &str,
    reaction: &str,
) -> anyhow::Result<()> {
    client
        .query(
            "
            BEGIN TRANSACTION;
            DELETE note_reaction WHERE note_id = type::thing('note', $note_id) AND actor_id = type::thing('user', $actor_id) AND reaction = $reaction;
            UPDATE note SET reactions[$reaction] = <int>(reactions[$reaction] OR 1) - 1 WHERE id = $note_id;
            COMMIT TRANSACTION;
            ",
        )
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", actor_id.to_string()))
        .bind(("reaction", reaction.to_string()))
        .await?;

    Ok(())
}
