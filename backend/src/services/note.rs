//! Note creation service
//!
//! Handles the complete flow of creating a note including:
//! - Mention extraction and notification
//! - Hashtag extraction and updating
//! - Emoji extraction
//! - ActivityPub delivery
//! - Stream publication

use chrono::Utc;
use std::collections::HashMap;
use tracing::{error, info, warn};
use ulid::Ulid;

use crate::{
    db::{DragonflyClient, SurrealClient},
    error::{AppError, Result},
    misc::{extract_emojis, extract_hashtags, extract_mentions, Mention},
    models::{Actor, ActorId, Note, NoteId, NoteVisibility, Notification, NotificationType},
    services::FederationService,
};

/// Options for creating a note
#[derive(Debug, Clone)]
pub struct CreateNoteOptions {
    pub text: Option<String>,
    pub cw: Option<String>,
    pub reply_id: Option<NoteId>,
    pub renote_id: Option<NoteId>,
    pub visibility: NoteVisibility,
    pub visible_user_ids: Vec<ActorId>,
    pub file_ids: Vec<String>,
    pub poll: Option<CreatePollOptions>,
}

/// Options for creating a poll
#[derive(Debug, Clone)]
pub struct CreatePollOptions {
    pub choices: Vec<String>,
    pub expires_in: Option<i32>,
    pub multiple: bool,
}

/// Note creation service
pub struct NoteService {
    surreal: SurrealClient,
    dragonfly: DragonflyClient,
    federation: FederationService,
    instance_url: String,
}

impl NoteService {
    pub fn new(
        surreal: SurrealClient,
        dragonfly: DragonflyClient,
        federation: FederationService,
        instance_url: String,
    ) -> Self {
        Self {
            surreal,
            dragonfly,
            federation,
            instance_url,
        }
    }

    /// Create a new note with full processing
    pub async fn create_note(
        &self,
        actor_id: ActorId,
        options: CreateNoteOptions,
    ) -> Result<Note> {
        let actor: Option<Actor> = self
            .surreal
            .query("SELECT * FROM user WHERE id = $id")
            .bind(("id", actor_id.to_string()))
            .await
            .and_then(|mut res| res.take(0))
            .map_err(|e| AppError::Database(e))?;

        let actor = actor.ok_or_else(|| AppError::NotFound("Actor not found".to_string()))?;

        // Extract mentions, hashtags, emojis
        let text = options.text.as_deref().unwrap_or("");
        let mentions = extract_mentions::extract_unique_mentions(text);
        let hashtags = extract_hashtags(text);
        let emojis = extract_emojis(text);

        // Create note
        let note_id = NoteId::new();
        let now = Utc::now();

        // Build reactions HashMap
        let reactions: HashMap<String, i32> = HashMap::new();

        let note = Note {
            id: note_id,
            created_at: now,
            reply_id: options.reply_id,
            renote_id: options.renote_id,
            text: options.text,
            cw: options.cw,
            actor_id: actor_id.clone(),
            renote_count: 0,
            replies_count: 0,
            reactions,
            visibility: options.visibility.clone(),
            uri: Some(format!("{}/notes/{}", self.instance_url, note_id)),
            file_ids: options.file_ids,
            visible_user_ids: options.visible_user_ids.clone(),
            mentions: mentions.iter().map(|_| actor_id.clone()).collect(), // TODO: resolve actual user IDs
            emojis: emojis.clone(),
            tags: hashtags.clone(),
            has_poll: options.poll.is_some(),
            actor_host: None,
            reply_actor_id: None,
            renote_actor_id: None,
        };

        // Save note
        self.surreal
            .create(("note", note_id.to_string()))
            .content(note.clone())
            .await
            .map_err(|e| {
                error!("Failed to create note: {}", e);
                AppError::Database(e)
            })?;

        // Update actor note count
        self.surreal
            .query("UPDATE user SET notes_count = notes_count + 1 WHERE id = $id")
            .bind(("id", actor_id.to_string()))
            .await
            .map_err(|e| AppError::Database(e))?;

        // Create notifications for mentions
        self.create_mention_notifications(&actor, &mentions, &note).await?;

        // Update hashtags
        self.update_hashtags(&hashtags).await?;

        // Deliver to followers if public
        if matches!(options.visibility, NoteVisibility::Public | NoteVisibility::Home) {
            if let Err(e) = self.deliver_to_followers(&actor, &note).await {
                warn!("Failed to deliver to followers: {}", e);
                // Don't fail the note creation if delivery fails
            }
        }

        // Publish to stream
        self.publish_to_stream(&note).await?;

        info!(
            "Created note {} by actor {} with {} mentions, {} hashtags",
            note_id,
            actor_id,
            mentions.len(),
            hashtags.len()
        );

        Ok(note)
    }

