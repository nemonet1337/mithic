use mithic_core::models::actor::ActorId;
use mithic_core::{AppError, Result};
use mithic_db::{DragonflyClient, SurrealClient};

pub async fn follow_user(
    surreal: &SurrealClient,
    dragonfly: &DragonflyClient,
    from_id: &ActorId,
    to_id: &ActorId,
) -> Result<()> {
    if *from_id == *to_id {
        return Err(AppError::Validation("Cannot follow yourself".to_string()));
    }

    surreal
        .query(
            "
            LET $follow = (CREATE follow SET in = type::record('user', $from), out = type::record('user', $to), is_accepted = true);
            UPDATE user SET following_count = <int>(following_count OR 0) + 1 WHERE id = type::record('user', $from);
            UPDATE user SET followers_count = <int>(followers_count OR 0) + 1 WHERE id = type::record('user', $to);
            ",
        )
        .bind(("from", from_id.to_string()))
        .bind(("to", to_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = dragonfly.sadd(format!("user:{}:follows", from_id), vec![to_id.to_string()]).await;

    Ok(())
}

pub async fn unfollow_user(
    surreal: &SurrealClient,
    dragonfly: &DragonflyClient,
    from_id: &ActorId,
    to_id: &ActorId,
) -> Result<()> {
    surreal
        .query(
            "
            DELETE follow WHERE in = type::record('user', $from) AND out = type::record('user', $to) AND is_accepted = true;
            UPDATE user SET following_count = <int>(following_count OR 1) - 1 WHERE id = type::record('user', $from);
            UPDATE user SET followers_count = <int>(followers_count OR 1) - 1 WHERE id = type::record('user', $to);
            ",
        )
        .bind(("from", from_id.to_string()))
        .bind(("to", to_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = dragonfly.srem(format!("user:{}:follows", from_id), vec![to_id.to_string()]).await;

    Ok(())
}

pub async fn block_user(
    surreal: &SurrealClient,
    dragonfly: &DragonflyClient,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> Result<()> {
    surreal
        .query(
            "CREATE block SET in = type::record('user', $blocker), out = type::record('user', $blocked);",
        )
        .bind(("blocker", blocker_id.to_string()))
        .bind(("blocked", blocked_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = dragonfly.sadd(format!("user:{}:blocks", blocker_id), vec![blocked_id.to_string()]).await;

    Ok(())
}