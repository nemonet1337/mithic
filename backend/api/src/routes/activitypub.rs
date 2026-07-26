//! ActivityPub 受信基盤
//!
//! WebFinger / Actor / NodeInfo と inbox (Follow / Like / Create / Announce / Undo 等)。
//! Misskey 拡張: 絵文字リアクション (`_misskey_reaction` / content)、引用 (`quoteUrl`)。

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{info, warn};

use mithic_core::models::actor::Actor;
use mithic_core::models::note::{Note, NoteVisibility};
use mithic_core::models::notification::Notification;
use mithic_core::{AppError, Result};
use mithic_db::queries::{
    add_reaction, create_actor, create_note, get_actor_by_username, get_note_by_id, get_note_by_uri,
    remove_all_reactions_by_actor, unfollow_user,
};

use crate::middleware::verify_http_signature;
use crate::services::note::publish_notification;
use crate::services::relationship;
use crate::state::AppState;

/// Misskey 互換: content 無し Like のデフォルトリアクション
const DEFAULT_REACTION: &str = "⭐";

const ACTIVITY_JSON: &str = "application/activity+json";
const JRD_JSON: &str = "application/jrd+json";

pub fn router(state: AppState) -> Router<AppState> {
    let inbox_routes = Router::new()
        .route("/users/{username}/inbox", post(inbox))
        .route("/inbox", post(shared_inbox))
        .layer(from_fn_with_state(state, verify_http_signature));

    Router::new()
        .route("/.well-known/webfinger", get(webfinger))
        .route("/.well-known/nodeinfo", get(nodeinfo_discovery))
        .route("/nodeinfo/2.0", get(nodeinfo))
        .route("/users/{username}", get(actor_document))
        .route("/@{username}", get(actor_redirect))
        .route("/users/{username}/outbox", get(outbox))
        .route("/users/{username}/followers", get(followers))
        .route("/users/{username}/following", get(following))
        .route("/users/{username}/collections/featured", get(featured))
        .merge(inbox_routes)
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
            "https://w3id.org/security/v1",
            // Misskey 互換拡張 (受信側が解釈できるように宣言)
            {
                "misskey": "https://misskey-hub.net/ns#",
                "_misskey_reaction": "misskey:_misskey_reaction",
                "_misskey_quote": "misskey:_misskey_quote",
                "quoteUrl": "http://fedibird.com/ns#quoteUrl",
                "quoteUri": "http://fedibird.com/ns#quoteUri",
                "sensitive": "as:sensitive",
                "toot": "http://joinmastodon.org/ns#",
                "Emoji": "toot:Emoji",
                "Hashtag": "as:Hashtag"
            }
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

async fn actor_redirect(
    Path(username): Path<String>,
) -> impl IntoResponse {
    (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, format!("/users/{}", username))])
}

async fn outbox(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let actor = get_actor_by_username(state.surreal(), &username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter(|a| a.is_local())
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "OrderedCollection",
        "totalItems": actor.notes_count,
        "id": format!("{}/users/{}/outbox", state.config().instance_url, actor.username),
    })))
}

async fn followers(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let actor = get_actor_by_username(state.surreal(), &username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter(|a| a.is_local())
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "OrderedCollection",
        "totalItems": actor.followers_count,
        "id": format!("{}/users/{}/followers", state.config().instance_url, actor.username),
    })))
}

async fn following(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let actor = get_actor_by_username(state.surreal(), &username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .filter(|a| a.is_local())
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "OrderedCollection",
        "totalItems": actor.following_count,
        "id": format!("{}/users/{}/following", state.config().instance_url, actor.username),
    })))
}

