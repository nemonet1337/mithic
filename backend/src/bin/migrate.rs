//! PostgreSQL -> SurrealDB Migration Tool
//! 
//! Usage:
//!   cargo run --bin migrate -- --source postgres://user:pass@localhost/mithic --target ws://localhost:8000

use std::collections::HashMap;
use std::time::Duration;

use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Migration CLI arguments
#[derive(Parser, Debug)]
#[command(name = "mithic-migrate")]
#[command(about = "Migrate data from PostgreSQL to SurrealDB")]
struct Args {
    /// PostgreSQL connection string
    #[arg(short, long, env = "POSTGRES_URL")]
    source: String,

    /// SurrealDB connection string
    #[arg(short, long, env = "SURREAL_URL", default_value = "ws://localhost:8000")]
    target: String,

    /// SurrealDB namespace
    #[arg(long, env = "SURREAL_NS", default_value = "mithic")]
    namespace: String,

    /// SurrealDB database
    #[arg(long, env = "SURREAL_DB", default_value = "main")]
    database: String,

    /// SurrealDB username
    #[arg(short, long, env = "SURREAL_USER", default_value = "root")]
    username: String,

    /// SurrealDB password
    #[arg(short = 'p', long, env = "SURREAL_PASS", default_value = "root")]
    password: String,

    /// Batch size for migration
    #[arg(short, long, default_value = "1000")]
    batch_size: usize,

    /// Specific table to migrate (if not specified, migrates all)
    #[arg(long)]
    table: Option<String>,

    /// Dry run - don't actually insert data
    #[arg(long)]
    dry_run: bool,
}

/// PostgreSQL row wrapper
#[derive(Debug, Serialize, Deserialize)]
struct PgRow {
    table: String,
    data: HashMap<String, Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Starting migration from PostgreSQL to SurrealDB");
    info!("Source: {}", args.source);
    info!("Target: {}", args.target);

    if args.dry_run {
        warn!("DRY RUN MODE - No data will be written");
    }

    // Connect to PostgreSQL
    let pg_pool = sqlx::PgPool::connect(&args.source).await?;
    info!("Connected to PostgreSQL");

    // Connect to SurrealDB
    let surreal_client = connect_surreal(&args).await?;
    info!("Connected to SurrealDB");

    // Get list of tables to migrate
    let tables = if let Some(table) = args.table {
        vec![table]
    } else {
        get_tables(&pg_pool).await?
    };

    info!("Tables to migrate: {:?}", tables);

    // Migrate each table
    for table in tables {
        match migrate_table(&pg_pool, &surreal_client, &table, args.batch_size, args.dry_run).await {
            Ok(count) => info!("Migrated {} rows from {}", count, table),
            Err(e) => error!("Failed to migrate table {}: {}", table, e),
        }
    }

    info!("Migration completed");
    Ok(())
}

/// Connect to SurrealDB
async fn connect_surreal(args: &Args) -> anyhow::Result<surrealdb::Surreal<surrealdb::engine::any::Any>> {
    use surrealdb::opt::auth::Root;

    let client = surrealdb::engine::any::connect(&args.target).await?;
    
    client
        .signin(Root {
            username: &args.username,
            password: &args.password,
        })
        .await?;

    client
        .use_ns(&args.namespace)
        .use_db(&args.database)
        .await?;

    Ok(client)
}

/// Get list of tables from PostgreSQL
async fn get_tables(pool: &sqlx::PgPool) -> anyhow::Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
    )
    .fetch_all(pool)
    .await?;

    let tables: Vec<String> = rows.into_iter().map(|(name,)| name).collect();
    Ok(tables)
}

/// Migrate a single table
async fn migrate_table(
    pg_pool: &sqlx::PgPool,
    surreal: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    table: &str,
    batch_size: usize,
    dry_run: bool,
) -> anyhow::Result<usize> {
    info!("Migrating table: {}", table);

    let mut total_count = 0;
    let mut offset = 0i64;

    loop {
        // Fetch batch from PostgreSQL
        let query = format!(
            "SELECT * FROM \"{}\" ORDER BY id LIMIT {} OFFSET {}",
            table, batch_size, offset
        );

        let rows = sqlx::query(&query)
            .fetch_all(pg_pool)
            .await?;

        if rows.is_empty() {
            break;
        }

        let batch_count = rows.len();
        info!("  Processing batch: {} rows starting at offset {}", batch_count, offset);

        // Transform and insert into SurrealDB
        for row in rows {
            let record = transform_row(table, row)?;
            
            if !dry_run {
                insert_to_surreal(surreal, table, record).await?;
            }
        }

        total_count += batch_count;
        offset += batch_size as i64;

        // Small delay to prevent overwhelming the database
        sleep(Duration::from_millis(10)).await;
    }

    Ok(total_count)
}

