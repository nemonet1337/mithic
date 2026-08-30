use crate::SurrealClient;

pub async fn get_reaction_by_actor(
    client: &SurrealClient,
    note_id: &str,
    actor_id: &str,
) -> anyhow::Result<Option<String>> {
    let mut response = client
        .query(
            "
            SELECT reaction FROM note_reaction
            WHERE note_id = type::record('note', $note_id)
              AND actor_id = type::record('user', $actor_id)
            LIMIT 1;
            ",
        )
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", actor_id.to_string()))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0).unwrap_or_default();
    for row in rows {
        let json = row.into_json_value();
        if let Some(r) = json.get("reaction").and_then(|v| v.as_str()) {
            return Ok(Some(r.to_string()));
        }
    }
    Ok(None)
}

/// 閲覧者のリアクションをノート ID の集合に対して一括取得する (note_id → emoji)
pub async fn get_reactions_by_actor_for_notes(
    client: &SurrealClient,
    actor_id: &str,
    note_ids: &[String],
) -> anyhow::Result<std::collections::HashMap<String, String>> {
    if note_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let id_records: Vec<String> = note_ids.iter().map(|id| format!("note:{id}")).collect();
    let mut response = client
        .query(
            "
            SELECT note_id.id AS note_id, reaction FROM note_reaction
            WHERE actor_id = type::record('user', $actor_id)
              AND note_id IN $note_ids;
            ",
        )
        .bind(("actor_id", actor_id.to_string()))
        .bind(("note_ids", id_records))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0).unwrap_or_default();
    let mut out = std::collections::HashMap::with_capacity(rows.len());
    for row in rows {
        let json = row.into_json_value();
        let Some(note_id) = json.get("note_id").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(reaction) = json.get("reaction").and_then(|v| v.as_str()) else {
            continue;
        };
        let note_id = note_id
            .rsplit_once(':')
            .map(|(_, id)| id)
            .unwrap_or(note_id);
        out.insert(note_id.to_string(), reaction.to_string());
    }
    Ok(out)
}

pub async fn add_reaction(
    client: &SurrealClient,
    note_id: &str,
    actor_id: &str,
    reaction: &str,
) -> anyhow::Result<()> {
    let id_str = ulid::Ulid::generate().to_string();
    let created_at = chrono::Utc::now();

    client
        .query(
            "
            BEGIN TRANSACTION;
            INSERT INTO note_reaction {
                id: $id,
                note_id: type::record('note', $note_id),
                actor_id: type::record('user', $actor_id),
                reaction: $reaction,
                created_at: $created_at
            };
            UPDATE note SET reactions[$reaction] = <int>(reactions[$reaction] OR 0) + 1 WHERE id = type::record('note', $note_id);
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
            DELETE note_reaction WHERE note_id = type::record('note', $note_id) AND actor_id = type::record('user', $actor_id) AND reaction = $reaction;
            UPDATE note SET reactions[$reaction] = <int>(reactions[$reaction] OR 1) - 1 WHERE id = type::record('note', $note_id);
            COMMIT TRANSACTION;
            ",
        )
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", actor_id.to_string()))
        .bind(("reaction", reaction.to_string()))
        .await?;

    Ok(())
}

/// アクターの当該ノートへのリアクションをすべて削除 (Undo(Like) 用)
pub async fn remove_all_reactions_by_actor(
    client: &SurrealClient,
    note_id: &str,
    actor_id: &str,
) -> anyhow::Result<Vec<String>> {
    let mut del = client
        .query(
            "
            DELETE note_reaction
            WHERE note_id = type::record('note', $note_id)
              AND actor_id = type::record('user', $actor_id)
            RETURN BEFORE;
            ",
        )
        .bind(("note_id", note_id.to_string()))
        .bind(("actor_id", actor_id.to_string()))
        .await?;

    let rows: Vec<surrealdb::types::Value> = del.take(0).unwrap_or_default();
    let mut removed = Vec::new();
    for row in rows {
        let json = row.into_json_value();
        if let Some(r) = json.get("reaction").and_then(|v| v.as_str()) {
            let reaction = r.to_string();
            let _ = client
                .query(
                    "UPDATE note SET reactions[$reaction] = <int>(reactions[$reaction] OR 1) - 1 WHERE id = type::record('note', $note_id);",
                )
                .bind(("note_id", note_id.to_string()))
                .bind(("reaction", reaction.clone()))
                .await;
            removed.push(reaction);
        }
    }
    Ok(removed)
}
