use crate::SurrealClient;

pub async fn vote_poll(
    client: &SurrealClient,
    poll_id: &str,
    actor_id: &str,
    choice_index: usize,
) -> anyhow::Result<()> {
    let id_str = ulid::Ulid::new().to_string();
    let created_at = chrono::Utc::now();

    client
        .query(
            "
        BEGIN TRANSACTION;
        INSERT INTO poll_vote {
            id: $id,
            poll_id: type::record('poll', $poll_id),
            actor_id: type::record('user', $actor_id),
            choice_index: $choice_index,
            created_at: $created_at
        };
        UPDATE poll SET choices[$choice_index].votes = <int>(choices[$choice_index].votes OR 0) + 1 WHERE id = type::record('poll', $poll_id);
        COMMIT TRANSACTION;
        ",
        )
        .bind(("id", id_str))
        .bind(("poll_id", poll_id.to_string()))
        .bind(("actor_id", actor_id.to_string()))
        .bind(("choice_index", choice_index as i32))
        .bind(("created_at", created_at))
        .await?;

    Ok(())
}
