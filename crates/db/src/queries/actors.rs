use anyhow::anyhow;
use mithic_core::models::actor::{Actor, ActorId};
use crate::SurrealClient;

pub async fn create_actor(client: &SurrealClient, actor: &Actor) -> anyhow::Result<Actor> {
    let id_str = actor.id.to_string();
    let mut response = client
        .query("
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
                token: $token
            };
        ")
        .bind(("id", &id_str))
        .bind(("username", &actor.username))
        .bind(("username_lower", &actor.username_lower))
        .bind(("name", &actor.name))
        .bind(("password_hash", &actor.password_hash))
        .bind(("email", &actor.email))
        .bind(("created_at", &actor.created_at))
        .bind(("updated_at", &actor.updated_at))
        .bind(("followers_count", actor.followers_count))
        .bind(("following_count", actor.following_count))
        .bind(("notes_count", actor.notes_count))
        .bind(("avatar_url", &actor.avatar_url))
        .bind(("banner_url", &actor.banner_url))
        .bind(("is_suspended", actor.is_suspended))
        .bind(("is_locked", actor.is_locked))
        .bind(("is_bot", actor.is_bot))
        .bind(("is_admin", actor.is_admin))
        .bind(("host", &actor.host))
        .bind(("inbox", &actor.inbox))
        .bind(("shared_inbox", &actor.shared_inbox))
        .bind(("featured", &actor.featured))
        .bind(("uri", &actor.uri))
        .bind(("public_key", &actor.public_key))
        .bind(("private_key", &actor.private_key))
        .bind(("token", &actor.token))
        .await?;

    let created: Option<Actor> = response.take(0)?;
    created.ok_or_else(|| anyhow!("Failed to create user"))
}

pub async fn get_actor_by_id(client: &SurrealClient, id: &ActorId) -> anyhow::Result<Option<Actor>> {
    let id_str = id.to_string();
    let mut response = client
        .query("SELECT * FROM user WHERE id = $id LIMIT 1;")
        .bind(("id", id_str))
        .await?;
    let actor: Option<Actor> = response.take(0)?;
    Ok(actor)
}

pub async fn get_actor_by_username(client: &SurrealClient, username: &str) -> anyhow::Result<Option<Actor>> {
    let username_lower = username.to_lowercase();
    let mut response = client
        .query("SELECT * FROM user WHERE username_lower = $username_lower LIMIT 1;")
        .bind(("username_lower", username_lower))
        .await?;
    let actor: Option<Actor> = response.take(0)?;
    Ok(actor)
}

pub async fn update_actor_token(client: &SurrealClient, id: &ActorId, token: Option<String>) -> anyhow::Result<()> {
    let id_str = id.to_string();
    client
        .query("UPDATE user SET token = $token WHERE id = $id;")
        .bind(("id", id_str))
        .bind(("token", token))
        .await?;
    Ok(())
}
