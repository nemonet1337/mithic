use redis::Client;
use redis::aio::ConnectionManager;

use crate::DragonflyClient;

/// Dragonflyクライアントを作成・接続。
/// `ConnectionManager` を使うことで切断時に自動再接続する。
pub async fn create_client(url: &str) -> anyhow::Result<DragonflyClient> {
    let client = Client::open(url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(DragonflyClient::new(manager))
}

/// レート制限キー
pub fn rate_limit_key(identifier: &str, action: &str) -> String {
    format!("rate_limit:{identifier}:{action}")
}
