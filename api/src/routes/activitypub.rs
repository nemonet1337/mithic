//! ActivityPub 受信基盤 (TODO Phase F1)
//!
//! WebFinger / Actor / NodeInfo の公開エンドポイントと、
//! Follow/Undo を処理する inbox を提供する。

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{info, warn};

use mithic_core::models::actor::Actor;
use mithic_core::models::notification::Notification;
use mithic_core::{AppError, Result};
use mithic_db::queries::{create_actor, get_actor_by_username, unfollow_user};

use crate::services::note::publish_notification;
use crate::services::relationship;
use crate::state::AppState;

const ACTIVITY_JSON: &str = "application/activity+json";
const JRD_JSON: &str = "application/jrd+json";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/.well-known/webfinger", get(webfinger))
        .route("/.well-known/nodeinfo", get(nodeinfo_discovery))
        .route("/nodeinfo/2.0", get(nodeinfo))
        .route("/users/{username}", get(actor_document))
        .route("/users/{username}/inbox", post(inbox))
        .route("/inbox", post(shared_inbox))
}

fn activity_response(body: Value) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, ACTIVITY_JSON)],
        Json(body),
    )
        .into_response()
}

fn instance_host(instance_url: &str) -> String {
    url::Url::parse(instance_url)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_else(|| "localhost".to_string())
}

// ---------------------------------------------------------------------------
// WebFinger
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

async fn webfinger(
    State(state): State<AppState>,
    Query(query): Query<WebfingerQuery>,
) -> Result<Response> {
    let host = instance_host(&state.config().instance_url);

    // acct:user@host 形式をパース
    let acct = query
        .resource
        .strip_prefix("acct:")
        .unwrap_or(&query.resource);
    let (username, resource_host) = acct.split_once('@').unwrap_or((acct, host.as_str()));

    if resource_host != host {
        return Err(AppError::NotFound("Unknown host".to_string()));
    }

    let actor = get_actor_by_username(state.surreal(), username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter(|a| a.is_local())
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let actor_uri = actor.actor_uri(&state.config().instance_url);
    let body = json!({
        "subject": format!("acct:{}@{}", actor.username, host),
        "aliases": [actor_uri],
        "links": [
            {
                "rel": "self",
                "type": ACTIVITY_JSON,
                "href": actor_uri
            }
        ]
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, JRD_JSON)],
        Json(body),
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// NodeInfo
// ---------------------------------------------------------------------------

async fn nodeinfo_discovery(State(state): State<AppState>) -> Json<Value> {
    let instance_url = &state.config().instance_url;
    Json(json!({
        "links": [
            {
                "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
                "href": format!("{instance_url}/nodeinfo/2.0")
            }
        ]
    }))
}

async fn nodeinfo(State(state): State<AppState>) -> Json<Value> {
    let mut users_total: Option<i64> = None;
    let mut notes_total: Option<i64> = None;

    if let Ok(mut res) = state
        .surreal()
        .query("SELECT count() AS c FROM user WHERE host = None GROUP ALL;")
        .await
        && let Ok(rows) = res.take::<Vec<surrealdb::types::Value>>(0)
    {
        users_total = rows
            .into_iter()
            .next()
            .and_then(|v| v.into_json_value().get("c").and_then(|c| c.as_i64()));
    }
    if let Ok(mut res) = state
        .surreal()
        .query("SELECT count() AS c FROM note GROUP ALL;")
        .await
        && let Ok(rows) = res.take::<Vec<surrealdb::types::Value>>(0)
    {
        notes_total = rows
            .into_iter()
            .next()
            .and_then(|v| v.into_json_value().get("c").and_then(|c| c.as_i64()));
    }

    Json(json!({
        "version": "2.0",
        "software": {
            "name": "mithic",
            "version": env!("CARGO_PKG_VERSION")
        },
        "protocols": ["activitypub"],
        "services": { "inbound": [], "outbound": [] },
        "openRegistrations": true,
        "usage": {
            "users": { "total": users_total.unwrap_or(0) },
            "localPosts": notes_total.unwrap_or(0)
        },
        "metadata": {
            "nodeName": state.config().instance_name
        }
    }))
}

// ---------------------------------------------------------------------------
// Actor
// ---------------------------------------------------------------------------

fn build_actor_document(actor: &Actor, instance_url: &str) -> Value {
    let actor_uri = actor.actor_uri(instance_url);
    json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1"
        ],
        "id": actor_uri,
        "type": if actor.is_bot { "Service" } else { "Person" },
        "preferredUsername": actor.username,
        "name": actor.name,
        "summary": actor.bio,
        "inbox": actor.inbox_url(instance_url),
        "outbox": actor.outbox_url(instance_url),
        "followers": format!("{actor_uri}/followers"),
        "following": format!("{actor_uri}/following"),
        "sharedInbox": format!("{instance_url}/inbox"),
        "endpoints": { "sharedInbox": format!("{instance_url}/inbox") },
        "manuallyApprovesFollowers": actor.is_locked,
        "publicKey": {
            "id": format!("{actor_uri}#main-key"),
            "owner": actor_uri,
            "publicKeyPem": actor.public_key
        },
        "icon": actor.avatar_url.as_ref().map(|url| json!({ "type": "Image", "url": url })),
        "published": actor.created_at.to_rfc3339()
    })
}

