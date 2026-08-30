use crate::SurrealClient;
use crate::queries::rows_to;
use anyhow::anyhow;
use mithic_core::models::actor::{Actor, ActorId};

pub async fn create_actor(client: &SurrealClient, actor: &Actor) -> anyhow::Result<Actor> {
    let id_str = actor.id.to_string();
    let mut response = client
        .query(
            "
            INSERT INTO user {
                id: $id,
                username: $username,
                username_lower: $username_lower,
                name: $name,
                password_hash: $password_hash,
                email: $email,
                created_at: $created_at,
                updated_at: $updated_at,
                followers_count: $followers_count,
                following_count: $following_count,
                notes_count: $notes_count,
                avatar_url: $avatar_url,
                banner_url: $banner_url,
                is_suspended: $is_suspended,
                is_locked: $is_locked,
                is_bot: $is_bot,
                is_admin: $is_admin,
                host: $host,
                inbox: $inbox,
                shared_inbox: $shared_inbox,
                featured: $featured,
                uri: $uri,
                public_key: $public_key,
                private_key: $private_key,
                token: $user_token
            };
        ",
        )
        .bind(("id", id_str))
        .bind(("username", actor.username.clone()))
        .bind(("username_lower", actor.username_lower.clone()))
        .bind(("name", actor.name.clone()))
        .bind(("password_hash", actor.password_hash.clone()))
        .bind(("email", actor.email.clone()))
        .bind(("created_at", actor.created_at))
        .bind(("updated_at", actor.updated_at))
        .bind(("followers_count", actor.followers_count))
        .bind(("following_count", actor.following_count))
        .bind(("notes_count", actor.notes_count))
        .bind(("avatar_url", actor.avatar_url.clone()))
        .bind(("banner_url", actor.banner_url.clone()))
        .bind(("is_suspended", actor.is_suspended))
        .bind(("is_locked", actor.is_locked))
        .bind(("is_bot", actor.is_bot))
        .bind(("is_admin", actor.is_admin))
        .bind(("host", actor.host.clone()))
        .bind(("inbox", actor.inbox.clone()))
        .bind(("shared_inbox", actor.shared_inbox.clone()))
        .bind(("featured", actor.featured.clone()))
        .bind(("uri", actor.uri.clone()))
        .bind(("public_key", actor.public_key.clone()))
        .bind(("private_key", actor.private_key.clone()))
        .bind(("user_token", actor.token.clone()))
        .await?;

    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<Actor>(rows)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("Failed to create user"))
}

pub async fn get_actor_by_id(
    client: &SurrealClient,
    id: &ActorId,
) -> anyhow::Result<Option<Actor>> {
    let id_str = id.to_string();
    let mut response = client
        .query("SELECT * FROM type::record('user', $id);")
        .bind(("id", id_str))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows_to::<Actor>(rows)?.into_iter().next())
}

pub async fn get_actors_by_ids(
    client: &SurrealClient,
    ids: &[String],
) -> anyhow::Result<Vec<Actor>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let id_records: Vec<String> = ids.iter().map(|id| format!("user:{id}")).collect();
    let mut response = client
        .query("SELECT * FROM user WHERE id IN $ids;")
        .bind(("ids", id_records))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    rows_to::<Actor>(rows)
}

pub async fn get_actor_by_username(
    client: &SurrealClient,
    username: &str,
) -> anyhow::Result<Option<Actor>> {
    let username_lower = username.to_lowercase();
    let mut response = client
        .query("SELECT * FROM user WHERE username_lower = $username_lower AND host = NONE LIMIT 1;")
        .bind(("username_lower", username_lower))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows_to::<Actor>(rows)?.into_iter().next())
}

pub async fn get_actor_by_username_or_email(
    client: &SurrealClient,
    username_or_email: &str,
) -> anyhow::Result<Option<Actor>> {
    let identifier_lower = username_or_email.to_lowercase();
    let mut response = client
        .query(
            "SELECT * FROM user WHERE (username_lower = $identifier OR email = $identifier) AND host = NONE LIMIT 1;",
        )
        .bind(("identifier", identifier_lower))
        .await?;
    let rows: Vec<surrealdb::types::Value> = response.take(0)?;
    Ok(rows_to::<Actor>(rows)?.into_iter().next())
}

pub async fn update_actor_token(
    client: &SurrealClient,
    id: &ActorId,
    token: Option<String>,
) -> anyhow::Result<()> {
    let id_str = id.to_string();
    client
        .query("UPDATE user SET token = $user_token WHERE id = type::record('user', $id);")
        .bind(("id", id_str))
        .bind(("user_token", token))
        .await?;
    Ok(())
}

pub async fn enable_totp(client: &SurrealClient, id: &ActorId, secret: &str) -> anyhow::Result<()> {
    let id_str = id.to_string();
    client
        .query("UPDATE user SET totp_secret = $secret, totp_verified = true WHERE id = type::record('user', $id);")
        .bind(("id", id_str))
        .bind(("secret", secret.to_string()))
        .await?;
    Ok(())
}