async fn featured(
    State(_state): State<AppState>,
    Path(_username): Path<String>,
) -> Result<Json<Value>> {
    Ok(Json(json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "OrderedCollection",
        "totalItems": 0
    })))
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

    // id と取得元 URL の一致は parse_remote_actor / fetch 側で検証済み
    if remote.uri.as_deref() != Some(actor_uri) {
        return Err(AppError::Validation(
            "Remote actor id does not match fetch URL".to_string(),
        ));
    }

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
        "Like" => handle_like(state, &remote_actor_uri, &activity).await,
        "Create" => handle_create(state, &remote_actor_uri, &activity).await,
        "Announce" => handle_announce(state, &remote_actor_uri, &activity).await,
        "Delete" | "Update" | "Accept" | "Reject" | "Block" => {
            // 受理のみ (永続化は後続)
            Ok(StatusCode::ACCEPTED)
        }
        _ => {
            warn!("Unhandled activity type: {activity_type}");
            Ok(StatusCode::ACCEPTED)
        }
    }
}

// ---------------------------------------------------------------------------
// Note / reaction helpers
// ---------------------------------------------------------------------------

/// ローカルノート URI (`{instance}/notes/{id}`) または remote `uri` からノートを解決
async fn resolve_note(state: &AppState, note_uri: &str) -> Result<Option<Note>> {
    let instance = state.config().instance_url.trim_end_matches('/');
    if let Some(id_str) = note_uri.strip_prefix(&format!("{instance}/notes/")) {
        if let Ok(id) = id_str.parse() {
            return get_note_by_id(state.surreal(), &id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()));
        }
    }
    get_note_by_uri(state.surreal(), note_uri)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn object_id(value: &Value) -> Option<&str> {
    match value {
        Value::String(s) => Some(s.as_str()),
        Value::Object(_) => value.get("id").and_then(|v| v.as_str()),
        _ => None,
    }
}

/// Like からリアクション文字列を抽出 (Misskey content / `_misskey_reaction` / デフォルト)
fn extract_reaction(activity: &Value) -> String {
    activity
        .get("_misskey_reaction")
        .and_then(|v| v.as_str())
        .or_else(|| activity.get("content").and_then(|v| v.as_str()))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_REACTION.to_string())
}

/// tag 配列から Emoji を remote_emoji にキャッシュ
async fn cache_emoji_tags(state: &AppState, tags: &Value, host: Option<&str>) {
    let Some(arr) = tags.as_array() else { return };
    for tag in arr {
        let typ = tag.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if typ != "Emoji" {
            continue;
        }
        let name = tag
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim_matches(':')
            .to_string();
        let url = tag
            .pointer("/icon/url")
            .and_then(|v| v.as_str())
            .or_else(|| tag.get("icon").and_then(|i| i.get("url")).and_then(|u| u.as_str()))
            .unwrap_or("")
            .to_string();
        if name.is_empty() || url.is_empty() {
            continue;
        }
        let id = ulid::Ulid::new().to_string();
        let _ = state
            .surreal()
            .query(
                "
                INSERT INTO remote_emoji {
                    id: $id,
                    name: $name,
                    url: $url,
                    host: $host,
                    created_at: time::now()
                }
                ON DUPLICATE KEY UPDATE url = $url;
                ",
            )
            .bind(("id", id))
            .bind(("name", name))
            .bind(("url", url))
            .bind(("host", host.map(String::from)))
            .await;
    }
}

