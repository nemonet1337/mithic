pub mod actors;
pub mod notes;
pub mod timeline;
pub mod reactions;
pub mod follows;
pub mod notifications;
pub mod favorites;
pub mod drive;

pub use actors::{create_actor, get_actor_by_id, get_actor_by_username, update_actor_token};
pub use notes::{create_note, delete_note, get_note_by_id};
pub use timeline::{get_global_timeline, get_local_timeline, get_home_timeline};
pub use reactions::{add_reaction, remove_reaction};
pub use follows::{follow_user, unfollow_user, block_user, unblock_user, mute_user, unmute_user, is_following, is_blocking, is_muting, get_following, get_followers};
pub use notifications::{create_notification, get_notifications, mark_notification_as_read, mark_all_notifications_as_read};
pub use favorites::{add_favorite, remove_favorite, is_favorited};
pub use drive::{create_drive_file, get_drive_file, get_user_drive_files, delete_drive_file};

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
