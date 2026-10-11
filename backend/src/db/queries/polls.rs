use crate::db::Db;

pub async fn vote_poll(
    db: &Db,
    poll_id: &str,
    actor_id: &str,
    choice_index: usize,
) -> anyhow::Result<()> {
    let id_str = ulid::Ulid::generate().to_string();
    let created_at = chrono::Utc::now();
    let index = choice_index as i32;

    let mut tx = db.begin().await?;

    sqlx::query(
        "INSERT INTO poll_votes (id, poll_id, actor_id, choice_index, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id_str)
    .bind(poll_id)
    .bind(actor_id)
    .bind(index)
    .bind(created_at)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE polls
         SET choices = jsonb_set(
                choices,
                ARRAY[$2::text, 'votes'],
                to_jsonb(COALESCE((choices->($2::int))->>'votes', '0')::int + 1)
             )
         WHERE id = $1",
    )
    .bind(poll_id)
    .bind(index)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