    /// Create notifications for mentions
    async fn create_mention_notifications(
        &self,
        actor: &Actor,
        mentions: &[Mention],
        note: &Note,
    ) -> Result<()> {
        for mention in mentions {
            // Try to find the mentioned user
            let target_user: Option<Actor> = if mention.host.is_empty() {
                // Local user
                self.surreal
                    .query("SELECT * FROM user WHERE username_lower = $username")
                    .bind(("username", mention.username.to_lowercase()))
                    .await
                    .and_then(|mut res| res.take(0))
                    .ok()
                    .flatten()
            } else {
                // Remote user - would need to resolve
                None
            };

            if let Some(target) = target_user {
                // Don't notify if it's the same user
                if target.id == actor.id {
                    continue;
                }

                let notification = Notification {
                    id: Ulid::new(),
                    created_at: Utc::now(),
                    notification_type: NotificationType::Mention,
                    recipient_id: target.id.clone(),
                    sender_id: Some(actor.id.clone()),
                    note_id: Some(note.id.clone()),
                    reaction: None,
                    is_read: false,
                };

                self.surreal
                    .create(("notification", notification.id.to_string()))
                    .content(notification)
                    .await
                    .map_err(|e| {
                        error!("Failed to create mention notification: {}", e);
                        AppError::Database(e)
                    })?;

                // Publish notification to stream
                let _ = self
                    .dragonfly
                    .publish(
                        &format!("user:{}", target.id),
                        serde_json::json!({
                            "type": "notification",
                            "notification_type": "mention",
                            "note_id": note.id.to_string(),
                        })
                        .to_string(),
                    )
                    .await;
            }
        }

        Ok(())
    }

    /// Update hashtag statistics
    async fn update_hashtags(&self, hashtags: &[String]) -> Result<()> {
        for tag in hashtags {
            // Try to find existing hashtag
            let existing: Option<serde_json::Value> = self
                .surreal
                .query("SELECT * FROM hashtag WHERE name = $name")
                .bind(("name", tag.clone()))
                .await
                .and_then(|mut res| res.take(0))
                .ok()
                .flatten();

            if existing.is_some() {
                // Update count
                self.surreal
                    .query("UPDATE hashtag SET count = count + 1, updated_at = time::now() WHERE name = $name")
                    .bind(("name", tag.clone()))
                    .await
                    .map_err(|e| AppError::Database(e))?;
            } else {
                // Create new hashtag
                let hashtag_doc = serde_json::json!({
                    "id": Ulid::new().to_string(),
                    "name": tag,
                    "count": 1,
                    "created_at": Utc::now().to_rfc3339(),
                    "updated_at": Utc::now().to_rfc3339(),
                });

                self.surreal
                    .create("hashtag")
                    .content(hashtag_doc)
                    .await
                    .map_err(|e| AppError::Database(e))?;
            }
        }

        Ok(())
    }