async fn actor_document(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Response> {
    let actor = get_actor_by_username(state.surreal(), &username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter(|a| a.is_local())
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(activity_response(build_actor_document(
        &actor,
        &state.config().instance_url,
    )))
}

// ---------------------------------------------------------------------------
// Inbox
// ---------------------------------------------------------------------------

async fn shared_inbox(
    State(state): State<AppState>,
    Json(activity): Json<Value>,
) -> Result<StatusCode> {
    process_activity(&state, None, activity).await
}

async fn inbox(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(activity): Json<Value>,
) -> Result<StatusCode> {
    process_activity(&state, Some(username), activity).await
}

/// リモートアクターを取得し、未知なら永続化する
async fn resolve_remote_actor(state: &AppState, actor_uri: &str) -> Result<Actor> {
    // 既知のアクターか確認
    let mut res = state
        .surreal()
        .query("SELECT * FROM user WHERE uri = $uri LIMIT 1;")
        .bind(("uri", actor_uri.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows: Vec<surrealdb::types::Value> =
        res.take(0).map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(actor) = rows.into_iter().next().and_then(|v| {
        let mut json = v.into_json_value();
        mithic_db::queries::strip_record_prefixes(&mut json);
        serde_json::from_value::<Actor>(json).ok()
    }) {
        return Ok(actor);
    }

    // 未知のアクターはリモートから取得して保存
    let remote = state
        .federation_service()
        .fetch_remote_actor(actor_uri)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Remote actor not found".to_string()))?;

    create_actor(state.surreal(), &remote)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

async fn process_activity(
    state: &AppState,
    inbox_owner: Option<String>,
    activity: Value,
) -> Result<StatusCode> {
    let activity_type = activity
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let remote_actor_uri = activity
        .get("actor")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing actor".to_string()))?
        .to_string();

    info!(
        "Inbox activity: {} from {} (owner: {:?})",
        activity_type, remote_actor_uri, inbox_owner
    );

    match activity_type.as_str() {
        "Follow" => handle_follow(state, &remote_actor_uri, &activity).await,
        "Undo" => handle_undo(state, &remote_actor_uri, &activity).await,
        // Create/Like/Announce/Delete/Update/Accept/Reject は受理のみ (Phase F3 で実装)
        _ => {
            warn!("Unhandled activity type: {}", activity_type);
            Ok(StatusCode::ACCEPTED)
        }
    }
}

/// 対象ローカルユーザーを activity の object URI から特定する
async fn resolve_local_object(state: &AppState, object_uri: &str) -> Result<Actor> {
    let instance_url = &state.config().instance_url;
    let username = object_uri
        .strip_prefix(&format!("{instance_url}/users/"))
        .ok_or_else(|| AppError::NotFound("Object is not a local user".to_string()))?;

    get_actor_by_username(state.surreal(), username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter(|a| a.is_local())
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))
}

async fn handle_follow(
    state: &AppState,
    remote_actor_uri: &str,
    activity: &Value,
) -> Result<StatusCode> {
    let object_uri = activity
        .get("object")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing object".to_string()))?;

    let local_actor = resolve_local_object(state, object_uri).await?;
    let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;

    relationship::follow(state.surreal(), &remote_actor.id, &local_actor.id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // フォロー通知
    let notif = Notification::follow(local_actor.id, remote_actor.id);
    publish_notification(state, &notif, Some(&remote_actor), None).await;

    // Accept を非同期で返送
    if let Some(inbox) = remote_actor.inbox.clone() {
        let federation = state.federation_service().clone();
        let remote_uri = remote_actor_uri.to_string();
        tokio::spawn(async move {
            if let Err(e) = federation
                .send_accept_follow(&inbox, &remote_uri, &local_actor)
                .await
            {
                warn!("Failed to send Accept Follow: {}", e);
            }
        });
    }

    Ok(StatusCode::ACCEPTED)
}

async fn handle_undo(
    state: &AppState,
    remote_actor_uri: &str,
    activity: &Value,
) -> Result<StatusCode> {
    let object = activity
        .get("object")
        .ok_or_else(|| AppError::Validation("Missing object".to_string()))?;

    let object_type = object
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if object_type != "Follow" {
        warn!("Unhandled Undo object type: {}", object_type);
        return Ok(StatusCode::ACCEPTED);
    }

    let followee_uri = object
        .get("object")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing follow object".to_string()))?;

    let local_actor = resolve_local_object(state, followee_uri).await?;
    let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;

    unfollow_user(state.surreal(), &remote_actor.id, &local_actor.id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(StatusCode::ACCEPTED)
}
