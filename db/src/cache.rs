//! Dragonfly キャッシュヘルパ (TODO Phase 2)
//!
//! JSON キャッシュ (TTL 付き) とタイムライン Sorted Set を提供する。

use redis::AsyncCommands;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::DragonflyClient;

/// タイムライン Sorted Set に保持するノート数の上限
pub const TIMELINE_MAX_ENTRIES: isize = 300;
/// タイムラインキャッシュの TTL (24h)
pub const TIMELINE_TTL_SECS: i64 = 24 * 60 * 60;

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

/// タイムライン Sorted Set にノート ID を追加し、上限超過分を切り詰める。
/// score には ULID から導出した作成時刻 (ミリ秒) を渡す。
pub async fn timeline_push(
    client: &DragonflyClient,
    key: &str,
    note_id: &str,
    score: f64,
) -> anyhow::Result<()> {
    let mut conn = client.manager();
    let mut pipe = redis::pipe();
    pipe.zadd(key, note_id, score)
        .ignore()
        .zremrangebyrank(key, 0, -(TIMELINE_MAX_ENTRIES + 1))
        .ignore()
        .expire(key, TIMELINE_TTL_SECS)
        .ignore();
    let _: () = pipe.query_async(&mut conn).await?;
    Ok(())
}

/// タイムライン Sorted Set から新しい順にノート ID を取得する
pub async fn timeline_range(
    client: &DragonflyClient,
    key: &str,
    limit: isize,
) -> anyhow::Result<Vec<String>> {
    let mut conn = client.manager();
    let ids: Vec<String> = conn.zrevrange(key, 0, limit - 1).await?;
    Ok(ids)
}
