use crate::db::Db;
use crate::models::actor::{Actor, ActorId, ProfileField};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::str::FromStr;

/// users テーブルの取得カラム。`FromRow` は名前で対応づけるので並び順は問わない。
pub(crate) const ACTOR_COLUMNS: &str = "
    id, created_at, updated_at, username, username_lower, name, bio,
    followers_count, following_count, notes_count,
    avatar_url, banner_url,
    is_suspended, is_locked, is_bot, is_cat, is_admin,
    location, birthday, lang, fields,
    followed_message, reaction_acceptance,
    host, inbox, shared_inbox, featured,
    uri, public_key, private_key, token, password_hash, email,
    totp_secret, totp_verified
";

#[derive(Debug, FromRow)]
pub(crate) struct ActorRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub username: String,
    pub username_lower: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub followers_count: i32,
    pub following_count: i32,
    pub notes_count: i32,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub is_suspended: bool,
    pub is_locked: bool,
    pub is_bot: bool,
    pub is_cat: bool,
    pub is_admin: bool,
    pub location: Option<String>,
    pub birthday: Option<String>,
    pub lang: Option<String>,
    pub fields: sqlx::types::Json<Vec<ProfileField>>,
    pub followed_message: Option<String>,
    pub reaction_acceptance: Option<String>,
    pub host: Option<String>,
    pub inbox: Option<String>,
    pub shared_inbox: Option<String>,
    pub featured: Option<String>,
    pub uri: Option<String>,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub token: Option<String>,
    pub password_hash: Option<String>,
    pub email: Option<String>,
    pub totp_secret: Option<String>,
    pub totp_verified: bool,
}

pub(crate) fn parse_actor_id(raw: &str) -> anyhow::Result<ActorId> {
    ActorId::from_str(raw).map_err(|e| anyhow::anyhow!("invalid actor id '{raw}': {e}"))
}

impl TryFrom<ActorRow> for Actor {
    type Error = anyhow::Error;

    fn try_from(row: ActorRow) -> anyhow::Result<Self> {
        Ok(Actor {
            id: parse_actor_id(&row.id)?,
            created_at: row.created_at,
            updated_at: row.updated_at,
            username: row.username,
            username_lower: row.username_lower,
            name: row.name,
            bio: row.bio,
            followers_count: row.followers_count,
            following_count: row.following_count,
            notes_count: row.notes_count,
            avatar_url: row.avatar_url,
            banner_url: row.banner_url,
            is_suspended: row.is_suspended,
            is_locked: row.is_locked,
            is_bot: row.is_bot,
            is_cat: row.is_cat,
            is_admin: row.is_admin,
            location: row.location,
            birthday: row.birthday,
            lang: row.lang,
            fields: row.fields.0,
            followed_message: row.followed_message,
            reaction_acceptance: row.reaction_acceptance,
            host: row.host,
            inbox: row.inbox,
            shared_inbox: row.shared_inbox,
            featured: row.featured,
            uri: row.uri,
            public_key: row.public_key,
            private_key: row.private_key,
            token: row.token,
            password_hash: row.password_hash,
            email: row.email,
            totp_secret: row.totp_secret,
            totp_verified: row.totp_verified,
        })
    }
}

fn rows_to_actors(rows: Vec<ActorRow>) -> anyhow::Result<Vec<Actor>> {
    rows.into_iter().map(Actor::try_from).collect()
}

