pub mod dragonfly;
pub mod surreal;
pub mod queries;

pub use dragonfly::create_client as create_dragonfly_client;
pub use surreal::{create_client as create_surreal_client, DbClient, SurrealConfig};

use redis::aio::MultiplexedConnection;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;

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
