use std::collections::HashMap;
use std::sync::Arc;

use base64::prelude::*;
use reqwest::Client;
use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::sha2::Sha256 as RsaSha256;
use rsa::signature::{SignatureEncoding, Signer};
use sha2::Digest;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use mithic_core::models::Actor;
use mithic_db::{DragonflyClient, SurrealClient};

/// 配送待ちキュー (BRPOP で取り出す)
const QUEUE_KEY: &str = "federation:queue";
/// リトライ予約 ZSET (score = 再投入予定 unix 秒)
const SCHEDULED_KEY: &str = "federation:scheduled";
/// 最終的に断念した配送
const DLQ_KEY: &str = "federation:dlq";
/// 配送リトライ上限
const MAX_ATTEMPTS: i64 = 5;

#[derive(Debug, Clone)]
pub struct FederationService {
    surreal: Arc<SurrealClient>,
    dragonfly: DragonflyClient,
    http_client: Client,
    instance_url: String,
    /// actor URI → パース済み秘密鍵のプロセス内キャッシュ
    key_cache: Arc<RwLock<HashMap<String, Arc<RsaPrivateKey>>>>,
}

impl FederationService {
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
        http_client: Client,
        instance_url: String,
    ) -> Self {
        Self {
            surreal: Arc::new(surreal),
            dragonfly,
            http_client,
            instance_url,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// public のみ連合へ配送する (home/followers/specified は配送しない)
    pub fn should_deliver(visibility: &str) -> bool {
        visibility == "public"
    }

    pub async fn queue_delivery(
        &self,
        activity: serde_json::Value,
        inbox_urls: Vec<String>,
    ) -> anyhow::Result<()> {
        if inbox_urls.is_empty() {
            return Ok(());
        }
        let mut conn = self.dragonfly.manager();
        let mut pipe = redis::pipe();
        for inbox_url in inbox_urls {
            let delivery = serde_json::json!({
                "inbox_url": inbox_url,
                "activity": activity,
                "attempts": 0,
            });
            pipe.cmd("LPUSH")
                .arg(QUEUE_KEY)
                .arg(delivery.to_string())
                .ignore();
        }
        pipe.query_async::<()>(&mut conn).await?;
        Ok(())
    }

    /// 秘密鍵 PEM をパースしキャッシュする
    async fn signing_key_for(&self, actor: &Actor) -> anyhow::Result<Arc<RsaPrivateKey>> {
        let key_id = actor
            .uri
            .clone()
            .unwrap_or_else(|| actor.actor_uri(&self.instance_url));

        if let Some(key) = self.key_cache.read().await.get(&key_id) {
            return Ok(key.clone());
        }

        let pem = actor
            .private_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Actor has no private key"))?;

        let key = RsaPrivateKey::from_pkcs8_pem(pem)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem))
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {e}"))?;
        let key = Arc::new(key);

        self.key_cache.write().await.insert(key_id, key.clone());
        Ok(key)
    }

    pub async fn deliver_signed(
        &self,
        inbox_url: &str,
        activity: &serde_json::Value,
        actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Delivering signed activity to {}", inbox_url);

        let actor_uri = actor
            .uri
            .clone()
            .unwrap_or_else(|| actor.actor_uri(&self.instance_url));
        let key_id = format!("{actor_uri}#main-key");

        let body = serde_json::to_vec(activity)?;
        let parsed_url = url::Url::parse(inbox_url)?;
        let path = parsed_url.path().to_string();
        let host = parsed_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid inbox URL: no host"))?
            .to_string();

        // HTTP-date 形式 (RFC 7231)
        let date = chrono::Utc::now()
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string();
        let digest = format!(
            "SHA-256={}",
            BASE64_STANDARD.encode(sha2::Sha256::digest(&body))
        );

        // RSA-SHA256 で署名 (TODO Phase F2: placeholder 置換済み)
        let signing_string =
            format!("(request-target): post {path}\nhost: {host}\ndate: {date}\ndigest: {digest}");
        let private_key = self.signing_key_for(actor).await?;
        let signing_key = SigningKey::<RsaSha256>::new((*private_key).clone());
        let signature = signing_key.sign(signing_string.as_bytes());
        let signature_b64 = BASE64_STANDARD.encode(signature.to_bytes());

        let signature_header = format!(
            "keyId=\"{}\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{}\"",
            key_id, signature_b64
        );

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

    pub async fn broadcast_to_followers(
        &self,
        activity: serde_json::Value,
        actor_id: &str,
        _actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Broadcasting activity to followers of {}", actor_id);

        // フォローグラフからリモートフォロワーの inbox を取得し、
        // sharedInbox 単位でグルーピングして同一インスタンスへは1回だけ配送する
        let query = r#"
            SELECT in.inbox AS inbox, in.shared_inbox AS shared_inbox
            FROM follow
            WHERE out = type::record('user', $actor_id)
            AND in.host != None
        "#;

        let mut result = self
            .surreal
            .query(query)
            .bind(("actor_id", actor_id.to_string()))
            .await
            .map_err(|e| {
                error!("Failed to fetch followers: {}", e);
                anyhow::anyhow!("Database error: {}", e)
            })?;

        let rows: Vec<surrealdb::types::Value> = result.take(0).unwrap_or_default();
        let follower_data: Vec<(Option<String>, Option<String>)> = rows
            .into_iter()
            .map(|v| {
                let json = v.into_json_value();
                (
                    json.get("inbox").and_then(|x| x.as_str()).map(String::from),
                    json.get("shared_inbox")
                        .and_then(|x| x.as_str())
                        .map(String::from),
                )
            })
            .collect();

        if follower_data.is_empty() {
            info!("No remote followers to broadcast to");
            return Ok(());
        }

        let mut unique_inboxes: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for (inbox, shared_inbox) in follower_data {
            if let Some(target) = shared_inbox.or(inbox) {
                unique_inboxes.insert(target);
            }
        }

        info!("Broadcasting to {} unique inboxes", unique_inboxes.len());
        self.queue_delivery(activity, unique_inboxes.into_iter().collect())
            .await?;
        Ok(())
    }

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
            .unwrap_or_else(|| local_actor.actor_uri(&self.instance_url));

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

        self.deliver_signed(follower_inbox, &accept_activity, local_actor)
            .await
    }

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
            .unwrap_or_else(|| local_actor.actor_uri(&self.instance_url));

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

        self.deliver_signed(follower_inbox, &reject_activity, local_actor)
            .await
    }

    pub async fn get_queue_stats(&self) -> anyhow::Result<QueueStats> {
        let mut conn = self.dragonfly.manager();
        let queue_length: usize = redis::cmd("LLEN")
            .arg(QUEUE_KEY)
            .query_async::<usize>(&mut conn)
            .await
            .unwrap_or(0);

        let scheduled: usize = redis::cmd("ZCARD")
            .arg(SCHEDULED_KEY)
            .query_async::<usize>(&mut conn)
            .await
            .unwrap_or(0);

        Ok(QueueStats {
            total: queue_length + scheduled,
            processing: scheduled,
            pending: queue_length,
        })
    }

    pub async fn get_queue_jobs(&self, limit: usize) -> anyhow::Result<Vec<QueueJob>> {
        let mut conn = self.dragonfly.manager();
        let jobs: Vec<String> = redis::cmd("LRANGE")
            .arg(QUEUE_KEY)
            .arg(0i64)
            .arg((limit.saturating_sub(1)) as i64)
            .query_async::<Vec<String>>(&mut conn)
            .await
            .unwrap_or_default();

        let mut result = Vec::new();
        for (i, job_str) in jobs.iter().enumerate() {
            if let Ok(job_value) = serde_json::from_str::<serde_json::Value>(job_str) {
                result.push(QueueJob {
                    id: i.to_string(),
                    inbox_url: job_value
                        .get("inbox_url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    activity: job_value
                        .get("activity")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                    attempts: job_value
                        .get("attempts")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0),
                });
            }
        }
        Ok(result)
    }

    /// リモートアクターを取得し JSON-LD を `Actor` へ変換する (TODO Phase F3)
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

        let data: serde_json::Value = response.json().await?;
        Ok(parse_remote_actor(&data))
    }

    /// 配送ワーカーを並列起動する (TODO Phase F2: 配送の並列化)
    pub async fn run_delivery_workers(&self, concurrency: usize) -> anyhow::Result<()> {
        let concurrency = concurrency.max(1);
        info!("Starting {} federation delivery workers", concurrency);

        let mut handles = Vec::new();
        for worker_id in 0..concurrency {
            let service = self.clone();
            handles.push(tokio::spawn(async move {
                if let Err(e) = service.run_delivery_worker_loop(worker_id).await {
                    error!("Delivery worker {} terminated: {}", worker_id, e);
                }
            }));
        }

        // リトライスケジューラ
        let service = self.clone();
        handles.push(tokio::spawn(async move {
            service.run_retry_scheduler().await;
        }));

        for handle in handles {
            let _ = handle.await;
        }
        Ok(())
    }

    /// 後方互換: 単一ワーカー起動
    pub async fn run_delivery_worker(&self) -> anyhow::Result<()> {
        self.run_delivery_workers(1).await
    }

    async fn run_delivery_worker_loop(&self, worker_id: usize) -> anyhow::Result<()> {
        // BRPOP は接続を占有するため専用コネクションを使う
        let mut conn = self.dragonfly.dedicated_connection().await?;
        info!("Federation delivery worker {} started", worker_id);
        loop {
            let delivery_json: Option<String> = match redis::cmd("BRPOP")
                .arg(QUEUE_KEY)
                .arg(30i64)
                .query_async::<Option<(String, String)>>(&mut conn)
                .await
            {
                Ok(opt) => opt.map(|(_, val)| val),
                Err(e) => {
                    warn!(
                        "Worker {} lost queue connection: {}; reconnecting",
                        worker_id, e
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    conn = self.dragonfly.dedicated_connection().await?;
                    continue;
                }
            };

            if let Some(delivery_json) = delivery_json {
                match serde_json::from_str::<serde_json::Value>(&delivery_json) {
                    Ok(delivery) => {
                        if let Err(e) = self.process_delivery_task(delivery).await {
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

    /// `federation:scheduled` ZSET の期限が来たリトライを本キューへ戻す
    async fn run_retry_scheduler(&self) {
        info!("Federation retry scheduler started");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            let now = chrono::Utc::now().timestamp();
            let mut conn = self.dragonfly.manager();

            let due: Vec<String> = match redis::cmd("ZRANGEBYSCORE")
                .arg(SCHEDULED_KEY)
                .arg("-inf")
                .arg(now)
                .arg("LIMIT")
                .arg(0)
                .arg(100)
                .query_async::<Vec<String>>(&mut conn)
                .await
            {
                Ok(items) => items,
                Err(e) => {
                    warn!("Retry scheduler poll failed: {}", e);
                    continue;
                }
            };

            for item in due {
                let mut pipe = redis::pipe();
                pipe.cmd("ZREM").arg(SCHEDULED_KEY).arg(&item).ignore();
                pipe.cmd("LPUSH").arg(QUEUE_KEY).arg(&item).ignore();
                if let Err(e) = pipe.query_async::<()>(&mut conn).await {
                    warn!("Failed to requeue scheduled delivery: {}", e);
                }
            }
        }
    }

    async fn process_delivery_task(&self, delivery: serde_json::Value) -> anyhow::Result<()> {
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

        let actor_id = activity
            .get("actor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing actor in activity"))?;

        let actor_result: Option<Actor> = self
            .surreal
            .query("SELECT * FROM user WHERE uri = $actor_id LIMIT 1")
            .bind(("actor_id", actor_id.to_string()))
            .await
            .ok()
            .and_then(|mut res| res.take::<Vec<surrealdb::types::Value>>(0).ok())
            .and_then(|v| v.into_iter().next())
            .and_then(|v| {
                let mut json = v.into_json_value();
                mithic_db::queries::strip_record_prefixes(&mut json);
                serde_json::from_value::<Actor>(json).ok()
            });

        let actor = actor_result.ok_or_else(|| anyhow::anyhow!("Actor not found: {}", actor_id))?;

        match self.deliver_signed(inbox_url, activity, &actor).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Delivery failed to {}: {}", inbox_url, e);
                let mut conn = self.dragonfly.manager();
                if attempts < MAX_ATTEMPTS {
                    // 指数バックオフ + ジッタで再スケジュール
                    let backoff_secs = 60i64 * (1 << attempts.min(6));
                    let jitter = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.subsec_nanos())
                        .unwrap_or(0) as i64)
                        % 30;
                    let retry_at = chrono::Utc::now().timestamp() + backoff_secs + jitter;
                    let retry_delivery = serde_json::json!({
                        "inbox_url": inbox_url,
                        "activity": activity,
                        "attempts": attempts + 1,
                    });
                    redis::cmd("ZADD")
                        .arg(SCHEDULED_KEY)
                        .arg(retry_at)
                        .arg(retry_delivery.to_string())
                        .query_async::<()>(&mut conn)
                        .await?;
                    warn!(
                        "Scheduled retry for {} (attempt {}, in {}s)",
                        inbox_url,
                        attempts + 1,
                        backoff_secs + jitter
                    );
                } else {
                    // 上限超過は DLQ へ
                    redis::cmd("LPUSH")
                        .arg(DLQ_KEY)
                        .arg(delivery.to_string())
                        .query_async::<()>(&mut conn)
                        .await?;
                    error!(
                        "Giving up on delivery to {} after {} attempts (moved to DLQ)",
                        inbox_url, attempts
                    );
                }
                Err(e)
            }
        }
    }
}

