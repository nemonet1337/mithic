pub mod activity;
pub mod actors;
pub mod drive;
pub mod favorites;
pub mod follows;
pub mod hashtags;
pub mod notes;
pub mod notifications;
pub mod polls;
pub mod reactions;
pub mod relay;
pub mod timeline;

pub use activity::{create_activity, get_activity_by_uri};
pub use actors::{
    create_actor, get_actor_by_id, get_actor_by_username, get_actor_by_username_or_email,
    update_actor_token, enable_totp,
};
pub use drive::{
    create_drive_file, delete_drive_file, get_drive_file, get_drive_file_by_hash,
    get_user_drive_files,
};
pub use favorites::{add_favorite, is_favorited, remove_favorite};
pub use follows::{
    block_user, count_followers, follow_user, get_followers, get_following, is_blocking, is_following, is_muting,
    mute_user, unblock_user, unfollow_user, unmute_user,
};
pub use hashtags::{get_notes_by_tag, get_trending_tags};
pub use notes::{create_note, delete_note, get_note_by_id, get_note_by_uri};
pub use notifications::{
    create_notification, get_notifications, mark_all_notifications_as_read,
    mark_notification_as_read,
};
pub use polls::vote_poll;
pub use reactions::{add_reaction, remove_reaction};
pub use relay::{
    create_relay, delete_relay, get_accepted_relays, get_relay_by_id, get_relay_by_inbox,
    list_relays, update_relay_status,
};
pub use timeline::{
    NoteWithAuthor, get_global_timeline, get_home_timeline, get_local_timeline, get_note_quotes,
    get_note_replies, get_user_notes,
};

use serde::de::DeserializeOwned;
use surrealdb::types::Value;

/// surrealdb 3.x の `take` は結果型に `SurrealValue` を要求するが、ドメインモデルは
/// serde の `Deserialize` のみ実装している。生の `Value` 行を JSON 経由で serde
/// デシリアライズしてモデルへ変換する。
pub fn rows_to<T: DeserializeOwned>(rows: Vec<Value>) -> anyhow::Result<Vec<T>> {
    rows.into_iter()
        .map(|value| {
            let mut json = value.into_json_value();
            strip_record_prefixes(&mut json);
            serde_json::from_value::<T>(json).map_err(anyhow::Error::from)
        })
        .collect()
}

/// SurrealDB のレコード ID は JSON 化すると `"table:01ULID..."` 形式になる。
/// ドメインモデルは ULID 部分のみを期待するため、再帰的にプレフィックスを剥がす。
pub fn strip_record_prefixes(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            if let Some(stripped) = strip_record_prefix(s) {
                *s = stripped;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                strip_record_prefixes(item);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                strip_record_prefixes(v);
            }
        }
        _ => {}
    }
}

/// `"table:01ULID"` 形式なら ULID 部分を返す
fn strip_record_prefix(s: &str) -> Option<String> {
    let (table, id) = s.split_once(':')?;
    if table.is_empty()
        || !table
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return None;
    }
    // ULID: Crockford Base32 26文字
    if id.len() == 26
        && id
            .chars()
            .all(|c| c.is_ascii_digit() || (c.is_ascii_uppercase() && !"ILOU".contains(c)))
    {
        Some(id.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_record_prefixes_recursively() {
        let mut value = serde_json::json!({
            "id": "user:01JXYZABCDEFGHJKMNPQRSTVWX",
            "uri": "https://example.com/users/alice",
            "nested": { "note_id": "note:01JXYZABCDEFGHJKMNPQRSTVWX" },
            "list": ["user:01JXYZABCDEFGHJKMNPQRSTVWX", "plain"],
            "emoji": ":fire:"
        });
        strip_record_prefixes(&mut value);
        assert_eq!(value["id"], "01JXYZABCDEFGHJKMNPQRSTVWX");
        assert_eq!(value["uri"], "https://example.com/users/alice");
        assert_eq!(value["nested"]["note_id"], "01JXYZABCDEFGHJKMNPQRSTVWX");
        assert_eq!(value["list"][0], "01JXYZABCDEFGHJKMNPQRSTVWX");
        assert_eq!(value["list"][1], "plain");
        assert_eq!(value["emoji"], ":fire:");
    }
}
