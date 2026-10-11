use redis::aio::{ConnectionLike, ConnectionManager};
use redis::{Client, Cmd, Pipeline, RedisFuture, Value};

/// Redis クライアントラッパー。
///
/// `ConnectionManager` は切断時に自動再接続する。
#[derive(Clone)]
pub struct RedisClient {
    manager: ConnectionManager,
}

impl std::fmt::Debug for RedisClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisClient").finish_non_exhaustive()
    }
}

impl RedisClient {
    pub fn new(manager: ConnectionManager) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> ConnectionManager {
        self.manager.clone()
    }
}

impl ConnectionLike for RedisClient {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a Cmd) -> RedisFuture<'a, Value> {
        self.manager.req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a Pipeline,
        offset: usize,
        count: usize,
    ) -> RedisFuture<'a, Vec<Value>> {
        self.manager.req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.manager.get_db()
    }
}

/// Redis クライアントを作成・接続。
/// `ConnectionManager` を使うことで切断時に自動再接続する。
pub async fn create_client(url: &str) -> anyhow::Result<RedisClient> {
    let client = Client::open(url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(RedisClient::new(manager))
}

/// レート制限キー
pub fn rate_limit_key(identifier: &str, action: &str) -> String {
    format!("rate_limit:{identifier}:{action}")
}
