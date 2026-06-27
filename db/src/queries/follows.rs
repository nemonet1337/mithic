use crate::SurrealClient;
use mithic_core::models::actor::{Actor, ActorId};

pub async fn follow_user(
    client: &SurrealClient,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<()> {
    let follower_str = follower_id.to_string();
    let followee_str = followee_id.to_string();
    let created_at = chrono::Utc::now();

    client
        .query(
            "
            RELATE (type::record('user', $follower)) -> follow -> (type::record('user', $followee))
            SET created_at = $created_at;
            ",
        )
        .bind(("follower", follower_str))
        .bind(("followee", followee_str))
        .bind(("created_at", created_at))
        .await?;

    Ok(())
}

pub async fn unfollow_user(
    client: &SurrealClient,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<()> {
    let follower_str = follower_id.to_string();
    let followee_str = followee_id.to_string();

    client
        .query(
            "
            DELETE follow WHERE in = type::record('user', $follower) AND out = type::record('user', $followee);
            ",
        )
        .bind(("follower", follower_str))
        .bind(("followee", followee_str))
        .await?;

    Ok(())
}

pub async fn block_user(
    client: &SurrealClient,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<()> {
    let blocker_str = blocker_id.to_string();
    let blocked_str = blocked_id.to_string();
    let created_at = chrono::Utc::now();

    client
        .query(
            "
            RELATE (type::record('user', $blocker)) -> block -> (type::record('user', $blocked))
            SET created_at = $created_at;
            ",
        )
        .bind(("blocker", blocker_str))
        .bind(("blocked", blocked_str))
        .bind(("created_at", created_at))
        .await?;

    Ok(())
}

pub async fn unblock_user(
    client: &SurrealClient,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<()> {
    let blocker_str = blocker_id.to_string();
    let blocked_str = blocked_id.to_string();

    client
        .query(
            "
            DELETE block WHERE in = type::record('user', $blocker) AND out = type::record('user', $blocked);
            ",
        )
        .bind(("blocker", blocker_str))
        .bind(("blocked", blocked_str))
        .await?;

    Ok(())
}

pub async fn mute_user(
    client: &SurrealClient,
    muter_id: &ActorId,
    muted_id: &ActorId,
) -> anyhow::Result<()> {
    let muter_str = muter_id.to_string();
    let muted_str = muted_id.to_string();
    let created_at = chrono::Utc::now();

    client
        .query(
            "
            RELATE (type::record('user', $muter)) -> mute -> (type::record('user', $muted))
            SET created_at = $created_at;
            ",
        )
        .bind(("muter", muter_str))
        .bind(("muted", muted_str))
        .bind(("created_at", created_at))
        .await?;

    Ok(())
}

pub async fn unmute_user(
    client: &SurrealClient,
    muter_id: &ActorId,
    muted_id: &ActorId,
) -> anyhow::Result<()> {
    let muter_str = muter_id.to_string();
    let muted_str = muted_id.to_string();

    client
        .query(
            "
            DELETE mute WHERE in = type::record('user', $muter) AND out = type::record('user', $muted);
            ",
        )
        .bind(("muter", muter_str))
        .bind(("muted", muted_str))
        .await?;

    Ok(())
}

pub async fn count_followers(
    client: &SurrealClient,
    actor_id: &ActorId,
) -> anyhow::Result<usize> {
    let mut response = client
        .query("SELECT VALUE count() FROM follow WHERE out = type::record('user', $actor_id)")
        .bind(("actor_id", actor_id.to_string()))
        .await?;
    let counts: Vec<usize> = response.take(0)?;
    Ok(counts.into_iter().next().unwrap_or(0))
}

pub async fn is_following(
    client: &SurrealClient,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<bool> {
    let follower_str = follower_id.to_string();
    let followee_str = followee_id.to_string();

    let mut response = client
        .query(
            "
            SELECT VALUE count() FROM follow WHERE in = type::record('user', $follower) AND out = type::record('user', $followee);
            ",
        )
        .bind(("follower", follower_str))
        .bind(("followee", followee_str))
        .await?;

    let counts: Vec<usize> = response.take(0)?;
    Ok(counts.first().cloned().unwrap_or(0) > 0)
}

pub async fn is_blocking(
    client: &SurrealClient,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<bool> {
    let blocker_str = blocker_id.to_string();
    let blocked_str = blocked_id.to_string();

    let mut response = client
        .query(
            "
            SELECT VALUE count() FROM block WHERE in = type::record('user', $blocker) AND out = type::record('user', $blocked);
            ",
        )
        .bind(("blocker", blocker_str))
        .bind(("blocked", blocked_str))
        .await?;

    let counts: Vec<usize> = response.take(0)?;
    Ok(counts.first().cloned().unwrap_or(0) > 0)
}

pub async fn is_muting(
    client: &SurrealClient,
    muter_id: &ActorId,
    muted_id: &ActorId,
) -> anyhow::Result<bool> {
    let muter_str = muter_id.to_string();
    let muted_str = muted_id.to_string();

    let mut response = client
        .query(
            "
            SELECT VALUE count() FROM mute WHERE in = type::record('user', $muter) AND out = type::record('user', $muted);
            ",
        )
        .bind(("muter", muter_str))
        .bind(("muted", muted_str))
        .await?;

    let counts: Vec<usize> = response.take(0)?;
    Ok(counts.first().cloned().unwrap_or(0) > 0)
}

pub async fn get_following(
    client: &SurrealClient,
    user_id: &ActorId,
) -> anyhow::Result<Vec<Actor>> {
    let user_str = user_id.to_string();
    let mut response = client
        .query(
            "
            SELECT out.* AS actor FROM follow WHERE in = type::record('user', $user);
            ",
        )
        .bind(("user", user_str))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    let actors: Vec<serde_json::Value> = rows
        .into_iter()
        .filter_map(|val| {
            let mut json = val.into_json_value();
            crate::queries::strip_record_prefixes(&mut json);
            json.get("actor").cloned()
        })
        .collect();

    let mut parsed_actors = Vec::new();
    for actor_val in actors {
        if let Ok(actor) = serde_json::from_value::<Actor>(actor_val) {
            parsed_actors.push(actor);
        }
    }
    Ok(parsed_actors)
}

pub async fn get_followers(
    client: &SurrealClient,
    user_id: &ActorId,
) -> anyhow::Result<Vec<Actor>> {
    let user_str = user_id.to_string();
    let mut response = client
        .query(
            "
            SELECT in.* AS actor FROM follow WHERE out = type::record('user', $user);
            ",
        )
        .bind(("user", user_str))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    let actors: Vec<serde_json::Value> = rows
        .into_iter()
        .filter_map(|val| {
            let mut json = val.into_json_value();
            crate::queries::strip_record_prefixes(&mut json);
            json.get("actor").cloned()
        })
        .collect();

    let mut parsed_actors = Vec::new();
    for actor_val in actors {
        if let Ok(actor) = serde_json::from_value::<Actor>(actor_val) {
            parsed_actors.push(actor);
        }
    }
    Ok(parsed_actors)
}