pub async fn create_actor(db: &Db, actor: &Actor) -> anyhow::Result<Actor> {
    let row = sqlx::query_as::<_, ActorRow>(&format!(
        "INSERT INTO users (
            id, created_at, updated_at, username, username_lower, name, bio,
            followers_count, following_count, notes_count,
            avatar_url, banner_url,
            is_suspended, is_locked, is_bot, is_cat, is_admin,
            location, birthday, lang, fields,
            followed_message, reaction_acceptance,
            host, inbox, shared_inbox, featured,
            uri, public_key, private_key, token, password_hash, email,
            totp_secret, totp_verified
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10,
            $11, $12,
            $13, $14, $15, $16, $17,
            $18, $19, $20, $21,
            $22, $23,
            $24, $25, $26, $27,
            $28, $29, $30, $31, $32, $33,
            $34, $35
        )
        RETURNING {ACTOR_COLUMNS}"
    ))
    .bind(actor.id.to_string())
    .bind(actor.created_at)
    .bind(actor.updated_at)
    .bind(&actor.username)
    .bind(&actor.username_lower)
    .bind(&actor.name)
    .bind(&actor.bio)
    .bind(actor.followers_count)
    .bind(actor.following_count)
    .bind(actor.notes_count)
    .bind(&actor.avatar_url)
    .bind(&actor.banner_url)
    .bind(actor.is_suspended)
    .bind(actor.is_locked)
    .bind(actor.is_bot)
    .bind(actor.is_cat)
    .bind(actor.is_admin)
    .bind(&actor.location)
    .bind(&actor.birthday)
    .bind(&actor.lang)
    .bind(sqlx::types::Json(&actor.fields))
    .bind(&actor.followed_message)
    .bind(&actor.reaction_acceptance)
    .bind(&actor.host)
    .bind(&actor.inbox)
    .bind(&actor.shared_inbox)
    .bind(&actor.featured)
    .bind(&actor.uri)
    .bind(&actor.public_key)
    .bind(&actor.private_key)
    .bind(&actor.token)
    .bind(&actor.password_hash)
    .bind(&actor.email)
    .bind(&actor.totp_secret)
    .bind(actor.totp_verified)
    .fetch_one(db)
    .await?;

    Actor::try_from(row)
}

pub async fn get_actor_by_id(db: &Db, id: &ActorId) -> anyhow::Result<Option<Actor>> {
    let row =
        sqlx::query_as::<_, ActorRow>(&format!("SELECT {ACTOR_COLUMNS} FROM users WHERE id = $1"))
            .bind(id.to_string())
            .fetch_optional(db)
            .await?;

    row.map(Actor::try_from).transpose()
}

/// URI が一致するアクターを 1 件返す (リモートアクター解決の共通実装)
pub async fn get_actor_by_uri(db: &Db, uri: &str) -> anyhow::Result<Option<Actor>> {
    let row = sqlx::query_as::<_, ActorRow>(&format!(
        "SELECT {ACTOR_COLUMNS} FROM users WHERE uri = $1 LIMIT 1"
    ))
    .bind(uri)
    .fetch_optional(db)
    .await?;

    row.map(Actor::try_from).transpose()
}

pub async fn get_actors_by_ids(db: &Db, ids: &[String]) -> anyhow::Result<Vec<Actor>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query_as::<_, ActorRow>(&format!(
        "SELECT {ACTOR_COLUMNS} FROM users WHERE id = ANY($1)"
    ))
    .bind(ids)
    .fetch_all(db)
    .await?;

    rows_to_actors(rows)
}

pub async fn get_actor_by_username(db: &Db, username: &str) -> anyhow::Result<Option<Actor>> {
    let username_lower = username.to_lowercase();
    let row = sqlx::query_as::<_, ActorRow>(&format!(
        "SELECT {ACTOR_COLUMNS} FROM users WHERE username_lower = $1 AND host IS NULL LIMIT 1"
    ))
    .bind(username_lower)
    .fetch_optional(db)
    .await?;

    row.map(Actor::try_from).transpose()
}

pub async fn get_actor_by_username_or_email(
    db: &Db,
    username_or_email: &str,
) -> anyhow::Result<Option<Actor>> {
    let identifier_lower = username_or_email.to_lowercase();
    let row = sqlx::query_as::<_, ActorRow>(&format!(
        "SELECT {ACTOR_COLUMNS} FROM users
         WHERE (username_lower = $1 OR email = $1) AND host IS NULL LIMIT 1"
    ))
    .bind(identifier_lower)
    .fetch_optional(db)
    .await?;

    row.map(Actor::try_from).transpose()
}

/// ローカルユーザー名一覧でアクターを一括解決する (メンション通知用)
pub async fn get_actors_by_username_lowers(
    db: &Db,
    username_lowers: &[String],
) -> anyhow::Result<Vec<Actor>> {
    if username_lowers.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query_as::<_, ActorRow>(&format!(
        "SELECT {ACTOR_COLUMNS} FROM users
         WHERE username_lower = ANY($1) AND host IS NULL"
    ))
    .bind(username_lowers)
    .fetch_all(db)
    .await?;

    rows_to_actors(rows)
}

pub async fn update_actor_token(
    db: &Db,
    id: &ActorId,
    token: Option<String>,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE users SET token = $2 WHERE id = $1")
        .bind(id.to_string())
        .bind(token)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn enable_totp(db: &Db, id: &ActorId, secret: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE users SET totp_secret = $2, totp_verified = true WHERE id = $1")
        .bind(id.to_string())
        .bind(secret)
        .execute(db)
        .await?;
    Ok(())
}
