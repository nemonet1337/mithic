pub mod cache;
pub mod queries;
pub mod redis;
pub mod storage;
pub mod surreal;

pub use self::redis::{RedisClient, create_client as create_redis_client};
pub use storage::create_storage_client;
pub use surreal::{
    SurrealConfig, create_client as create_surreal_client, create_pool, init_schema,
};

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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
