//! Home Timeline Channel
//!
//! Stream notes from followed users.

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

/// Home Timeline Channel
pub struct HomeTimelineChannel {
    base: Arc<Mutex<ChannelBase>>,
    user_id: Option<String>,
    redis_subscription: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl HomeTimelineChannel {
    pub fn new(id: String, user_id: Option<String>) -> Self {
        Self {
            base: Arc::new(Mutex::new(ChannelBase::new(id, "homeTimeline"))),
            user_id,
            redis_subscription: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl Channel for HomeTimelineChannel {
    fn name(&self) -> &str {
        "homeTimeline"
    }
    
    fn id(&self) -> &str {
        let base = self.base.blocking_lock();
        &base.id
    }
    
    async fn init(&self, redis: &DragonflyClient) -> anyhow::Result<()> {
        if self.user_id.is_none() {
            return Err(anyhow::anyhow!("Home timeline requires authentication"));
        }
        
        let user_id = self.user_id.clone().unwrap();
        let base = self.base.clone();
        let mut redis_client = redis.client.clone();
        
        // Subscribe to Redis Pub/Sub
        let handle = tokio::spawn(async move {
            let mut pubsub = redis_client.get_async_pubsub().await.unwrap();
            
            // Subscribe to user's channel
            let channel = format!("user:{}:", user_id);
            let _ = pubsub.subscribe(&channel).await;
            
            // Subscribe to timeline:home
            let _ = pubsub.subscribe("timeline:home").await;
            
            let mut stream = pubsub.on_message();
            
            while let Some(msg) = stream.next().await {
                let payload: String = msg.get_payload().unwrap_or_default();
                
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&payload) {
                    let note_type = data.get("type").and_then(|v| v.as_str()).unwrap_or("note");
                    
                    if note_type == "note" {
                        let base = base.lock().await;
                        base.send(ChannelMessage::new("note", data));
                    }
                }
            }
        });
        
        let mut sub = self.redis_subscription.lock().await;
        *sub = Some(handle);
        
        info!("Home timeline channel initialized for user {}", user_id);
        Ok(())
    }
    
    async fn on_message(&self, msg_type: &str, body: serde_json::Value) {
        match msg_type {
            "read" => {
                // Mark note as read
                info!("Marking note as read: {:?}", body);
            }
            _ => {
                info!("Unknown message type: {}", msg_type);
            }
        }
    }
    
    async fn dispose(&self) {
        let mut sub = self.redis_subscription.lock().await;
        if let Some(handle) = sub.take() {
            handle.abort();
        }
        info!("Home timeline channel disposed");
    }
    
    fn sender(&self) -> &UnboundedSender<ChannelMessage> {
        // Return a reference to the sender (this is a simplified implementation)
        panic!("Use base.send() instead");
    }
    
    fn set_sender(&self, _sender: UnboundedSender<ChannelMessage>) {
        // Set the sender for this channel
    }
    
    fn require_credential(&self) -> bool {
        true
    }
    
    fn should_share(&self) -> bool {
        true
    }
}
