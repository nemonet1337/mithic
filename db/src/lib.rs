pub mod dragonfly;
pub mod queries;
pub mod surreal;

pub use dragonfly::create_client as create_dragonfly_client;
pub use surreal::{SurrealConfig, create_client as create_surreal_client};

use redis::aio::MultiplexedConnection;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// SurrealDB client wrapper
#[derive(Debug, Clone)]
pub struct SurrealClient(pub Surreal<Any>);

impl std::ops::Deref for SurrealClient {
    type Target = Surreal<Any>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Dragonfly client wrapper
#[derive(Debug, Clone)]
pub struct DragonflyClient(pub MultiplexedConnection);

impl std::ops::Deref for DragonflyClient {
    type Target = MultiplexedConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl redis::aio::ConnectionLike for DragonflyClient {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        self.0.req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.0.req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.0.get_db()
    }
}
