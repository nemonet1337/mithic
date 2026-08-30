//! Dragonfly キャッシュヘルパ
//!
//! JSON キャッシュ (TTL 付き) と block/mute Set。

use redis::AsyncCommands;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::db::DragonflyClient;

/// 公開タイムライン JSON レスポンスの短命 TTL (秒)
pub const TIMELINE_JSON_TTL_SECS: u64 = 15;
/// トレンドタグ JSON TTL
pub const TRENDING_JSON_TTL_SECS: u64 = 60;

/// local / global 先頭ページのキャッシュキー
pub fn timeline_json_key(kind: &str, limit: usize) -> String {
    format!("tl:json:{kind}:limit={limit}")
}

pub fn trending_json_key(limit: usize) -> String {
    format!("tl:json:trending:limit={limit}")
}

/// 公開タイムライン JSON キャッシュを無効化 (投稿時)
pub async fn invalidate_public_timelines(client: &DragonflyClient) {
    let mut conn = client.manager();
    let mut pipe = redis::pipe();
    for kind in ["local", "global"] {
        for limit in [10usize, 20, 40, 50, 100] {
            pipe.del(timeline_json_key(kind, limit)).ignore();
        }
    }
    for limit in [5usize, 10, 20, 50] {
        pipe.del(trending_json_key(limit)).ignore();
    }
    let _: Result<(), _> = pipe.query_async(&mut conn).await;
}

/// JSON 値を TTL 付きでキャッシュする
pub async fn set_json<T: Serialize>(
    client: &DragonflyClient,
    key: &str,
    value: &T,
    ttl_secs: u64,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(value)?;
    let mut conn = client.manager();
    let _: () = conn.set_ex(key, payload, ttl_secs).await?;
    Ok(())
}

/// キャッシュから JSON 値を取得する。ミスやデコード失敗は `None`。
pub async fn get_json<T: DeserializeOwned>(client: &DragonflyClient, key: &str) -> Option<T> {
    let mut conn = client.manager();
    let raw: Option<String> = conn.get(key).await.ok()?;
    serde_json::from_str(&raw?).ok()
}

/// キャッシュキーを削除する
pub async fn delete(client: &DragonflyClient, key: &str) -> anyhow::Result<()> {
    let mut conn = client.manager();
    let _: () = conn.del(key).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Block/Mute set cache for fast lookup
// ---------------------------------------------------------------------------

/// block/mute 判定の Set キャッシュ TTL (10min)
pub const BLOCK_MUTE_SET_TTL: i64 = 10 * 60;

/// ユーザーのブロックセットにユーザーIDを追加する
pub async fn block_set_add(
    client: &DragonflyClient,
    user_id: &str,
    target_id: &str,
    ttl_secs: i64,
) -> anyhow::Result<()> {
    let key = format!("block_set:{user_id}");
    let mut conn = client.manager();
    let _: () = conn.sadd(&key, target_id).await?;
    let _: () = conn.expire(&key, ttl_secs).await?;
    Ok(())
}

/// ユーザーのミュートセットにユーザーIDを追加する
pub async fn mute_set_add(
    client: &DragonflyClient,
    user_id: &str,
    target_id: &str,
    ttl_secs: i64,
) -> anyhow::Result<()> {
    let key = format!("mute_set:{user_id}");
    let mut conn = client.manager();
    let _: () = conn.sadd(&key, target_id).await?;
    let _: () = conn.expire(&key, ttl_secs).await?;
    Ok(())
}

/// ユーザーのブロックセットからユーザーIDを削除する
pub async fn block_set_remove(
    client: &DragonflyClient,
    user_id: &str,
    target_id: &str,
) -> anyhow::Result<()> {
    let key = format!("block_set:{user_id}");
    let mut conn = client.manager();
    let _: () = conn.srem(&key, target_id).await?;
    Ok(())
}

/// ユーザーのミュートセットからユーザーIDを削除する
pub async fn mute_set_remove(
    client: &DragonflyClient,
    user_id: &str,
    target_id: &str,
) -> anyhow::Result<()> {
    let key = format!("mute_set:{user_id}");
    let mut conn = client.manager();
    let _: () = conn.srem(&key, target_id).await?;
    Ok(())
}

/// ユーザーのブロックセットにユーザーIDが含まれるか確認する
pub async fn block_set_contains(client: &DragonflyClient, user_id: &str, target_id: &str) -> bool {
    let key = format!("block_set:{user_id}");
    let mut conn = client.manager();
    conn.sismember(&key, target_id).await.unwrap_or(false)
}

/// ユーザーのミュートセットにユーザーIDが含まれるか確認する
pub async fn mute_set_contains(client: &DragonflyClient, user_id: &str, target_id: &str) -> bool {
    let key = format!("mute_set:{user_id}");
    let mut conn = client.manager();
    conn.sismember(&key, target_id).await.unwrap_or(false)
}