/// Transform PostgreSQL row to SurrealDB record
fn transform_row(table: &str, row: sqlx::postgres::PgRow) -> anyhow::Result<serde_json::Value> {
    use sqlx::Column;
    use sqlx::Row;
    use sqlx::TypeInfo;

    let mut data = serde_json::Map::new();

    for (i, column) in row.columns().iter().enumerate() {
        let name = column.name();
        let type_info = column.type_info();
        let type_name = type_info.name();

        // Try to get value based on type
        let value: Value = match type_name {
            "INT4" | "INT8" | "INT2" => {
                row.try_get::<i64, _>(i)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "FLOAT4" | "FLOAT8" => {
                row.try_get::<f64, _>(i)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "BOOL" => {
                row.try_get::<bool, _>(i)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "TIMESTAMP" | "TIMESTAMPTZ" => {
                row.try_get::<chrono::DateTime<chrono::Utc>, _>(i)
                    .map(|v| json!(v.to_rfc3339()))
                    .unwrap_or(json!(null))
            }
            "JSON" | "JSONB" => {
                row.try_get::<Value, _>(i)
                    .unwrap_or(json!(null))
            }
            "TEXT" | "VARCHAR" | "BPCHAR" | "UUID" => {
                row.try_get::<String, _>(i)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
            "BYTEA" => {
                // Convert binary to base64
                row.try_get::<Vec<u8>, _>(i)
                    .map(|v| json!(base64::encode(&v)))
                    .unwrap_or(json!(null))
            }
            _ => {
                // Try as string fallback
                row.try_get::<String, _>(i)
                    .map(|v| json!(v))
                    .unwrap_or(json!(null))
            }
        };

        // Transform field names for compatibility
        let surreal_name = transform_field_name(table, name);
        data.insert(surreal_name, value);
    }

    Ok(Value::Object(data))
}

/// Transform field names for SurrealDB compatibility
fn transform_field_name(table: &str, pg_name: &str) -> String {
    // Map PostgreSQL field names to SurrealDB field names
    let mapping: HashMap<&str, &str> = match table {
        "user" => [
            ("id", "id"),
            ("createdAt", "created_at"),
            ("updatedAt", "updated_at"),
            ("userName", "username"),
            ("displayName", "name"),
            ("passwordHash", "password_hash"),
            ("avatarUrl", "avatar_url"),
            ("bannerUrl", "banner_url"),
            ("isSuspended", "is_suspended"),
            ("isLocked", "is_locked"),
            ("isBot", "is_bot"),
            ("isAdmin", "is_admin"),
            ("followersCount", "followers_count"),
            ("followingCount", "following_count"),
            ("notesCount", "notes_count"),
            ("publicKey", "public_key"),
            ("privateKey", "private_key"),
        ].into(),
        "note" => [
            ("userId", "actor_id"),
            ("replyId", "reply_id"),
            ("renoteId", "renote_id"),
            ("renoteCount", "renote_count"),
            ("repliesCount", "replies_count"),
            ("fileIds", "file_ids"),
            ("visibleUserIds", "visible_user_ids"),
            ("replyUserId", "reply_actor_id"),
            ("renoteUserId", "renote_actor_id"),
            ("userHost", "actor_host"),
        ].into(),
        _ => HashMap::new(),
    };

    mapping.get(pg_name).copied().unwrap_or(pg_name).to_string()
}

/// Insert record into SurrealDB
async fn insert_to_surreal(
    surreal: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    table: &str,
    record: Value,
) -> anyhow::Result<()> {
    // Map table names
    let surreal_table = match table {
        "user" => "user",
        "note" => "note",
        "drive_file" => "drive_file",
        "drive_folder" => "drive_folder",
        "notification" => "notification",
        "reaction" => "reaction",
        "follow_request" => "follow_request",
        "meta" => "meta",
        _ => table,
    };

    // Get ID from record
    let id = record.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Record missing ID"))?;

    // Create record
    let thing = surrealdb::sql::Thing::from((surreal_table, id));
    surreal.create(thing).content(record).await?;

    Ok(())
}
