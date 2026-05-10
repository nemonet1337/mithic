use std::net::SocketAddr;

use axum::Router;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod db;
mod error;
mod i18n;
mod mfm;
mod misc;
mod models;
mod routes;
mod services;
mod state;
mod stream;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 環境変数読み込み
    dotenvy::dotenv().ok();

    // ロガー初期化
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Mithic API server...");

    // 設定読み込み
    let config = AppConfig::from_env()?;
    info!("Configuration loaded");

    // SurrealDB接続
    let surreal_config = db::surreal::SurrealConfig {
        endpoint: config.surrealdb_endpoint.clone(),
        namespace: config.surrealdb_namespace.clone(),
        database: config.surrealdb_database.clone(),
        username: config.surrealdb_username.clone(),
        password: config.surrealdb_password.clone(),
    };

    let surreal_client = db::surreal::create_client(&surreal_config).await?;
    info!("SurrealDB client connected");

    // Dragonfly接続
    let dragonfly_client = db::dragonfly::create_client(&config.dragonfly_url).await?;
    info!("Dragonfly client connected");

    // スキーマ初期化
    db::surreal::init_schema(&surreal_client).await?;
    info!("SurrealDB schema initialized");

    // アプリケーション状態構築
    let state = AppState::new(surreal_client, dragonfly_client, config)?;

    // ルーター構築
    let app = routes::create_router(state);

    // サーバー起動
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