    /// Deliver note to followers via ActivityPub
    async fn deliver_to_followers(&self, actor: &Actor, note: &Note) -> anyhow::Result<()> {
        // Get followers
        let followers: Vec<serde_json::Value> = self
            .surreal
            .query("SELECT out FROM follow WHERE in = $actor_id")
            .bind(("actor_id", actor.id.to_string()))
            .await
            .and_then(|mut res| res.take(0))?;

        if followers.is_empty() {
            return Ok(());
        }

        // Build Create activity
        let note_url = format!("{}/notes/{}", self.instance_url, note.id);
        let actor_url = format!("{}/users/{}", self.instance_url, actor.id);

        let create_activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": format!("{}/activity", note_url),
            "type": "Create",
            "actor": actor_url,
            "published": note.created_at.to_rfc3339(),
            "object": {
                "id": note_url,
                "type": "Note",
                "attributedTo": actor_url,
                "content": note.text.clone().unwrap_or_default(),
                "published": note.created_at.to_rfc3339(),
                "to": ["https://www.w3.org/ns/activitystreams#Public"],
            }
        });

        // Queue delivery for each follower
        for follower in followers {
            if let Some(follower_id) = follower.get("out").and_then(|v| v.as_str()) {
                // Get follower's inbox
                let follower_actor: Option<Actor> = self
                    .surreal
                    .select(("user", follower_id.to_string()))
                    .await
                    .ok()
                    .flatten();

                if let Some(f) = follower_actor {
                    if let Some(inbox) = f.inbox {
                        self.federation
                            .queue_delivery(create_activity.clone(), vec![inbox])
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Publish note to streaming channels
    async fn publish_to_stream(&self, note: &Note) -> Result<()> {
        let message = serde_json::json!({
            "type": "note",
            "note_id": note.id.to_string(),
            "actor_id": note.actor_id.to_string(),
            "visibility": serde_json::to_string(&note.visibility).unwrap_or_default(),
        });

        // Publish to timeline channels based on visibility
        match note.visibility {
            NoteVisibility::Public => {
                self.dragonfly
                    .publish("timeline:public", message.to_string())
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
            NoteVisibility::Home => {
                self.dragonfly
                    .publish("timeline:home", message.to_string())
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
            _ => {}
        }

        // Publish to user's channel
        self.dragonfly
            .publish(
                &format!("user:{}", note.actor_id),
                message.to_string(),
            )
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }
}

/// ノートを既読にする
pub async fn mark_note_as_read(
    surreal: &SurrealClient,
    note_id: NoteId,
    user_id: ActorId,
) -> Result<()> {
    surreal
        .query("DELETE note_unread WHERE note_id = $note_id AND user_id = $user_id")
        .bind(("note_id", note_id.to_string()))
        .bind(("user_id", user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;
    Ok(())
}

/// 全ノートを既読にする
pub async fn mark_all_notes_as_read(surreal: &SurrealClient, user_id: ActorId) -> Result<()> {
    surreal
        .query("DELETE note_unread WHERE user_id = $user_id")
        .bind(("user_id", user_id.to_string()))
        .await
        .map_err(|e| AppError::Database(e))?;
    Ok(())
}

/// ノートを未読に追加する
pub async fn mark_note_as_unread(
    surreal: &SurrealClient,
    note_id: NoteId,
    user_id: ActorId,
    is_specified: bool,
    is_mention: bool,
) -> Result<()> {
    let unread_id = Ulid::new();
    surreal
        .query("CREATE note_unread:$id SET note_id = $note_id, user_id = $user_id, is_specified = $is_specified, is_mention = $is_mention, created_at = time::now()")
        .bind(("id", unread_id.to_string()))
        .bind(("note_id", note_id.to_string()))
        .bind(("user_id", user_id.to_string()))
        .bind(("is_specified", is_specified))
        .bind(("is_mention", is_mention))
        .await
        .map_err(|e| AppError::Database(e))?;
    Ok(())
}

/// ハッシュタグ統計を更新する
pub async fn update_hashtag(surreal: &SurrealClient, tag: &str, user_id: ActorId) -> Result<()> {
    let tag_lower = tag.to_lowercase();

    // Upsert hashtag record
    surreal
        .query("IF (SELECT count() FROM hashtag WHERE name = $name)[0].count = 0 THEN CREATE hashtag SET name = $name, count = 1, created_at = time::now() ELSE UPDATE hashtag SET count = count + 1 WHERE name = $name END")
        .bind(("name", tag_lower))
        .await
        .map_err(|e| AppError::Database(e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_note_options() {
        let options = CreateNoteOptions {
            text: Some("Hello world".to_string()),
            cw: None,
            reply_id: None,
            renote_id: None,
            visibility: NoteVisibility::Public,
            visible_user_ids: Vec::new(),
            file_ids: Vec::new(),
            poll: None,
        };

        assert_eq!(options.text, Some("Hello world".to_string()));
    }
}
