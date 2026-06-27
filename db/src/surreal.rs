use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Root;

/// SurrealDB client type
pub type DbClient = Surreal<Any>;

/// SurrealDB config
#[derive(Debug, Clone)]
pub struct SurrealConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl Default for SurrealConfig {
    fn default() -> Self {
        Self {
            endpoint: "ws://localhost:8000".to_string(),
            namespace: "mithic".to_string(),
            database: "main".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
        }
    }
}

/// Create and connect SurrealDB client
pub async fn create_client(config: &SurrealConfig) -> anyhow::Result<DbClient> {
    let client = connect(&config.endpoint).await?;

    if !config.endpoint.starts_with("mem") {
        client
            .signin(Root {
                username: config.username.clone(),
                password: config.password.clone(),
            })
            .await?;
    }

    client
        .use_ns(&config.namespace)
        .use_db(&config.database)
        .await?;

    Ok(client)
}

/// Create connection pool of specified size
pub async fn create_pool(
    config: &SurrealConfig,
    size: usize,
) -> anyhow::Result<crate::SurrealClient> {
    let size = size.max(1);
    let mut connections = Vec::with_capacity(size);
    for _ in 0..size {
        connections.push(create_client(config).await?);
    }
    Ok(crate::SurrealClient::new(connections))
}