fn extract_quote_uri(object: &Value) -> Option<String> {
    object
        .get("quoteUrl")
        .or_else(|| object.get("_misskey_quote"))
        .or_else(|| object.get("quoteUri"))
        .or_else(|| object.get("quote"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn host_from_uri(uri: &str) -> Option<String> {
    url::Url::parse(uri)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
}

async fn handle_like(
    state: &AppState,
    remote_actor_uri: &str,
    activity: &Value,
) -> Result<StatusCode> {
    let note_uri = activity
        .get("object")
        .and_then(object_id)
        .ok_or_else(|| AppError::Validation("Like missing object".to_string()))?;

    let Some(note) = resolve_note(state, note_uri).await? else {
        // 未知のノートへの Like は無視 (受理のみ)
        return Ok(StatusCode::ACCEPTED);
    };

    let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;
    let reaction = extract_reaction(activity);

    if let Some(tags) = activity.get("tag") {
        cache_emoji_tags(state, tags, remote_actor.host.as_deref()).await;
    }

    // 同一アクターの既存リアクションを置換
    let _ = remove_all_reactions_by_actor(
        state.surreal(),
        &note.id.to_string(),
        &remote_actor.id.to_string(),
    )
    .await;

    if let Err(e) = add_reaction(
        state.surreal(),
        &note.id.to_string(),
        &remote_actor.id.to_string(),
        &reaction,
    )
    .await
    {
        // UNIQUE 衝突等は警告のみ
        warn!("Failed to add remote reaction: {e}");
        return Ok(StatusCode::ACCEPTED);
    }

    // ローカル投稿者への通知
    if note.actor_id != remote_actor.id {
        let notif =
            Notification::reaction(note.actor_id, remote_actor.id, note.id, reaction.clone());
        publish_notification(state, &notif, Some(&remote_actor), None).await;
    }

    Ok(StatusCode::ACCEPTED)
}

async fn handle_create(
    state: &AppState,
    remote_actor_uri: &str,
    activity: &Value,
) -> Result<StatusCode> {
    let object = activity
        .get("object")
        .ok_or_else(|| AppError::Validation("Create missing object".to_string()))?;

    // 埋め込みオブジェクト or 参照のみ
    let object = if object.is_string() {
        // リモート取得はフェデレーションサービスに任せる (参照のみはスキップ)
        return Ok(StatusCode::ACCEPTED);
    } else {
        object
    };

    let object_type = object
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Note");

    // 投票: name のみの Note (inReplyTo = Question/Note)
    if object_type == "Note" {
        if let Some(choice_name) = object.get("name").and_then(|v| v.as_str()) {
            if object.get("content").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
                return handle_poll_vote(state, remote_actor_uri, object, choice_name).await;
            }
        }
    }

    if object_type == "Question" {
        return handle_question_create(state, remote_actor_uri, object).await;
    }

    if object_type != "Note" {
        warn!("Unhandled Create object type: {object_type}");
        return Ok(StatusCode::ACCEPTED);
    }

    let note_uri = object
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Note missing id".to_string()))?;

    // 重複チェック
    if get_note_by_uri(state.surreal(), note_uri)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .is_some()
    {
        return Ok(StatusCode::ACCEPTED);
    }

    let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;

    // 本文 (HTML の場合はプレーン化は最小限: タグ除去はせず原文を保持)
    let content = object
        .get("source")
        .and_then(|s| s.get("content"))
        .and_then(|v| v.as_str())
        .or_else(|| object.get("content").and_then(|v| v.as_str()))
        .map(|s| s.to_string());

    let cw = object
        .get("summary")
        .and_then(|v| v.as_str())
        .map(String::from);

    // 引用 / リノート対象
    let quote_uri = extract_quote_uri(object);
    let reply_uri = object
        .get("inReplyTo")
        .and_then(|v| v.as_str())
        .map(String::from);

    let mut renote_id = None;
    let mut is_quote = false;
    if let Some(ref q) = quote_uri {
        if let Some(n) = resolve_note(state, q).await? {
            renote_id = Some(n.id);
            is_quote = content.as_ref().is_some_and(|t| !t.trim().is_empty());
        }
    }

    let mut reply_id = None;
    if let Some(ref r) = reply_uri {
        if let Some(n) = resolve_note(state, r).await? {
            reply_id = Some(n.id);
        }
    }

    // 永続化判定: フォロー関係 or ローカルへの reply/quote or メンション
    let mention_names: Vec<String> = object
        .get("tag")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|t| t.get("type").and_then(|x| x.as_str()) == Some("Mention"))
                .filter_map(|t| {
                    t.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.trim_start_matches('@').to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    let should = state
        .federation_service()
        .should_persist_note(remote_actor_uri, &mention_names)
        .await
        .unwrap_or(false)
        || reply_id.is_some()
        || renote_id.is_some();

    if !should {
        return Ok(StatusCode::ACCEPTED);
    }

    if let Some(tags) = object.get("tag") {
        cache_emoji_tags(state, tags, remote_actor.host.as_deref()).await;
    }

    let mut note = Note::new(remote_actor.id, content, NoteVisibility::Public);
    note.uri = Some(note_uri.to_string());
    note.cw = cw;
    note.reply_id = reply_id;
    note.renote_id = renote_id;
    note.actor_host = remote_actor.host.clone().or_else(|| host_from_uri(remote_actor_uri));
    if let Some(tags) = object.get("tag").and_then(|t| t.as_array()) {
        note.tags = tags
            .iter()
            .filter(|t| t.get("type").and_then(|x| x.as_str()) == Some("Hashtag"))
            .filter_map(|t| {
                t.get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.trim_start_matches('#').to_string())
            })
            .collect();
        note.emojis = tags
            .iter()
            .filter(|t| t.get("type").and_then(|x| x.as_str()) == Some("Emoji"))
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
            .collect();
    }

    let created = create_note(state.surreal(), &note)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // 通知
    if let Some(parent_id) = created.reply_id {
        if let Ok(Some(parent)) = get_note_by_id(state.surreal(), &parent_id).await {
            if parent.actor_id != remote_actor.id {
                let notif = Notification::new(
                    mithic_core::models::notification::NotificationType::Reply,
                    parent.actor_id,
                    Some(remote_actor.id),
                    Some(created.id),
                );
                publish_notification(state, &notif, Some(&remote_actor), None).await;
            }
        }
    }
    if is_quote {
        if let Some(qid) = created.renote_id {
            if let Ok(Some(target)) = get_note_by_id(state.surreal(), &qid).await {
                if target.actor_id != remote_actor.id {
                    let notif = Notification::new(
                        mithic_core::models::notification::NotificationType::Quote,
                        target.actor_id,
                        Some(remote_actor.id),
                        Some(created.id),
                    );
                    publish_notification(state, &notif, Some(&remote_actor), None).await;
                }
            }
        }
    }

    Ok(StatusCode::ACCEPTED)
}

async fn handle_announce(
    state: &AppState,
    remote_actor_uri: &str,
    activity: &Value,
) -> Result<StatusCode> {
    let note_uri = activity
        .get("object")
        .and_then(object_id)
        .ok_or_else(|| AppError::Validation("Announce missing object".to_string()))?;

    let Some(target) = resolve_note(state, note_uri).await? else {
        return Ok(StatusCode::ACCEPTED);
    };

    let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;
    let activity_id = activity
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from);

    // 重複: 同一 uri の renote があればスキップ
    if let Some(ref uri) = activity_id {
        if get_note_by_uri(state.surreal(), uri)
            .await
            .ok()
            .flatten()
            .is_some()
        {
            return Ok(StatusCode::ACCEPTED);
        }
    }

    let mut renote = Note::new(remote_actor.id, None, NoteVisibility::Public);
    renote.renote_id = Some(target.id);
    renote.uri = activity_id;
    renote.actor_host = remote_actor.host.clone();

    let created = create_note(state.surreal(), &renote)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let _ = state
        .surreal()
        .query("UPDATE note SET renote_count += 1 WHERE id = type::record('note', $id);")
        .bind(("id", target.id.to_string()))
        .await;

    if target.actor_id != remote_actor.id {
        let notif = Notification::new(
            mithic_core::models::notification::NotificationType::Renote,
            target.actor_id,
            Some(remote_actor.id),
            Some(created.id),
        );
        publish_notification(state, &notif, Some(&remote_actor), None).await;
    }

    Ok(StatusCode::ACCEPTED)
}

/// Question オブジェクトの受信 (投票付きノートとして最小保存)
async fn handle_question_create(
    state: &AppState,
    remote_actor_uri: &str,
    object: &Value,
) -> Result<StatusCode> {
    let note_uri = object
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Question missing id".to_string()))?;

    if get_note_by_uri(state.surreal(), note_uri)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .is_some()
    {
        return Ok(StatusCode::ACCEPTED);
    }

    let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;
    let content = object
        .get("content")
        .and_then(|v| v.as_str())
        .map(String::from);

    let mut note = Note::new(remote_actor.id, content, NoteVisibility::Public);
    note.uri = Some(note_uri.to_string());
    note.has_poll = true;
    note.actor_host = remote_actor.host.clone();

    // 選択肢を poll テーブルへ (oneOf / anyOf)
    let choices: Vec<String> = object
        .get("oneOf")
        .or_else(|| object.get("anyOf"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let created = create_note(state.surreal(), &note)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !choices.is_empty() {
        let choice_objs: Vec<serde_json::Value> = choices
            .into_iter()
            .map(|name| json!({ "text": name, "votes": 0 }))
            .collect();
        let poll_id = ulid::Ulid::new().to_string();
        let _ = state
            .surreal()
            .query(
                "
                INSERT INTO poll {
                    id: $id,
                    note_id: type::record('note', $note_id),
                    created_at: time::now(),
                    multiple: $multiple,
                    choices: $choices
                };
                ",
            )
            .bind(("id", poll_id))
            .bind(("note_id", created.id.to_string()))
            .bind(("multiple", object.get("anyOf").is_some()))
            .bind(("choices", choice_objs))
            .await;
    }

    Ok(StatusCode::ACCEPTED)
}

/// 投票: inReplyTo で Question/Note を指し name に選択肢
async fn handle_poll_vote(
    state: &AppState,
    remote_actor_uri: &str,
    object: &Value,
    choice_name: &str,
) -> Result<StatusCode> {
    let in_reply = object
        .get("inReplyTo")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Poll vote missing inReplyTo".to_string()))?;

    let Some(note) = resolve_note(state, in_reply).await? else {
        return Ok(StatusCode::ACCEPTED);
    };
    if !note.has_poll {
        return Ok(StatusCode::ACCEPTED);
    }

    let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;

    // poll の choices から index を解決
    let mut res = state
        .surreal()
        .query("SELECT * FROM poll WHERE note_id = type::record('note', $nid) LIMIT 1;")
        .bind(("nid", note.id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows: Vec<surrealdb::types::Value> = res
        .take(0)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let Some(poll_json) = rows.into_iter().next().map(|v| v.into_json_value()) else {
        return Ok(StatusCode::ACCEPTED);
    };
    let poll_id = poll_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_start_matches("poll:")
        .to_string();
    let choices = poll_json
        .get("choices")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let choice_index = choices.iter().position(|c| {
        c.get("text")
            .or_else(|| c.get("name"))
            .and_then(|n| n.as_str())
            == Some(choice_name)
    });
    let Some(idx) = choice_index else {
        return Ok(StatusCode::ACCEPTED);
    };

    if let Err(e) = mithic_db::queries::vote_poll(
        state.surreal(),
        &poll_id,
        &remote_actor.id.to_string(),
        idx,
    )
    .await
    {
        warn!("Remote poll vote failed: {e}");
    }

    Ok(StatusCode::ACCEPTED)
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

    // object が文字列 (Like activity URI 等) の場合は Like 取消として処理を試行
    if object.is_string() {
        return Ok(StatusCode::ACCEPTED);
    }

    let object_type = object
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    match object_type {
        "Follow" => {
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
        "Like" => {
            let note_uri = object
                .get("object")
                .and_then(object_id)
                .ok_or_else(|| AppError::Validation("Undo Like missing object".to_string()))?;
            let Some(note) = resolve_note(state, note_uri).await? else {
                return Ok(StatusCode::ACCEPTED);
            };
            let remote_actor = resolve_remote_actor(state, remote_actor_uri).await?;
            let reaction = extract_reaction(object);
            if let Err(e) = remove_all_reactions_by_actor(
                state.surreal(),
                &note.id.to_string(),
                &remote_actor.id.to_string(),
            )
            .await
            {
                warn!("Undo Like failed: {e}");
            } else {
                // extract_reaction はログ用; remove_all が本体
                let _ = reaction;
            }
            Ok(StatusCode::ACCEPTED)
        }
        other => {
            warn!("Unhandled Undo object type: {other}");
            Ok(StatusCode::ACCEPTED)
        }
    }
}
