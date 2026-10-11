use crate::db::Db;

pub async fn get_reaction_by_actor(
    db: &Db,
    note_id: &str,
    actor_id: &str,
) -> anyhow::Result<Option<String>> {
    let reaction: Option<String> = sqlx::query_scalar(
        "SELECT reaction FROM note_reactions
         WHERE note_id = $1 AND actor_id = $2 LIMIT 1",
    )
    .bind(note_id)
    .bind(actor_id)
    .fetch_optional(db)
    .await?;
    Ok(reaction)
}

/// 閲覧者のリアクションをノート ID の集合に対して一括取得する (note_id → emoji)
pub async fn get_reactions_by_actor_for_notes(
    db: &Db,
    actor_id: &str,
    note_ids: &[String],
) -> anyhow::Result<std::collections::HashMap<String, String>> {
    if note_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT note_id, reaction FROM note_reactions
         WHERE actor_id = $1 AND note_id = ANY($2)",
    )
    .bind(actor_id)
    .bind(note_ids)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().collect())
}

pub async fn add_reaction(
    db: &Db,
    note_id: &str,
    actor_id: &str,
    reaction: &str,
) -> anyhow::Result<()> {
    let id_str = ulid::Ulid::generate().to_string();
    let created_at = chrono::Utc::now();

    let mut tx = db.begin().await?;

    sqlx::query(
        "INSERT INTO note_reactions (id, note_id, actor_id, reaction, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id_str)
    .bind(note_id)
    .bind(actor_id)
    .bind(reaction)
    .bind(created_at)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE notes
         SET reactions = jsonb_set(
                COALESCE(reactions, '{}'),
                ARRAY[$3],
                to_jsonb(COALESCE((reactions->>$3)::int, 0) + 1)
             )
         WHERE id = $1",
    )
    .bind(note_id)
    .bind(actor_id)
    .bind(reaction)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn remove_reaction(
    db: &Db,
    note_id: &str,
    actor_id: &str,
    reaction: &str,
) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;

    sqlx::query(
        "DELETE FROM note_reactions
         WHERE note_id = $1 AND actor_id = $2 AND reaction = $3",
    )
    .bind(note_id)
    .bind(actor_id)
    .bind(reaction)
    .execute(&mut *tx)
    .await?;

    // 0 以下になるキーは jsonb から除去する (`object::remove` 相当)
    sqlx::query(
        "UPDATE notes
         SET reactions = CASE
                WHEN COALESCE((reactions->>$3)::int, 1) - 1 <= 0 THEN reactions - $3
                ELSE jsonb_set(reactions, ARRAY[$3], to_jsonb((reactions->>$3)::int - 1))
             END
         WHERE id = $1",
    )
    .bind(note_id)
    .bind(actor_id)
    .bind(reaction)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// アクターの当該ノートへのリアクションをすべて削除 (Undo(Like) 用)
pub async fn remove_all_reactions_by_actor(
    db: &Db,
    note_id: &str,
    actor_id: &str,
) -> anyhow::Result<Vec<String>> {
    let removed: Vec<String> = sqlx::query_scalar(
        "DELETE FROM note_reactions
         WHERE note_id = $1 AND actor_id = $2
         RETURNING reaction",
    )
    .bind(note_id)
    .bind(actor_id)
    .fetch_all(db)
    .await?;

    if removed.is_empty() {
        return Ok(removed);
    }

    let mut tx = db.begin().await?;
    for reaction in &removed {
        sqlx::query(
            "UPDATE notes
             SET reactions = CASE
                    WHEN COALESCE((reactions->>$2)::int, 1) - 1 <= 0 THEN reactions - $2
                    ELSE jsonb_set(reactions, ARRAY[$2], to_jsonb((reactions->>$2)::int - 1))
                 END
             WHERE id = $1",
        )
        .bind(note_id)
        .bind(reaction)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(removed)
}
