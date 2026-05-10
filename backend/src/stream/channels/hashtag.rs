//! Hashtag Channel
//!
//! Stream notes with a specific hashtag.

use async_trait::async_trait;
use redis::AsyncCommands;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tracing::{error, info};

use crate::{
    db::DragonflyClient,
    stream::channel::{Channel, ChannelBase, ChannelMessage},
};

/// Hashtag Channel
pub struct HashtagChannel {
    base: Arc<Mutex<ChannelBase>>,
    hashtag: String,
    redis_subscription: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl HashtagChannel {
    pub fn new(id: String, hashtag: String) -> Self {
        Self {
            base: Arc::new(Mutex::new(ChannelBase::new(id, "hashtag"))),
            hashtag,
            redis_subscription: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Channel for HashtagChannel {
    fn name(&self) -> &str {
        "hashtag"
    }
    
    fn id(&self) -> &str {
        let base = self.base.blocking_lock();
        &base.id
    }
    
    async fn init(&self, redis: &DragonflyClient) -> anyhow::Result<()> {
        let base = self.base.clone();
        let hashtag = self.hashtag.clone();
        let mut redis_client = redis.client.clone();
        
        // Subscribe to Redis Pub/Sub
        let handle = tokio::spawn(async move {
            let mut pubsub = redis_client.get_async_pubsub().await.unwrap();
            
            // Subscribe to public timeline (filter by hashtag in client)
            let _ = pubsub.subscribe("timeline:public").await;
            
            let mut stream = pubsub.on_message();
            
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap_or_default();
                
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload) {
                    // Filter by hashtag
                    let tags = data.get("tags")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_lowercase())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    
                    if tags.contains(&hashtag.to_lowercase()) {
                        let base = base.lock().await;
                        base.send(ChannelMessage::new("note", data));
                    }
                }
            }
        });
        
        let mut sub = self.redis_subscription.lock().await;
        *sub = Some(handle);
        
        info!("Hashtag channel initialized for # {}", self.hashtag);
        Ok(())
    }
    
    async fn on_message(&self, msg_type: &str, body: serde_json::Value) {
        match msg_type {
            _ => {
                info!("Hashtag channel received message: {}", msg_type);
            }
        }
    }
    
    async fn dispose(&self) {
        let mut sub = self.redis_subscription.lock().await;
        if let Some(handle) = sub.take() {
            handle.abort();
        }
        info!("Hashtag channel disposed");
    }
    
    fn sender(&self) -> &UnboundedSender<ChannelMessage> {
        panic!("Use base.send() instead");
    }
    
    fn set_sender(&self, _sender: UnboundedSender<ChannelMessage>) {
        // Set the sender for this channel
    }
    
    fn require_credential(&self) -> bool {
        false
    }
    
    fn should_share(&self) -> bool {
        true
    }
}
