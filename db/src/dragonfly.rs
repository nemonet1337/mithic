use redis::Client;
use redis::aio::MultiplexedConnection;

/// Dragonflyクライアント型（Redis互換）
pub type DragonflyClient = MultiplexedConnection;

/// Dragonflyクライアントを作成・接続
pub async fn create_client(url: &str) -> anyhow::Result<DragonflyClient> {
    let client = Client::open(url)?;
    let connection = client.get_multiplexed_async_connection().await?;
    Ok(connection)
}

/// キャッシュキー生成ヘルパー
pub fn cache_key(prefix: &str, id: &str) -> String {
    format!("{}:{}", prefix, id)
}

/// タイムラインキャッシュキー
pub fn timeline_key(user_id: &str, timeline_type: &str) -> String {
    format!("timeline:{}:{}", user_id, timeline_type)
}

/// レート制限キー
pub fn rate_limit_key(identifier: &str, action: &str) -> String {
    format!("rate_limit:{}:{}", identifier, action)
}
