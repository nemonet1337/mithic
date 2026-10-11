//! PostgreSQL コネクションプールとマイグレーション適用。

use sqlx::migrate::Migrator;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub type Db = PgPool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// プールを作成し、起動時にマイグレーションを適用する。
/// `migrate!` は `backend/` 基準の相対パスで解決する。
pub async fn create_pool(url: &str, max: u32) -> anyhow::Result<Db> {
    let pool = PgPoolOptions::new()
        .max_connections(max.max(1))
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(url)
        .await?;

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}
