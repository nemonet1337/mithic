pub mod actors;
pub mod drive;
pub mod favorites;
pub mod follows;
pub mod notes;
pub mod notifications;
pub mod reactions;
pub mod timeline;

pub use actors::{create_actor, get_actor_by_id, get_actor_by_username, update_actor_token};
pub use drive::{create_drive_file, delete_drive_file, get_drive_file, get_user_drive_files};
pub use favorites::{add_favorite, is_favorited, remove_favorite};
pub use follows::{
    block_user, follow_user, get_followers, get_following, is_blocking, is_following, is_muting,
    mute_user, unblock_user, unfollow_user, unmute_user,
};
pub use notes::{create_note, delete_note, get_note_by_id};
pub use notifications::{
    create_notification, get_notifications, mark_all_notifications_as_read,
    mark_notification_as_read,
};
pub use reactions::{add_reaction, remove_reaction};
pub use timeline::{get_global_timeline, get_home_timeline, get_local_timeline};

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
