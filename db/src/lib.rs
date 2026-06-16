pub mod cache;
pub mod dragonfly;
pub mod queries;
pub mod storage;
pub mod surreal;

pub use dragonfly::create_client as create_dragonfly_client;
pub use storage::create_storage_client;
pub use surreal::{
    SurrealConfig, create_client as create_surreal_client, create_pool, init_schema,
};

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use redis::aio::ConnectionManager;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB クライアントプール。
///
/// SurrealDB の WebSocket 接続は単一コネクション上でリクエストを多重化するため、
/// 高負荷時に行列ができる。複数コネクションをラウンドロビンで使い分けて
/// スループットを稼ぐ (TODO Phase 0)。
#[derive(Debug, Clone)]
pub struct SurrealClient {
    connections: Arc<Vec<Surreal<Any>>>,
    cursor: Arc<AtomicUsize>,
}

impl SurrealClient {
    pub fn new(connections: Vec<Surreal<Any>>) -> Self {
        assert!(
            !connections.is_empty(),
            "SurrealClient requires at least one connection"
        );
        Self {
            connections: Arc::new(connections),
            cursor: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn single(connection: Surreal<Any>) -> Self {
        Self::new(vec![connection])
    }

    /// ラウンドロビンで次のコネクションを返す
    pub fn get(&self) -> &Surreal<Any> {
        let index = self.cursor.fetch_add(1, Ordering::Relaxed) % self.connections.len();
        &self.connections[index]
    }

    pub fn pool_size(&self) -> usize {
        self.connections.len()
    }
}

impl std::ops::Deref for SurrealClient {
    type Target = Surreal<Any>;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

/// Dragonfly クライアントラッパー。
///
/// `ConnectionManager` は切断時に自動再接続する。BRPOP のようなブロッキング
/// コマンドは多重化コネクションを長時間占有するため、`dedicated_connection()`
/// で専用コネクションを払い出す (TODO Phase 0)。
#[derive(Clone)]
pub struct DragonflyClient {
    manager: ConnectionManager,
    client: redis::Client,
}

impl std::fmt::Debug for DragonflyClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DragonflyClient").finish_non_exhaustive()
    }
}

impl DragonflyClient {
    pub fn new(manager: ConnectionManager, client: redis::Client) -> Self {
        Self { manager, client }
    }

    /// ブロッキングコマンド (BRPOP 等) 用の専用コネクションを払い出す
    pub fn dedicated_connection(&self) -> anyhow::Result<redis::Connection> {
        Ok(self.client.get_connection()?)
    }

    pub fn manager(&self) -> ConnectionManager {
        self.manager.clone()
    }
}

impl redis::aio::ConnectionLike for DragonflyClient {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        self.manager.req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.manager.req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.manager.get_db()
    }
}
