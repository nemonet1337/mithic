use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    error::{AppError, Result},
    models::Actor,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ActorQuery {
    #[serde(default)]
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct ActivityPubActor {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub actor_type: String,
    pub preferred_username: String,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub inbox: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_inbox: Option<String>,
    pub outbox: String,
    pub followers: String,
    pub following: String,
    pub featured: String,
    pub url: String,
    pub manually_approves_followers: bool,
    pub discoverable: bool,
    pub public_key: PublicKey,
    pub icon: Option<Image>,
    pub image: Option<Image>,
}

#[derive(Debug, Serialize)]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}

#[derive(Debug, Serialize)]
pub struct Image {
    #[serde(rename = "type")]
    pub image_type: String,
    pub url: String,
}

/// Actor取得エンドポイント
pub async fn get_actor(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(_query): Query<ActorQuery>,
) -> Result<Json<ActivityPubActor>> {
    let mut result = state
        .surreal()
        .query("SELECT * FROM user WHERE username_lower = $username")
        .bind(("username", username.to_lowercase()))
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            AppError::Database(e)
        })?;

    let actor: Option<Actor> = result.take(0).map_err(|e| {
        AppError::Internal(format!("Failed to deserialize actor: {}", e))
    })?;

    let actor = actor.ok_or_else(|| AppError::NotFound("Actor not found".to_string()))?;

    let instance_url = &state.config().instance_url;
    let actor_url = format!("{}/users/{}", instance_url, username);

    let icon = actor.avatar_url.map(|url| Image {
        image_type: "Image".to_string(),
        url,
    });

    let image = actor.banner_url.map(|url| Image {
        image_type: "Image".to_string(),
        url,
    });

    let public_key_pem = actor.public_key.unwrap_or_default();

    Ok(Json(ActivityPubActor {
        context: vec![
            "https://www.w3.org/ns/activitystreams".to_string(),
            "https://w3id.org/security/v1".to_string(),
        ],
        id: actor_url.clone(),
        actor_type: "Person".to_string(),
        preferred_username: actor.username,
        name: actor.name,
        summary: actor.bio,
        inbox: format!("{}/inbox", actor_url),
        shared_inbox: Some(format!("{}/inbox", instance_url)),
        outbox: format!("{}/outbox", actor_url),
        followers: format!("{}/followers", actor_url),
        following: format!("{}/following", actor_url),
        featured: format!("{}/featured", actor_url),
        url: actor_url.clone(),
        manually_approves_followers: actor.is_locked,
        discoverable: true,
        public_key: PublicKey {
            id: format!("{}#main-key", actor_url),
            owner: actor_url,
            public_key_pem,
        },
        icon,
        image,
    }))
}
