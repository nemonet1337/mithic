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

    // TODO: フェデレーション配送キュー処理
    // TODO: メディア処理（サムネイル生成等）
    // TODO: 定期クリーンアップジョブ
    // TODO: 期限切れポーリング結果の集計

    info!("Worker started. Waiting for jobs...");

    // メインループ（シグナルを待つ）
    tokio::signal::ctrl_c().await?;
    info!("Worker shutting down...");

    Ok(())
}
