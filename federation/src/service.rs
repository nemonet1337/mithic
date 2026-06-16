use std::collections::HashMap;
use std::sync::Arc;

use apalis::prelude::*;
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

/// apalis-redis で使用されるキー名
const QUEUE_KEY: &str = "apalis:job:mithic::ActivityDelivery:pending";
const SCHEDULED_KEY: &str = "apalis:job:mithic::ActivityDelivery:processing";
/// 最終的に断念した配送
const DLQ_KEY: &str = "federation:dlq";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivityDelivery {
    pub inbox_url: String,
    pub activity: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct FederationService {
    surreal: Arc<SurrealClient>,
    dragonfly: DragonflyClient,
    storage: apalis_redis::RedisStorage<ActivityDelivery>,
    http_client: Client,
    instance_url: String,
    /// actor URI → パース済み秘密鍵のプロセス内キャッシュ
    key_cache: Arc<RwLock<HashMap<String, Arc<RsaPrivateKey>>>>,
}

impl FederationService {
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
        storage: apalis_redis::RedisStorage<ActivityDelivery>,
        http_client: Client,
        instance_url: String,
    ) -> Self {
        Self {
            surreal: Arc::new(surreal),
            dragonfly,
            storage,
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
        let mut storage = self.storage.clone();
        for inbox_url in inbox_urls {
            let delivery = ActivityDelivery {
                inbox_url,
                activity: activity.clone(),
            };
            if let Err(e) = storage.push(delivery).await {
                error!("Failed to push delivery to apalis: {}", e);
            }
        }
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

    fn to_apalis_error(&self, err: anyhow::Error) -> apalis::prelude::Error {
        let box_err: Box<dyn std::error::Error + Send + Sync> = err.into();
        apalis::prelude::Error::Failed(std::sync::Arc::new(box_err))
    }

    pub async fn process_delivery_task(
        &self,
        job: ActivityDelivery,
    ) -> Result<(), apalis::prelude::Error> {
        let inbox_url = &job.inbox_url;
        let activity = &job.activity;

        let parsed_url = url::Url::parse(inbox_url)
            .map_err(|e| self.to_apalis_error(anyhow::anyhow!("Invalid inbox URL: {e}")))?;
        let host = parsed_url
            .host_str()
            .ok_or_else(|| self.to_apalis_error(anyhow::anyhow!("Invalid inbox URL: no host")))?;

        // Dead Inbox Circuit Breaker (P-G8 / P-G9)
        if self.is_inbox_dead(host).await {
            warn!("Skipping delivery to dead inbox host: {}", host);
            return Ok(());
        }

        let actor_id = activity
            .get("actor")
            .and_then(|v| v.as_str())
            .ok_or_else(|| self.to_apalis_error(anyhow::anyhow!("Missing actor in activity")))?;

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

        let actor = actor_result.ok_or_else(|| {
            self.to_apalis_error(anyhow::anyhow!("Actor not found: {}", actor_id))
        })?;

        match self.deliver_signed(inbox_url, activity, &actor).await {
            Ok(_) => {
                self.clear_inbox_dead(host).await;
                Ok(())
            }
            Err(e) => {
                error!("Delivery failed to {}: {}", inbox_url, e);
                self.handle_delivery_failure(host).await;
                Err(self.to_apalis_error(e))
            }
        }
    }

    async fn is_inbox_dead(&self, host: &str) -> bool {
        let mut conn = self.dragonfly.manager();
        let key = format!("dead_inbox:{}", host);
        redis::cmd("EXISTS")
            .arg(&key)
            .query_async::<i32>(&mut conn)
            .await
            .unwrap_or(0)
            == 1
    }

    async fn clear_inbox_dead(&self, host: &str) {
        let mut conn = self.dragonfly.manager();
        let key_dead = format!("dead_inbox:{}", host);
        let key_fail = format!("inbox_failures:{}", host);
        let _: Result<(), _> = redis::cmd("DEL")
            .arg(&key_dead)
            .arg(&key_fail)
            .query_async(&mut conn)
            .await;
    }

    async fn handle_delivery_failure(&self, host: &str) {
        let mut conn = self.dragonfly.manager();
        let key_fail = format!("inbox_failures:{}", host);
        let key_dead = format!("dead_inbox:{}", host);

        let failures: i32 = match redis::cmd("INCR")
            .arg(&key_fail)
            .query_async::<i32>(&mut conn)
            .await
        {
            Ok(val) => val,
            Err(e) => {
                warn!("Failed to increment inbox failure count: {}", e);
                return;
            }
        };

        if failures >= 5 {
            warn!(
                "Inbox host {} failed {} times. Marking as dead for 1 hour.",
                host, failures
            );
            // 1時間 (3600秒) 配送をブロック
            let _: Result<(), _> = redis::cmd("SET")
                .arg(&key_dead)
                .arg("1")
                .arg("EX")
                .arg(3600)
                .query_async(&mut conn)
                .await;
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
