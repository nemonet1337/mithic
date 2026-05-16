use std::sync::Arc;

use base64::prelude::*;
use reqwest::Client;
use tracing::{error, info, warn};

use mithic_core::models::Actor;
use mithic_db::{DragonflyClient, SurrealClient};

#[derive(Debug, Clone)]
pub struct FederationService {
    surreal: Arc<SurrealClient>,
    dragonfly: Arc<DragonflyClient>,
    http_client: Client,
    instance_url: String,
}

impl FederationService {
    pub fn new(surreal: SurrealClient, dragonfly: DragonflyClient, instance_url: String) -> Self {
        Self {
            surreal: Arc::new(surreal),
            dragonfly: Arc::new(dragonfly),
            http_client: Client::new(),
            instance_url,
        }
    }

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
            redis::cmd("LPUSH")
                .arg("federation:queue")
                .arg(delivery.to_string())
                .query_async::<_, ()>(&mut self.dragonfly.clone())
                .await?;
        }
        Ok(())
    }

    pub async fn deliver_signed(
        &self,
        inbox_url: &str,
        activity: &serde_json::Value,
        actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Delivering signed activity to {}", inbox_url);

        let private_key = actor
            .private_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Actor has no private key"))?;

        let key_id = format!("{}#main-key", actor.actor_uri(&self.instance_url));
        let body = serde_json::to_vec(activity)?;
        let parsed_url = url::Url::parse(inbox_url)?;
        let path = parsed_url.path();
        let host = parsed_url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid inbox URL: no host"))?;

        let date = chrono::Utc::now().to_rfc2822();
        let digest = format!(
            "SHA-256={}",
            BASE64_STANDARD.encode(sha2::Sha256::digest(&body))
        );

        // TODO: implement RSA-SHA256 signing (requires openssl or ring)
        // For now use a placeholder signature
        let signature_b64 = "placeholder".to_string();
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

    pub async fn deliver(
        &self,
        inbox_url: &str,
        activity: &serde_json::Value,
    ) -> anyhow::Result<()> {
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

    pub async fn broadcast_to_followers(
        &self,
        activity: serde_json::Value,
        actor_id: &str,
        actor: &Actor,
    ) -> anyhow::Result<()> {
        info!("Broadcasting activity to followers of {}", actor_id);

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
            .map_err(|e| { error!("Failed to fetch followers: {}", e); anyhow::anyhow!("Database error: {}", e) })?;

        let follower_data: Vec<(Option<String>, Option<String>)> = result.take(0).unwrap_or_default();

        if follower_data.is_empty() {
            info!("No remote followers to broadcast to");
            return Ok(());
        }

        let mut inbox_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for (inbox, shared_inbox) in follower_data {
            let target_inbox = shared_inbox.clone().or(inbox.clone());
            if let Some(target) = target_inbox {
                let group_key = shared_inbox.unwrap_or_else(|| inbox.clone().unwrap_or_default());
                inbox_groups.entry(group_key).or_default().push(target);
            }
        }

        let unique_inboxes: Vec<String> = inbox_groups.keys().cloned().collect();
        info!("Broadcasting to {} unique inboxes", unique_inboxes.len());
        self.queue_delivery(activity, unique_inboxes).await?;
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
            .unwrap_or_else(|| format!("{}/users/{}", self.instance_url, local_actor.username));

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

        self.deliver_signed(follower_inbox, &accept_activity, local_actor).await
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
            .unwrap_or_else(|| format!("{}/users/{}", self.instance_url, local_actor.username));

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

        self.deliver_signed(follower_inbox, &reject_activity, local_actor).await
    }

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

        let _actor_data: serde_json::Value = response.json().await?;
        warn!("Remote actor fetching not fully implemented");
        Ok(None)
    }

    pub async fn run_delivery_worker(&self) -> anyhow::Result<()> {
        info!("Starting federation delivery worker");
        loop {
            let delivery_json: Option<String> = redis::cmd("BRPOP")
                .arg("federation:queue")
                .arg(30)
                .query_async::<_, Option<(String, String)>>(&mut self.dragonfly.clone())
                .await
                .map(|opt| opt.map(|(_, val)| val))?;

            if let Some(delivery_json) = delivery_json {
                match serde_json::from_str::<serde_json::Value>(&delivery_json) {
                    Ok(delivery) => {
                        if let Err(e) = self.process_delivery_task(delivery).await {
                            error!("Failed to process delivery: {}", e);
                        }
                    }
                    Err(e) => { error!("Invalid delivery JSON: {}", e); }
                }
            }
        }
    }

    async fn process_delivery_task(&self, delivery: serde_json::Value) -> anyhow::Result<()> {
        let inbox_url = delivery.get("inbox_url").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing inbox_url"))?;
        let activity = delivery.get("activity")
            .ok_or_else(|| anyhow::anyhow!("Missing activity"))?;
        let attempts = delivery.get("attempts").and_then(|v| v.as_i64()).unwrap_or(0);

        let actor_id = activity.get("actor").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing actor in activity"))?;

        let actor_result: Option<Actor> = self
            .surreal
            .query("SELECT * FROM user WHERE uri = $actor_id")
            .bind(("actor_id", actor_id))
            .await
            .and_then(|mut res| res.take(0))
            .ok()
            .flatten();

        let actor = actor_result
            .ok_or_else(|| anyhow::anyhow!("Actor not found or has no private key: {}", actor_id))?;

        match self.deliver_signed(inbox_url, activity, &actor).await {
            Ok(_) => {
                info!("Successfully delivered to {}", inbox_url);
                Ok(())
            }
            Err(e) => {
                error!("Delivery failed to {}: {}", inbox_url, e);
                if attempts < 5 {
                    let delay = std::time::Duration::from_secs(60 * (attempts + 1) as u64);
                    let retry_delivery = serde_json::json!({
                        "inbox_url": inbox_url,
                        "activity": activity,
                        "attempts": attempts + 1,
                        "retry_after": chrono::Utc::now() + chrono::Duration::from_std(delay)?,
                    });
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