/// ActivityPub Person JSON-LD を `Actor` (Remote) へ変換する
pub fn parse_remote_actor(data: &serde_json::Value) -> Option<Actor> {
    let uri = data.get("id")?.as_str()?.to_string();
    let username = data
        .get("preferredUsername")
        .and_then(|v| v.as_str())
        .map(String::from)?;
    let host = url::Url::parse(&uri).ok()?.host_str()?.to_string();

    let mut actor = Actor::new_local(
        username,
        data.get("name").and_then(|v| v.as_str()).map(String::from),
    );
    actor.host = Some(host);
    actor.uri = Some(uri);
    actor.bio = data
        .get("summary")
        .and_then(|v| v.as_str())
        .map(String::from);
    actor.inbox = data.get("inbox").and_then(|v| v.as_str()).map(String::from);
    actor.shared_inbox = data
        .get("endpoints")
        .and_then(|e| e.get("sharedInbox"))
        .and_then(|v| v.as_str())
        .map(String::from);
    actor.avatar_url = data
        .get("icon")
        .and_then(|i| i.get("url"))
        .and_then(|v| v.as_str())
        .map(String::from);
    actor.banner_url = data
        .get("image")
        .and_then(|i| i.get("url"))
        .and_then(|v| v.as_str())
        .map(String::from);
    actor.public_key = data
        .get("publicKey")
        .and_then(|k| k.get("publicKeyPem"))
        .and_then(|v| v.as_str())
        .map(String::from);
    actor.is_locked = data
        .get("manuallyApprovesFollowers")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    actor.is_bot = data.get("type").and_then(|v| v.as_str()) == Some("Service");

    Some(actor)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct QueueStats {
    pub total: usize,
    pub processing: usize,
    pub pending: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct QueueJob {
    pub id: String,
    pub inbox_url: String,
    pub activity: serde_json::Value,
    pub attempts: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_remote_actor_extracts_fields() {
        let data = serde_json::json!({
            "id": "https://example.com/users/alice",
            "type": "Person",
            "preferredUsername": "alice",
            "name": "Alice",
            "summary": "hello",
            "inbox": "https://example.com/users/alice/inbox",
            "endpoints": { "sharedInbox": "https://example.com/inbox" },
            "publicKey": {
                "id": "https://example.com/users/alice#main-key",
                "publicKeyPem": "-----BEGIN PUBLIC KEY-----..."
            },
            "icon": { "type": "Image", "url": "https://example.com/avatar.png" }
        });

        let actor = parse_remote_actor(&data).expect("should parse");
        assert_eq!(actor.username, "alice");
        assert_eq!(actor.host.as_deref(), Some("example.com"));
        assert_eq!(
            actor.uri.as_deref(),
            Some("https://example.com/users/alice")
        );
        assert_eq!(
            actor.inbox.as_deref(),
            Some("https://example.com/users/alice/inbox")
        );
        assert_eq!(
            actor.shared_inbox.as_deref(),
            Some("https://example.com/inbox")
        );
        assert!(actor.public_key.is_some());
        assert!(actor.is_remote());
    }

    #[test]
    fn should_deliver_only_public() {
        assert!(FederationService::should_deliver("public"));
        assert!(!FederationService::should_deliver("home"));
        assert!(!FederationService::should_deliver("followers"));
        assert!(!FederationService::should_deliver("specified"));
    }
}
