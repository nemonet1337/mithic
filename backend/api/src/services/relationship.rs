use mithic_core::models::actor::ActorId;
use mithic_db::SurrealClient;
use mithic_db::queries::{
    block_user as db_block, follow_user as db_follow, mute_user as db_mute,
    unblock_user as db_unblock, unfollow_user as db_unfollow, unmute_user as db_unmute,
};

pub async fn follow(
    surreal: &SurrealClient,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<()> {
    db_follow(surreal, follower_id, followee_id).await?;

    let follower_str = follower_id.to_string();
    let followee_str = followee_id.to_string();

    surreal
        .query(
            "
        UPDATE user SET following_count += 1 WHERE id = type::record('user', $follower);
        UPDATE user SET followers_count += 1 WHERE id = type::record('user', $followee);
        ",
        )
        .bind(("follower", follower_str))
        .bind(("followee", followee_str))
        .await?;

    Ok(())
}

pub async fn unfollow(
    surreal: &SurrealClient,
    follower_id: &ActorId,
    followee_id: &ActorId,
) -> anyhow::Result<()> {
    db_unfollow(surreal, follower_id, followee_id).await?;

    let follower_str = follower_id.to_string();
    let followee_str = followee_id.to_string();

    surreal
        .query(
            "
        UPDATE user SET following_count = <int>(following_count OR 1) - 1 WHERE id = type::record('user', $follower);
        UPDATE user SET followers_count = <int>(followers_count OR 1) - 1 WHERE id = type::record('user', $followee);
        ",
        )
        .bind(("follower", follower_str))
        .bind(("followee", followee_str))
        .await?;

    Ok(())
}

pub async fn block(
    surreal: &SurrealClient,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<()> {
    db_block(surreal, blocker_id, blocked_id).await?;
    Ok(())
}

pub async fn unblock(
    surreal: &SurrealClient,
    blocker_id: &ActorId,
    blocked_id: &ActorId,
) -> anyhow::Result<()> {
    db_unblock(surreal, blocker_id, blocked_id).await?;
    Ok(())
}

pub async fn mute(
    surreal: &SurrealClient,
    muter_id: &ActorId,
    muted_id: &ActorId,
) -> anyhow::Result<()> {
    db_mute(surreal, muter_id, muted_id).await?;
    Ok(())
}

pub async fn unmute(
    surreal: &SurrealClient,
    muter_id: &ActorId,
    muted_id: &ActorId,
) -> anyhow::Result<()> {
    db_unmute(surreal, muter_id, muted_id).await?;
    Ok(())
}
