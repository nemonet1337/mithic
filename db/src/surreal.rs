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

impl From<&mithic_config::AppConfig> for SurrealConfig {
    fn from(c: &mithic_config::AppConfig) -> Self {
        Self {
            endpoint: c.surrealdb_endpoint.clone(),
            namespace: c.surrealdb_namespace.clone(),
            database: c.surrealdb_database.clone(),
            username: c.surrealdb_username.clone(),
            password: c.surrealdb_password.clone(),
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
    // Drop speculative / superseded objects. IF EXISTS keeps a fresh DB quiet.
    client
        .query(
            "
        REMOVE TABLE IF EXISTS word_mute;
        REMOVE TABLE IF EXISTS chart;
        REMOVE TABLE IF EXISTS meta;
        REMOVE TABLE IF EXISTS hashtag;
        REMOVE INDEX IF EXISTS idx_username_lower ON TABLE user;
        REMOVE INDEX IF EXISTS idx_note_created_at ON TABLE note;
        REMOVE INDEX IF EXISTS idx_note_visibility_created ON TABLE note;
        REMOVE INDEX IF EXISTS idx_note_host ON TABLE note;
        REMOVE INDEX IF EXISTS idx_follow_in ON TABLE follow;
        REMOVE INDEX IF EXISTS idx_follow_out ON TABLE follow;
        REMOVE FIELD IF EXISTS inbox ON TABLE follow;
        REMOVE FIELD IF EXISTS shared_inbox ON TABLE follow;
        REMOVE INDEX IF EXISTS idx_notification_user ON TABLE notification;
        REMOVE INDEX IF EXISTS idx_poll_expires ON TABLE poll;
        REMOVE INDEX IF EXISTS idx_block_in ON TABLE block;
        REMOVE INDEX IF EXISTS idx_block_out ON TABLE block;
        REMOVE INDEX IF EXISTS idx_mute_in ON TABLE mute;
        REMOVE INDEX IF EXISTS idx_mute_out ON TABLE mute;
        REMOVE INDEX IF EXISTS idx_note_reaction_note ON TABLE note_reaction;
        REMOVE INDEX IF EXISTS idx_note_reaction_actor ON TABLE note_reaction;
        REMOVE INDEX IF EXISTS idx_bookmark_user ON TABLE bookmark;
        REMOVE INDEX IF EXISTS idx_bookmark_note ON TABLE bookmark;
        REMOVE INDEX IF EXISTS idx_activity_type ON TABLE activity;
        REMOVE INDEX IF EXISTS idx_user_note_pining_user ON TABLE user_note_pining;
        REMOVE INDEX IF EXISTS idx_user_note_pining_note ON TABLE user_note_pining;
        ",
        )
        .await?;

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
        DEFINE FIELD IF NOT EXISTS is_cat ON user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS is_admin ON user TYPE bool DEFAULT false;
        DEFINE FIELD IF NOT EXISTS location ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS birthday ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS lang ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS fields ON user TYPE array FLEXIBLE DEFAULT [];
        DEFINE FIELD IF NOT EXISTS followed_message ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS reaction_acceptance ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS host ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS token ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS inbox ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS shared_inbox ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS public_key ON user TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS private_key ON user TYPE option<string>;
        DEFINE INDEX IF NOT EXISTS idx_username_host ON user FIELDS username_lower, host UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_host ON user FIELDS host;
        DEFINE INDEX IF NOT EXISTS idx_user_uri ON user FIELDS uri UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_email ON user FIELDS email;
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
        DEFINE INDEX IF NOT EXISTS idx_note_actor_id ON note FIELDS actor_id;
        DEFINE INDEX IF NOT EXISTS idx_note_visibility_host_id ON note FIELDS visibility, host, id;
        DEFINE INDEX IF NOT EXISTS idx_note_renote_id ON note FIELDS renote_id;
        DEFINE INDEX IF NOT EXISTS idx_note_reply_id ON note FIELDS reply_id;
        DEFINE INDEX IF NOT EXISTS idx_note_uri ON note FIELDS uri;
        DEFINE INDEX IF NOT EXISTS idx_note_tags ON note FIELDS tags.*;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS follow TYPE RELATION IN user OUT user;
        DEFINE FIELD IF NOT EXISTS created_at ON follow TYPE datetime;
        DEFINE FIELD IF NOT EXISTS is_accepted ON follow TYPE bool DEFAULT true;
        DEFINE INDEX IF NOT EXISTS idx_follow_in_out ON follow FIELDS in, out UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_follow_out_in ON follow FIELDS out, in;
    ",
        )
        .await?;

    // block / mute は後段の IN user OUT user + インデックス定義のみ使う (二重定義を避ける)

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
        DEFINE INDEX IF NOT EXISTS idx_drive_user_created ON drive_file FIELDS user_id, created_at;
        DEFINE INDEX IF NOT EXISTS idx_drive_md5 ON drive_file FIELDS md5;
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
        DEFINE INDEX IF NOT EXISTS idx_notification_read ON notification FIELDS user_id, is_read;
        DEFINE INDEX IF NOT EXISTS idx_notification_user_created ON notification FIELDS user_id, created_at;
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
        DEFINE INDEX IF NOT EXISTS idx_poll_note_id ON poll FIELDS note_id;
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
        DEFINE INDEX IF NOT EXISTS idx_poll_vote_unique ON poll_vote FIELDS poll_id, actor_id, choice_index UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS block TYPE RELATION IN user OUT user;
        DEFINE FIELD IF NOT EXISTS created_at ON block TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_block_in_out ON block FIELDS in, out UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS mute TYPE RELATION IN user OUT user;
        DEFINE FIELD IF NOT EXISTS created_at ON mute TYPE datetime;
        DEFINE FIELD IF NOT EXISTS expires_at ON mute TYPE option<datetime>;
        DEFINE INDEX IF NOT EXISTS idx_mute_in_out ON mute FIELDS in, out UNIQUE;
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
        DEFINE INDEX IF NOT EXISTS idx_note_reaction_unique ON note_reaction FIELDS note_id, actor_id UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS push_subscription SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON push_subscription TYPE string;
        DEFINE FIELD IF NOT EXISTS user_id ON push_subscription TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS endpoint ON push_subscription TYPE string;
        DEFINE FIELD IF NOT EXISTS p256dh ON push_subscription TYPE string;
        DEFINE FIELD IF NOT EXISTS auth ON push_subscription TYPE string;
        DEFINE FIELD IF NOT EXISTS created_at ON push_subscription TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_push_user ON push_subscription FIELDS user_id;
        DEFINE INDEX IF NOT EXISTS idx_push_endpoint ON push_subscription FIELDS endpoint UNIQUE;
    ",
        )
        .await?;

    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS user_note_pining TYPE RELATION IN user OUT note;
        DEFINE FIELD IF NOT EXISTS created_at ON user_note_pining TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_user_note_pining_unique ON user_note_pining FIELDS in, out UNIQUE;
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
        DEFINE INDEX IF NOT EXISTS idx_bookmark_unique ON bookmark FIELDS user_id, note_id UNIQUE;
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
        DEFINE INDEX IF NOT EXISTS idx_relay_inbox ON relay FIELDS inbox UNIQUE;
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
        DEFINE INDEX IF NOT EXISTS idx_activity_uri ON activity FIELDS uri UNIQUE;
    ",
        )
        .await?;

    // リモートカスタム絵文字キャッシュ (連合受信)
    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS remote_emoji SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON remote_emoji TYPE string;
        DEFINE FIELD IF NOT EXISTS name ON remote_emoji TYPE string;
        DEFINE FIELD IF NOT EXISTS url ON remote_emoji TYPE string;
        DEFINE FIELD IF NOT EXISTS host ON remote_emoji TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS created_at ON remote_emoji TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_remote_emoji_name_host ON remote_emoji FIELDS name, host UNIQUE;
    ",
        )
        .await?;

    // ローカル / 公開カスタム絵文字
    client
        .query(
            "
        DEFINE TABLE IF NOT EXISTS emoji SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS id ON emoji TYPE string;
        DEFINE FIELD IF NOT EXISTS name ON emoji TYPE string;
        DEFINE FIELD IF NOT EXISTS url ON emoji TYPE string;
        DEFINE FIELD IF NOT EXISTS category ON emoji TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS aliases ON emoji TYPE array<string> DEFAULT [];
        DEFINE FIELD IF NOT EXISTS is_public ON emoji TYPE bool DEFAULT true;
        DEFINE FIELD IF NOT EXISTS created_at ON emoji TYPE datetime;
        DEFINE INDEX IF NOT EXISTS idx_emoji_name ON emoji FIELDS name UNIQUE;
    ",
        )
        .await?;

    // 既存リモートノート: SCHEMAFULL で落ちていた host を actor から一度だけ埋める
    client
        .query("UPDATE note SET host = actor_id.host WHERE host = NONE AND actor_id.host != NONE;")
        .await?;

    Ok(())
}
