//! Relay Service
//!
//! Manages ActivityPub relay servers for federation.

use std::sync::Arc;

use serde_json::json;
use tracing::{error, info, warn};

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    models::{Actor, ActorType, CreateRelayRequest, Relay, RelayId, RelayStatus},
};

/// Relay service
#[derive(Debug, Clone)]
pub struct RelayService {
    surreal: Arc<SurrealClient>,
    dragonfly: Arc<DragonflyClient>,
    instance_url: String,
}

impl RelayService {
    /// Create a new relay service
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
        instance_url: String,
    ) -> Self {
        Self {
            surreal: Arc::new(surreal),
            dragonfly: Arc::new(dragonfly),
            instance_url,
        }
    }

    /// Add a new relay
    pub async fn add_relay(&self, request: CreateRelayRequest) -> Result<Relay> {
        // Check if relay already exists
        let existing: Option<Relay> = self.surreal
            .query("SELECT * FROM relay WHERE inbox = $inbox")
            .bind(("inbox", request.inbox.clone()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default()
            .into_iter()
            .next();

        if existing.is_some() {
            return Err(AppError::Conflict("Relay already exists".to_string()));
        }

        let relay = Relay::new(request.inbox.clone());

        // Save to database
        self.surreal
            .create::<Option<Relay>>(
                ("relay", relay.id.to_string()),
            )
            .content(relay.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Added relay: {}", request.inbox);

        // Send Follow activity to relay
        self.send_follow_to_relay(&relay).await?;

        Ok(relay)
    }

    /// Remove a relay
    pub async fn remove_relay(&self, inbox: &str) -> Result<()> {
        let relay: Relay = self.surreal
            .query("SELECT * FROM relay WHERE inbox = $inbox")
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default()
            .into_iter()
            .next()
            .ok_or_else(|| AppError::NotFound("Relay not found".to_string()))?;

        // Send Undo Follow activity to relay
        self.send_undo_follow_to_relay(&relay).await?;

        // Delete from database
        self.surreal
            .query("DELETE relay WHERE inbox = $inbox")
            .bind(("inbox", inbox.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Removed relay: {}", inbox);

        Ok(())
    }

    /// List all relays
    pub async fn list_relays(&self) -> Result<Vec<Relay>> {
        let relays: Vec<Relay> = self.surreal
            .query("SELECT * FROM relay ORDER BY created_at DESC")
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(relays)
    }

    /// List accepted relays (for delivery)
    pub async fn list_accepted_relays(&self) -> Result<Vec<Relay>> {
        let relays: Vec<Relay> = self.surreal
            .query("SELECT * FROM relay WHERE status = 'accepted'")
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?
            .unwrap_or_default();

        Ok(relays)
    }

    /// Accept a relay request
    pub async fn accept_relay(&self, id: &RelayId) -> Result<Relay> {
        let mut relay: Relay = self.surreal
            .select(("relay", id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Relay not found".to_string()))?;

        relay.accept();

        self.surreal
            .update::<Option<Relay>>(
                ("relay", id.to_string()),
            )
            .merge(relay.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Accepted relay: {}", relay.inbox);

        Ok(relay)
    }

    /// Reject a relay request
    pub async fn reject_relay(&self, id: &RelayId) -> Result<Relay> {
        let mut relay: Relay = self.surreal
            .select(("relay", id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("Relay not found".to_string()))?;

        relay.reject();

        self.surreal
            .update::<Option<Relay>>(
                ("relay", id.to_string()),
            )
            .merge(relay.clone())
            .await
            .map_err(|e| AppError::Database(e))?;

        info!("Rejected relay: {}", relay.inbox);

        Ok(relay)
    }

    /// Deliver activity to all accepted relays
    pub async fn deliver_to_relays(
        &self,
        actor: &Actor,
        activity: serde_json::Value,
    ) -> Result<()> {
        let relays = self.list_accepted_relays().await?;

        if relays.is_empty() {
            return Ok(());
        }

        let mut activity = activity.clone();

        // Ensure activity has 'to' field
        if activity.get("to").is_none() {
            activity["to"] = json!(["https://www.w3.org/ns/activitystreams#Public"]);
        }

        // Add signature if needed
        let signed_activity = self.sign_activity(actor, activity).await?;

        // Queue deliveries
        for relay in relays {
            self.queue_delivery(signed_activity.clone(), &relay.inbox).await?;
        }

        Ok(())
    }

    /// Send Follow activity to relay (to establish relay connection)
    async fn send_follow_to_relay(&self, relay: &Relay) -> Result<()> {
        let follow_activity = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Follow",
            "actor": format!("{}/users/relay.actor", self.instance_url),
            "object": "https://www.w3.org/ns/activitystreams#Public",
            "id": format!("{}/activities/follow/{}", self.instance_url, uuid::Uuid::new_v4()),
        });

        self.queue_delivery(follow_activity, &relay.inbox).await?;

        Ok(())
    }

    /// Send Undo Follow activity to relay
    async fn send_undo_follow_to_relay(&self, relay: &Relay) -> Result<()> {
        let undo_activity = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Undo",
            "actor": format!("{}/users/relay.actor", self.instance_url),
            "object": {
                "type": "Follow",
                "actor": format!("{}/users/relay.actor", self.instance_url),
                "object": "https://www.w3.org/ns/activitystreams#Public",
            },
            "id": format!("{}/activities/undo/{}", self.instance_url, uuid::Uuid::new_v4()),
        });

        self.queue_delivery(undo_activity, &relay.inbox).await?;

        Ok(())
    }

    /// Queue a delivery to an inbox
    async fn queue_delivery(
        &self,
        activity: serde_json::Value,
        inbox_url: &str,
    ) -> Result<()> {
        let delivery = json!({
            "inbox_url": inbox_url,
            "activity": activity,
            "attempts": 0,
        });

        // Dragonfly (Redis) list
        redis::cmd("LPUSH")
            .arg("federation:queue")
            .arg(delivery.to_string())
            .query_async::<_, ()>(&mut self.dragonfly.clone())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to queue delivery: {}", e)))?;

        info!("Queued delivery to relay: {}", inbox_url);

        Ok(())
    }

    /// Sign an activity with actor's key
    async fn sign_activity(
        &self,
        actor: &Actor,
        activity: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // TODO: Implement proper LD signature
        // For now, return activity as-is (the federation worker will handle signing)
        Ok(activity)
    }
}
