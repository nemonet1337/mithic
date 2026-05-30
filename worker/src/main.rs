//! Mithic Worker
//!
//! バックグラウンドワーカープロセス。
//! フェデレーションキューの処理、メディア処理、クリーンアップ等を担当する。

use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 環境変数読み込み
    dotenvy::dotenv().ok();

    // ロガー初期化
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Mithic Worker...");

    let config = mithic_config::AppConfig::from_env()?;

    info!("Connecting to SurrealDB at {}", config.surrealdb_endpoint);
    let surreal = mithic_db::create_surreal_client(&mithic_db::SurrealConfig {
        endpoint: config.surrealdb_endpoint.clone(),
        namespace: config.surrealdb_namespace.clone(),
        database: config.surrealdb_database.clone(),
        username: config.surrealdb_username.clone(),
        password: config.surrealdb_password.clone(),
    })
    .await?;

    info!("Initializing SurrealDB schema");
    mithic_db::init_schema(&surreal).await?;

    info!("Connecting to Dragonfly at {}", config.dragonfly_url);
    let dragonfly = mithic_db::create_dragonfly_client(&config.dragonfly_url).await?;

    let surreal_client = mithic_db::SurrealClient(surreal);
    let dragonfly_client = mithic_db::DragonflyClient(dragonfly);

    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(32)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .build()
        .unwrap_or_default();

    let federation_service = mithic_federation::FederationService::new(
        surreal_client,
        dragonfly_client,
        http_client,
        config.instance_url.clone(),
    );

    // フェデレーション配送キュー処理を非同期タスクとして起動
    let fed_service_clone = federation_service.clone();
    tokio::spawn(async move {
        if let Err(e) = fed_service_clone.run_delivery_worker().await {
            tracing::error!("Federation delivery worker failed: {}", e);
        }
    });

    info!("Worker started. Waiting for jobs...");

    // メインループ（シグナルを待つ）
    tokio::signal::ctrl_c().await?;
    info!("Worker shutting down...");

    Ok(())
}