/// Table initialization (schema definition)
pub async fn init_schema(client: &DbClient) -> anyhow::Result<()> {
    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON user TYPE string;
        DEFINE FIELD IF NOT EXISTS username ON user TYPE string;
        DEFINE FIELD IF NOT EXISTS username_lower ON user TYPE string;
        DEFINE FIELD IF NOT EXISTS name ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS bio ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS uri ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS featured ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS password_hash ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS email ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS totp_secret ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS totp_verified ON user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS created_at ON user TYPE datetime;
        DEFINE FIELD IF NOT EXISTS updated_at ON user TYPE option<datetime>;
        DEFINE FIELD IF NOT EXISTS followers_count ON user TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS following_count ON user TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS notes_count ON user TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS avatar_url ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS banner_url ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS is_suspended ON user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS is_locked ON user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS is_bot ON user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS is_admin ON user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS host ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS token ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS inbox ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS shared_inbox ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS public_key ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS private_key ON user TYPE option<string>;
        DEFINE INDEX IF NOT EXISTS idx_username_lower ON user COLUMNS username_lower UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_host ON user COLUMNS host;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS note SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON note TYPE string;
        DEFINE FIELD IF NOT EXISTS created_at ON note TYPE datetime;
        DEFINE FIELD IF NOT EXISTS text ON note TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS cw ON note TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS actor_id ON note TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS visibility ON note TYPE string DEFAULT 'public';
        DEFINE FIELD IF NOT EXISTS renote_count ON note TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS replies_count ON note TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS reactions ON note TYPE object FLEXIBLE DEFAULT {};
        DEFINE FIELD IF NOT EXISTS reply_id ON note TYPE option<record<note>>;
        DEFINE FIELD IF NOT EXISTS renote_id ON note TYPE option<record<note>>;
        DEFINE FIELD IF NOT EXISTS file_ids ON note TYPE array<string> DEFAULT [];
        DEFINE FIELD IF NOT EXISTS tags ON note TYPE array<string> DEFAULT [];
        DEFINE FIELD IF NOT EXISTS has_poll ON note TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS uri ON note TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS host ON note TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS mentions ON note TYPE array<string> DEFAULT [];
        DEFINE FIELD IF NOT EXISTS emojis ON note TYPE array<string> DEFAULT [];
        DEFINE FIELD IF NOT EXISTS visible_user_ids ON note TYPE array<string> DEFAULT [];
        DEFINE INDEX IF NOT EXISTS idx_note_actor_id ON note COLUMNS actor_id;
        DEFINE INDEX IF NOT EXISTS idx_note_created_at ON note COLUMNS created_at;
        DEFINE INDEX IF NOT EXISTS idx_note_visibility_created ON note COLUMNS visibility, created_at;
        DEFINE INDEX IF NOT EXISTS idx_note_host ON note COLUMNS host;
        DEFINE INDEX IF NOT EXISTS idx_note_visibility_host_id ON note COLUMNS visibility, host, id;
        DEFINE INDEX IF NOT EXISTS idx_note_renote_id ON note COLUMNS renote_id;
        DEFINE INDEX IF NOT EXISTS idx_note_reply_id ON note COLUMNS reply_id;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS follow TYPE RELATION;
        DEFINE FIELD IF NOT EXISTS created_at ON follow TYPE datetime;
        DEFINE FIELD IF NOT EXISTS inbox ON follow TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS shared_inbox ON follow TYPE option<string>;
        DEFINE INDEX IF NOT EXISTS idx_follow_in ON follow COLUMNS in;
        DEFINE INDEX IF NOT EXISTS idx_follow_out ON follow COLUMNS out;
        DEFINE INDEX IF NOT EXISTS idx_follow_out_in ON follow COLUMNS out, in;
        DEFINE INDEX IF NOT EXISTS idx_follow_in_out ON follow COLUMNS in, out;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS block TYPE RELATION;
        DEFINE FIELD IF NOT EXISTS created_at ON block TYPE datetime;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS mute TYPE RELATION;
        DEFINE FIELD IF NOT EXISTS created_at ON mute TYPE datetime;
        DEFINE FIELD IF NOT EXISTS expires_at ON mute TYPE option<datetime>;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS drive_file SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON drive_file TYPE string;
        DEFINE FIELD IF NOT EXISTS user_id ON drive_file TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS name ON drive_file TYPE string;
        DEFINE FIELD IF NOT EXISTS mime_type ON drive_file TYPE string;
        DEFINE FIELD IF NOT EXISTS size ON drive_file TYPE int;
        DEFINE FIELD IF NOT EXISTS md5 ON drive_file TYPE string;
        DEFINE FIELD IF NOT EXISTS url ON drive_file TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS thumbnail_url ON drive_file TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS is_sensitive ON drive_file TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS created_at ON drive_file TYPE datetime;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS notification SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON notification TYPE string;
        DEFINE FIELD IF NOT EXISTS created_at ON notification TYPE datetime;
        DEFINE FIELD IF NOT EXISTS user_id ON notification TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS notification_type ON notification TYPE string;
        DEFINE FIELD IF NOT EXISTS notifier_id ON notification TYPE option<record<user>>;
        DEFINE FIELD IF NOT EXISTS note_id ON notification TYPE option<record<note>>;
        DEFINE FIELD IF NOT EXISTS reaction ON notification TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS is_read ON notification TYPE bool DEFAULT false;
        DEFINE INDEX IF NOT EXISTS idx_notification_user ON notification COLUMNS user_id;
        DEFINE INDEX IF NOT EXISTS idx_notification_read ON notification COLUMNS user_id, is_read;
        DEFINE INDEX IF NOT EXISTS idx_notification_user_created ON notification COLUMNS user_id, created_at;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS poll SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON poll TYPE string;
        DEFINE FIELD IF NOT EXISTS note_id ON poll TYPE record<note>;
        DEFINE FIELD IF NOT EXISTS created_at ON poll TYPE datetime;
        DEFINE FIELD IF NOT EXISTS expires_at ON poll TYPE option<datetime>;
        DEFINE FIELD IF NOT EXISTS multiple ON poll TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS is_archived ON poll TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS choices ON poll TYPE array<object>;
        DEFINE INDEX IF NOT EXISTS idx_poll_note_id ON poll COLUMNS note_id;
        DEFINE INDEX IF NOT EXISTS idx_poll_expires ON poll COLUMNS expires_at;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS poll_vote SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON poll_vote TYPE string;
        DEFINE FIELD IF NOT EXISTS poll_id ON poll_vote TYPE record<poll>;
        DEFINE FIELD IF NOT EXISTS actor_id ON poll_vote TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS choice_index ON poll_vote TYPE int;
        DEFINE FIELD IF NOT EXISTS created_at ON poll_vote TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_poll_vote_poll ON poll_vote COLUMNS poll_id;
        DEFINE INDEX IF NOT EXISTS idx_poll_vote_actor ON poll_vote COLUMNS actor_id;
        DEFINE INDEX IF NOT EXISTS idx_poll_vote_unique ON poll_vote COLUMNS poll_id, actor_id, choice_index UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS block TYPE RELATION IN user OUT user;
        DEFINE FIELD IF NOT EXISTS created_at ON block TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_block_in ON block COLUMNS in;
        DEFINE INDEX IF NOT EXISTS idx_block_out ON block COLUMNS out;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS mute TYPE RELATION IN user OUT user;
        DEFINE FIELD IF NOT EXISTS created_at ON mute TYPE datetime;
        DEFINE FIELD IF NOT EXISTS expires_at ON mute TYPE option<datetime>;
        DEFINE INDEX IF NOT EXISTS idx_mute_in ON mute COLUMNS in;
        DEFINE INDEX IF NOT EXISTS idx_mute_out ON mute COLUMNS out;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS hashtag SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON hashtag TYPE string;
        DEFINE FIELD IF NOT EXISTS name ON hashtag TYPE string;
        DEFINE FIELD IF NOT EXISTS count ON hashtag TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS created_at ON hashtag TYPE datetime;
        DEFINE FIELD IF NOT EXISTS updated_at ON hashtag TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_hashtag_name ON hashtag COLUMNS name UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS meta SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON meta TYPE string;
        DEFINE FIELD IF NOT EXISTS description ON meta TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS blocked_hosts ON meta TYPE array<string> DEFAULT [];
        DEFINE FIELD IF NOT EXISTS cache_remote_files ON meta TYPE bool DEFAULT true;
        DEFINE FIELD IF NOT EXISTS remote_drive_capacity_mb ON meta TYPE int DEFAULT 32;

        DEFINE TABLE IF NOT EXISTS word_mute SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON word_mute TYPE string;
        DEFINE FIELD IF NOT EXISTS user_id ON word_mute TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS pattern ON word_mute TYPE string;
        DEFINE FIELD IF NOT EXISTS created_at ON word_mute TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_word_mute_user ON word_mute COLUMNS user_id;
        DEFINE INDEX IF NOT EXISTS idx_word_mute_user_pattern ON word_mute COLUMNS user_id, pattern UNIQUE;

        DEFINE TABLE IF NOT EXISTS chart SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON chart TYPE string;
        DEFINE FIELD IF NOT EXISTS kind ON chart TYPE string;
        DEFINE FIELD IF NOT EXISTS timestamp ON chart TYPE datetime;
        DEFINE FIELD IF NOT EXISTS value ON chart TYPE int;
        DEFINE INDEX IF NOT EXISTS idx_chart_kind_time ON chart COLUMNS kind, timestamp;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS note_reaction SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON note_reaction TYPE string;
        DEFINE FIELD IF NOT EXISTS note_id ON note_reaction TYPE record<note>;
        DEFINE FIELD IF NOT EXISTS actor_id ON note_reaction TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS reaction ON note_reaction TYPE string;
        DEFINE FIELD IF NOT EXISTS created_at ON note_reaction TYPE datetime;
        DEFINE FIELD IF NOT EXISTS is_remote ON note_reaction TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS uri ON note_reaction TYPE option<string>;
        DEFINE INDEX IF NOT EXISTS idx_note_reaction_note ON note_reaction COLUMNS note_id;
        DEFINE INDEX IF NOT EXISTS idx_note_reaction_actor ON note_reaction COLUMNS actor_id;
        DEFINE INDEX IF NOT EXISTS idx_note_reaction_unique ON note_reaction COLUMNS note_id, actor_id UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS user_note_pining TYPE RELATION;
        DEFINE FIELD IF NOT EXISTS created_at ON user_note_pining TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_user_note_pining_user ON user_note_pining COLUMNS in;
        DEFINE INDEX IF NOT EXISTS idx_user_note_pining_note ON user_note_pining COLUMNS out;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS bookmark SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON bookmark TYPE string;
        DEFINE FIELD IF NOT EXISTS user_id ON bookmark TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS note_id ON bookmark TYPE record<note>;
        DEFINE FIELD IF NOT EXISTS created_at ON bookmark TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_bookmark_user ON bookmark COLUMNS user_id;
        DEFINE INDEX IF NOT EXISTS idx_bookmark_note ON bookmark COLUMNS note_id;
        DEFINE INDEX IF NOT EXISTS idx_bookmark_unique ON bookmark COLUMNS user_id, note_id UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS relay SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON relay TYPE string;
        DEFINE FIELD IF NOT EXISTS inbox ON relay TYPE string;
        DEFINE FIELD IF NOT EXISTS status ON relay TYPE string DEFAULT 'requesting';
        DEFINE FIELD IF NOT EXISTS created_at ON relay TYPE datetime;
        DEFINE FIELD IF NOT EXISTS updated_at ON relay TYPE option<datetime>;
        DEFINE INDEX IF NOT EXISTS idx_relay_inbox ON relay COLUMNS inbox UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS activity SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON activity TYPE string;
        DEFINE FIELD IF NOT EXISTS uri ON activity TYPE string;
        DEFINE FIELD IF NOT EXISTS activity_type ON activity TYPE string;
        DEFINE FIELD IF NOT EXISTS actor_id ON activity TYPE option<record<user>>;
        DEFINE FIELD IF NOT EXISTS note_id ON activity TYPE option<record<note>>;
        DEFINE FIELD IF NOT EXISTS created_at ON activity TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_activity_uri ON activity COLUMNS uri UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_activity_type ON activity COLUMNS activity_type;
    ",
        )
        .await?;

    Ok(())
}
