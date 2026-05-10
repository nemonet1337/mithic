use std::sync::Arc;

use reqwest::Client;
use base64::prelude::*;
use sigh::Key;
use tracing::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    db::{DragonflyClient, SurrealClient},
    models::{Actor, Note},
};

/// 連合配送サービス
#[derive(Debug, Clone)]
pub struct FederationService {
    surreal: Arc<SurrealClient>,
    dragonfly: Arc<DragonflyClient>,
    http_client: Client,
    instance_url: String,
}

impl FederationService {
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
        instance_url: String,
    ) -> Self {
        Self {
            surreal: Arc::new(surreal),
            dragonfly: Arc::new(dragonfly),
            http_client: Client::new(),
            instance_url,
        }
    }

    /// 配送キューに追加
    pub async fn queue_delivery(
        &self,
        activity: serde_json::Value,
        inbox_urls: Vec<String>,
    ) -> anyhow::Result<()> {
        for inbox_url in inbox_urls {
            let delivery = serde_json::json!({
                "inbox_url": inbox_url,
                "activity": activity,
                "attempts": 0,
            });

            // Dragonfly（Redis）リストに追加
            redis::cmd("LPUSH")
                .arg("federation:queue")
                .arg(delivery.to_string())
                .query_async::<_, ()>(&mut self.dragonfly.clone())
                .await?;
        }

        Ok(())
    }

    /// 配送を実行（署名付き）
    pub async fn deliver_signed(
        &self,
        inbox_url: &str,
        activity: &serde_json::Value,
        actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Delivering signed activity to {}", inbox_url);

        // アクターの秘密鍵を取得
        let private_key = actor
            .private_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Actor has no private key"))?;

        let key_id = format!("{}#main-key", actor.actor_uri(&self.instance_url));

        // JSONボディをバイト列に変換
        let body = serde_json::to_vec(activity)?;

        // URLをパース
        let parsed_url = url::Url::parse(inbox_url)?;
        let path = parsed_url.path();
        let host = parsed_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid inbox URL: no host"))?;

        // 署名生成
        let date = chrono::Utc::now().to_rfc2822();
        let digest = format!("SHA-256={}", BASE64_STANDARD.encode(sha2::Sha256::digest(&body)));

        // 署名対象文字列
        let signature_input = format!(
            "(request-target): post {}\nhost: {}\ndate: {}\ndigest: {}",
            path, host, date, digest
        );

        // 秘密鍵で署名
        let key = Key::from_pem(private_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {:?}", e))?;
        let signature_bytes = key
            .sign(signature_input.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to sign request: {:?}", e))?;
        let signature_b64 = BASE64_STANDARD.encode(signature_bytes);

        // Signatureヘッダー
        let signature_header = format!(
            "keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{}\"",
            key_id, signature_b64
        );

        // HTTPリクエスト送信
        let response = self
            .http_client
            .post(inbox_url)
            .header("Content-Type", "application/activity+json")
            .header("Accept", "application/activity+json")
            .header("Date", date)
            .header("Digest", digest)
            .header("Signature", signature_header)
            .header("Host", host)
            .body(body)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Successfully delivered to {}", inbox_url);
                    Ok(())
                } else {
                    let status = resp.status();
                    error!("Delivery failed to {}: {}", inbox_url, status);
                    Err(anyhow::anyhow!("HTTP error: {}", status))
                }
            }
            Err(e) => {
                error!("Delivery error to {}: {}", inbox_url, e);
                Err(e.into())
            }
        }
    }

    /// 配送を実行（署名なし - レガシー互換用）
    pub async fn deliver(&self, inbox_url: &str, activity: &serde_json::Value) -> anyhow::Result<()> {
        warn!("Delivering unsigned activity to {} (legacy mode)", inbox_url);

        let response = self
            .http_client
            .post(inbox_url)
            .header("Content-Type", "application/activity+json")
            .header("Accept", "application/activity+json")
            .json(activity)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Successfully delivered to {}", inbox_url);
                    Ok(())
                } else {
                    let status = resp.status();
                    error!("Delivery failed to {}: {}", inbox_url, status);
                    Err(anyhow::anyhow!("HTTP error: {}", status))
                }
            }
            Err(e) => {
                error!("Delivery error to {}: {}", inbox_url, e);
                Err(e.into())
            }
        }
    }

    /// フォロワーに配信
    ///
    /// # Arguments
    /// * `activity` - 配信するActivityPubアクティビティ
    /// * `actor_id` - 送信元アクターID (user:xxx形式)
    /// * `actor` - 署名用アクター（秘密鍵を含む）
    pub async fn broadcast_to_followers(
        &self,
        activity: serde_json::Value,
        actor_id: &str,
        actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Broadcasting activity to followers of {}", actor_id);

        // フォロワーのinbox URLを取得
        // まずリモートフォロワーのinbox情報を取得
        let query = r#"
            SELECT follower.inbox as inbox, follower.shared_inbox as shared_inbox
            FROM follow
            WHERE followee = $actor_id
            AND follower.host IS NOT NULL
        "#;

        let mut result = self
            .surreal
            .query(query)
            .bind(("actor_id", actor_id))
            .await
            .map_err(|e| {
                error!("Failed to fetch followers: {}", e);
                anyhow::anyhow!("Database error: {}", e)
            })?;

        // 結果を取得
        let follower_data: Vec<(Option<String>, Option<String>)> = result
            .take(0)
            .unwrap_or_default();

        if follower_data.is_empty() {
            info!("No remote followers to broadcast to");
            return Ok(());
        }

        // shared_inboxでグループ化して重複排除
        let mut inbox_groups: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for (inbox, shared_inbox) in follower_data {
            let target_inbox = shared_inbox.clone().or(inbox.clone());

            if let Some(target) = target_inbox {
                // shared_inboxがある場合はそれをキーに、ない場合は個別inbox
                let group_key = shared_inbox.unwrap_or_else(|| inbox.clone().unwrap_or_default());
                inbox_groups
                    .entry(group_key)
                    .or_default()
                    .push(target);
            }
        }

        let unique_inboxes: Vec<String> = inbox_groups.keys().cloned().collect();
        info!("Broadcasting to {} unique inboxes", unique_inboxes.len());

        // 各inboxに配送（キューに追加）
        self.queue_delivery(activity, unique_inboxes).await?;

        Ok(())
    }

    /// Acceptアクティビティを配送（フォロー承認時）
    ///
    /// # Arguments
    /// * `follower_inbox` - フォロワーのinbox URL
    /// * `follower_actor_id` - フォロワーのActor ID
    /// * `local_actor` - ローカルアクター（署名用）
    pub async fn send_accept_follow(
        &self,
        follower_inbox: &str,
        follower_actor_id: &str,
        local_actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Sending Accept Follow to {}", follower_inbox);

        let local_actor_uri = local_actor
            .uri
            .clone()
            .unwrap_or_else(|| format!("{}/users/{}", self.instance_url, local_actor.username));

        // Acceptアクティビティを作成
        let accept_activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}#accepts/{}", local_actor_uri, uuid::Uuid::new_v4()),
            "type": "Accept",
            "actor": local_actor_uri,
            "object": {
                "type": "Follow",
                "actor": follower_actor_id,
                "object": local_actor_uri
            }
        });

        // 署名付きで直接配送
        self.deliver_signed(follower_inbox, &accept_activity, local_actor).await
    }

    /// Rejectアクティビティを配送（フォロー拒否時）
    ///
    /// # Arguments
    /// * `follower_inbox` - フォロワーのinbox URL
    /// * `follower_actor_id` - フォロワーのActor ID
    /// * `local_actor` - ローカルアクター（署名用）
    pub async fn send_reject_follow(
        &self,
        follower_inbox: &str,
        follower_actor_id: &str,
        local_actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Sending Reject Follow to {}", follower_inbox);

        let local_actor_uri = local_actor
            .uri
            .clone()
            .unwrap_or_else(|| format!("{}/users/{}", self.instance_url, local_actor.username));

        // Rejectアクティビティを作成
        let reject_activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}#rejects/{}", local_actor_uri, uuid::Uuid::new_v4()),
            "type": "Reject",
            "actor": local_actor_uri,
            "object": {
                "type": "Follow",
                "actor": follower_actor_id,
                "object": local_actor_uri
            }
        });

        // 署名付きで直接配送
        self.deliver_signed(follower_inbox, &reject_activity, local_actor).await
    }

    /// キュー統計を取得
    pub async fn get_queue_stats(&self) -> anyhow::Result<QueueStats> {
        let queue_length: usize = redis::cmd("LLEN")
            .arg("federation:queue")
            .query_async(&mut self.dragonfly.clone())
            .await
            .unwrap_or(0);

        let processing: usize = redis::cmd("LLEN")
            .arg("federation:processing")
            .query_async(&mut self.dragonfly.clone())
            .await
            .unwrap_or(0);

        Ok(QueueStats {
            total: queue_length,
            processing,
            pending: queue_length.saturating_sub(processing),
        })
    }

    /// キュージョブ一覧を取得
    pub async fn get_queue_jobs(&self, limit: usize) -> anyhow::Result<Vec<QueueJob>> {
        let jobs: Vec<String> = redis::cmd("LRANGE")
            .arg("federation:queue")
            .arg(0)
            .arg(limit - 1)
            .query_async(&mut self.dragonfly.clone())
            .await
            .unwrap_or_default();

        let mut result = Vec::new();
        for (i, job_str) in jobs.iter().enumerate() {
            if let Ok(job_value) = serde_json::from_str::<serde_json::Value>(job_str) {
                result.push(QueueJob {
                    id: i.to_string(),
                    inbox_url: job_value.get("inbox_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    activity: job_value.get("activity").cloned().unwrap_or(serde_json::Value::Null),
                    attempts: job_value.get("attempts").and_then(|v| v.as_i64()).unwrap_or(0),
                });
            }
        }

        Ok(result)
    }

    /// リモートActorを取得
    pub async fn fetch_remote_actor(&self, actor_url: &str) -> anyhow::Result<Option<Actor>> {
        let response = self
            .http_client
            .get(actor_url)
            .header("Accept", "application/activity+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let actor_data: serde_json::Value = response.json().await?;

        // TODO: JSON-LDからActorモデルへの変換
        warn!("Remote actor fetching not fully implemented");

        Ok(None)
    }

    /// 配送キューワーカーを実行
    ///
    /// バックグラウンドタスクとして実行し、キューから配送タスクを処理
    pub async fn run_delivery_worker(&self) -> anyhow::Result<()> {
        info!("Starting federation delivery worker");

        loop {
            // キューからタスクを取り出し（ブロッキング）
            let delivery_json: Option<String> = redis::cmd("BRPOP")
                .arg("federation:queue")
                .arg(30) // 30秒タイムアウト
                .query_async::<_, Option<(String, String)>>(&mut self.dragonfly.clone())
                .await
                .map(|opt| opt.map(|(_, val)| val))?;

            if let Some(delivery_json) = delivery_json {
                match serde_json::from_str::<serde_json::Value>(&delivery_json) {
                    Ok(delivery) => {
                        if let Err(e) =
                            self.process_delivery_task(delivery).await
                        {
                            error!("Failed to process delivery: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Invalid delivery JSON: {}", e);
                    }
                }
            }
        }
    }

    /// 単一の配送タスクを処理
    async fn process_delivery_task(
        &self,
        delivery: serde_json::Value,
    ) -> anyhow::Result<()> {
        let inbox_url = delivery
            .get("inbox_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing inbox_url"))?;
        let activity = delivery
            .get("activity")
            .ok_or_else(|| anyhow::anyhow!("Missing activity"))?;
        let attempts = delivery
            .get("attempts")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // アクター情報を取得（activity.actorから）
        let actor_id = activity
            .get("actor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing actor in activity"))?;

        // ローカルアクターをDBから取得
        let actor_result: Option<Actor> = self
            .surreal
            .query("SELECT * FROM user WHERE uri = $actor_id")
            .bind(("actor_id", actor_id))
            .await
            .and_then(|mut res| res.take(0))
            .ok()
            .flatten();

        let actor = actor_result.ok_or_else(|| {
            anyhow::anyhow!("Actor not found or has no private key: {}", actor_id)
        })?;

        // 署名付きで配送
        match self.deliver_signed(inbox_url, activity, &actor).await {
            Ok(_) => {
                info!("Successfully delivered to {}", inbox_url);
                Ok(())
            }
            Err(e) => {
                error!("Delivery failed to {}: {}", inbox_url, e);

                // 最大試行回数をチェック
                if attempts < 5 {
                    // リトライキューに追加（指数バックオフ）
                    let delay = std::time::Duration::from_secs(60 * (attempts + 1) as u64);
                    let retry_delivery = serde_json::json!({
                        "inbox_url": inbox_url,
                        "activity": activity,
                        "attempts": attempts + 1,
                        "retry_after": chrono::Utc::now() + chrono::Duration::from_std(delay)?,
                    });

                    // 遅延キューに追加（簡易実装：リストにプッシュ）
                    redis::cmd("LPUSH")
                        .arg("federation:queue:retry")
                        .arg(retry_delivery.to_string())
                        .query_async::<_, ()>(&mut self.dragonfly.clone())
                        .await?;

                    warn!("Scheduled retry for {} (attempt {})", inbox_url, attempts + 1);
                } else {
                    error!("Giving up on delivery to {} after {} attempts", inbox_url, attempts);
                }

                Err(e)
            }
        }
    }
}
