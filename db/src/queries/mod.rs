pub mod actors;
pub mod notes;
pub mod timeline;

pub use actors::{create_actor, get_actor_by_id, get_actor_by_username, update_actor_token};
pub use notes::{create_note, delete_note, get_note_by_id};
pub use timeline::{get_global_timeline, get_local_timeline};

use serde::de::DeserializeOwned;
use surrealdb::types::Value;

/// surrealdb 3.x の `take` は結果型に `SurrealValue` を要求するが、ドメインモデルは
/// serde の `Deserialize` のみ実装している。生の `Value` 行を JSON 経由で serde
/// デシリアライズしてモデルへ変換する。
pub(crate) fn rows_to<T: DeserializeOwned>(rows: Vec<Value>) -> anyhow::Result<Vec<T>> {
    rows.into_iter()
        .map(|value| {
            serde_json::from_value::<T>(value.into_json_value()).map_err(anyhow::Error::from)
        })
        .collect()
}
