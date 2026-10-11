use crate::db::Db;
use crate::db::queries::actors::ActorRow;
use crate::models::actor::{Actor, ActorId};

pub async fn follow_user(
    db: &Db,
    follower_id: &ActorId,
    followee_id: &ActorId,
    accepted: bool,
) -> anyhow::Result<()> {
    let created_at = chrono::Utc::now();
    let follower = follower_id.to_string();
    let followee = followee_id.to_string();

    let mut tx = db.begin().await?;

    sqlx::query(
        "INSERT INTO follows (follower_id, followee_id, created_at, is_accepted)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (follower_id, followee_id)
         DO UPDATE SET created_at = EXCLUDED.created_at, is_accepted = EXCLUDED.is_accepted",
    )
    .bind(&follower)
    .bind(&followee)
    .bind(created_at)
    .bind(accepted)
    .execute(&mut *tx)
    .await?;

    if accepted {
        sqlx::query("UPDATE users SET following_count = following_count + 1 WHERE id = $1")
            .bind(&follower)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE users SET followers_count = followers_count + 1 WHERE id = $1")
            .bind(&followee)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn unfollow_user(
    db: &Db,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<()> {
    let follower = follower_id.to_string();
    let followee = followee_id.to_string();

    let mut tx = db.begin().await?;

    let accepted: Option<bool> = sqlx::query_scalar(
        "SELECT is_accepted FROM follows WHERE follower_id = $1 AND followee_id = $2",
    )
    .bind(&follower)
    .bind(&followee)
    .fetch_optional(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM follows WHERE follower_id = $1 AND followee_id = $2")
        .bind(&follower)
        .bind(&followee)
        .execute(&mut *tx)
        .await?;

    if accepted == Some(true) {
        sqlx::query(
            "UPDATE users SET following_count = GREATEST(following_count - 1, 0) WHERE id = $1",
        )
        .bind(&follower)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE users SET followers_count = GREATEST(followers_count - 1, 0) WHERE id = $1",
        )
        .bind(&followee)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn block_user(db: &Db, blocker_id: &ActorId, blocked_id: &ActorId) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO blocks (blocker_id, blocked_id, created_at) VALUES ($1, $2, $3)
         ON CONFLICT (blocker_id, blocked_id) DO NOTHING",
    )
    .bind(blocker_id.to_string())
    .bind(blocked_id.to_string())
    .bind(chrono::Utc::now())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn unblock_user(
    db: &Db,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM blocks WHERE blocker_id = $1 AND blocked_id = $2")
        .bind(blocker_id.to_string())
        .bind(blocked_id.to_string())
        .execute(db)
        .await?;
    Ok(())
}

pub async fn mute_user(db: &Db, muter_id: &ActorId, muted_id: &ActorId) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO mutes (muter_id, muted_id, created_at) VALUES ($1, $2, $3)
         ON CONFLICT (muter_id, muted_id) DO NOTHING",
    )
    .bind(muter_id.to_string())
    .bind(muted_id.to_string())
    .bind(chrono::Utc::now())
    .execute(db)
    .await?;
    Ok(())
}

pub async fn unmute_user(db: &Db, muter_id: &ActorId, muted_id: &ActorId) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM mutes WHERE muter_id = $1 AND muted_id = $2")
        .bind(muter_id.to_string())
        .bind(muted_id.to_string())
        .execute(db)
        .await?;
    Ok(())
}

pub async fn is_following(
    db: &Db,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM follows
            WHERE follower_id = $1 AND followee_id = $2 AND is_accepted
        )",
    )
    .bind(follower_id.to_string())
    .bind(followee_id.to_string())
    .fetch_one(db)
    .await?;
    Ok(exists)
}

/// フォローリクエストの状態。`None` = フォロー関係なし、`Some(false)` = 承認待ち
pub async fn follow_request_state(
    db: &Db,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<Option<bool>> {
    let state: Option<Option<bool>> = sqlx::query_scalar(
        "SELECT is_accepted FROM follows
         WHERE follower_id = $1 AND followee_id = $2 LIMIT 1",
    )
    .bind(follower_id.to_string())
    .bind(followee_id.to_string())
    .fetch_optional(db)
    .await?;
    Ok(state.flatten())
}

pub async fn is_blocking(
    db: &Db,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM blocks WHERE blocker_id = $1 AND blocked_id = $2)",
    )
    .bind(blocker_id.to_string())
    .bind(blocked_id.to_string())
    .fetch_one(db)
    .await?;
    Ok(exists)
}

pub async fn is_muting(db: &Db, muter_id: &ActorId, muted_id: &ActorId) -> anyhow::Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM mutes WHERE muter_id = $1 AND muted_id = $2)",
    )
    .bind(muter_id.to_string())
    .bind(muted_id.to_string())
    .fetch_one(db)
    .await?;
    Ok(exists)
}

pub async fn get_following(db: &Db, user_id: &ActorId) -> anyhow::Result<Vec<Actor>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        "SELECT u.*
         FROM users u
         JOIN follows f ON u.id = f.followee_id
         WHERE f.follower_id = $1 AND f.is_accepted",
    )
    .bind(user_id.to_string())
    .fetch_all(db)
    .await?;

    rows.into_iter().map(Actor::try_from).collect()
}

pub async fn get_followers(db: &Db, user_id: &ActorId) -> anyhow::Result<Vec<Actor>> {
    let rows = sqlx::query_as::<_, ActorRow>(
        "SELECT u.*
         FROM users u
         JOIN follows f ON u.id = f.follower_id
         WHERE f.followee_id = $1 AND f.is_accepted",
    )
    .bind(user_id.to_string())
    .fetch_all(db)
    .await?;

    rows.into_iter().map(Actor::try_from).collect()
}
