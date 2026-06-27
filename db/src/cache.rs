//! Dragonfly キャッシュヘルパ (TODO Phase 2)
//!
//! JSON キャッシュ (TTL 付き) とタイムライン Sorted Set を提供する。

use redis::AsyncCommands;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

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

/// CacheMetrics を使用するバージョン
pub async fn get_json_with_metrics<T: DeserializeOwned>(
    client: &DragonflyClient,
    key: &str,
    metrics: &CacheMetrics,
) -> Option<T> {
    let mut conn = client.manager();
    let raw: Option<String> = match conn.get(key).await {
        Ok(v) => {
            metrics.record_hit();
            v
        }
        Err(_) => {
            metrics.record_miss();
            return None;
        }
    };
    match raw {
        Some(ref s) => serde_json::from_str(s).ok().or_else(|| {
            metrics.record_miss();
            None
        }),
        None => {
            metrics.record_miss();
            None
        }
    }
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

// ---------------------------------------------------------------------------
// Block/Mute set cache for fast lookup (P-G6)
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
    let key = format!("block_set:{}", user_id);
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
    let key = format!("mute_set:{}", user_id);
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
    let key = format!("block_set:{}", user_id);
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
    let key = format!("mute_set:{}", user_id);
    let mut conn = client.manager();
    let _: () = conn.srem(&key, target_id).await?;
    Ok(())
}

/// ユーザーのブロックセットにユーザーIDが含まれるか確認する
pub async fn block_set_contains(client: &DragonflyClient, user_id: &str, target_id: &str) -> bool {
    let key = format!("block_set:{}", user_id);
    let mut conn = client.manager();
    conn.sismember(&key, target_id).await.unwrap_or(false)
}

/// ユーザーのミュートセットにユーザーIDが含まれるか確認する
pub async fn mute_set_contains(client: &DragonflyClient, user_id: &str, target_id: &str) -> bool {
    let key = format!("mute_set:{}", user_id);
    let mut conn = client.manager();
    conn.sismember(&key, target_id).await.unwrap_or(false)
}

/// ユーザーのブロックセットを全件取得する（フォロワー数 >= 10,000 のケース用）
pub async fn block_set_get_all(client: &DragonflyClient, user_id: &str) -> Vec<String> {
    let key = format!("block_set:{}", user_id);
    let mut conn = client.manager();
    conn.smembers(&key).await.unwrap_or_default()
}

/// ユーザーのミュートセットを全件取得する
pub async fn mute_set_get_all(client: &DragonflyClient, user_id: &str) -> Vec<String> {
    let key = format!("mute_set:{}", user_id);
    let mut conn = client.manager();
    conn.smembers(&key).await.unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Cache hit/miss counters (P-G16)
// ---------------------------------------------------------------------------

/// キャッシュヒット率計測用カウンタ
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub hits: Arc<AtomicU64>,
    pub misses: Arc<AtomicU64>,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let misses = self.misses.load(Ordering::Relaxed) as f64;
        if hits + misses == 0.0 {
            0.0
        } else {
            hits / (hits + misses)
        }
    }
}
