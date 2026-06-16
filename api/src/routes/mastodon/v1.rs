//! フロントエンド (`frontend-web/src/api/`) が呼び出す `/api/v1/*` ルート群。
//!
//! Misskey 風の既存ルート (`/api/notes/create` 等) と同じサービス層を共有する。

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    middleware::from_fn_with_state,
    routing::{delete, get, post},
};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use mithic_core::models::actor::ActorId;
use mithic_core::models::note::{Note, NoteId, NoteVisibility};
use mithic_core::models::notification::{Notification, NotificationType};
use mithic_core::services::auth::generate_jwt;
use mithic_core::{AppError, AuthUser, Result};
use mithic_db::queries::{
    NoteWithAuthor, add_reaction, create_note, delete_note, get_actor_by_id, get_actor_by_username,
    get_global_timeline, get_home_timeline, get_local_timeline, get_note_by_id, get_note_quotes,
    get_note_replies, get_notifications, get_user_notes, is_following,
    mark_all_notifications_as_read, mark_notification_as_read, remove_reaction, update_actor_token,
};
use shared::{
    CreateNoteRequest, LoginRequest, Note as NoteDto, Notification as NotifDto, RefreshRequest,
    SigninRequest, SignupRequest, TokenPair, User,
};

use crate::dto::{actor_to_user, note_to_dto};
use crate::events::StreamBroadcast;
use crate::middleware::auth_middleware;
use crate::routes::misskey::notifications::notif_type_to_dto;
use crate::services::note::{create_note_service, publish_notification};
use crate::services::relationship;
use crate::services::user::{authenticate_user, register_user};
use crate::state::AppState;

const REFRESH_EXPIRY_HOURS: i64 = 24 * 30;

pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/users/me", get(me).patch(update_me))
        .route("/api/v1/users/{username}", get(show_user))
        .route("/api/v1/users/{username}/notes", get(user_notes))
        .route(
            "/api/v1/users/{id}/follow",
            post(follow_user_route).delete(unfollow_user_route),
        )
        .route("/api/v1/timelines/{kind}", get(timeline))
        .route("/api/v1/notes", post(create_note_route))
        .route(
            "/api/v1/notes/{id}",
            get(show_note).delete(delete_note_route),
        )
        .route("/api/v1/notes/{id}/replies", get(note_replies))
        .route("/api/v1/notes/{id}/quotes", get(note_quotes))
        .route("/api/v1/notes/{id}/reactions", post(add_reaction_route))
        .route(
            "/api/v1/notes/{id}/reactions/{emoji}",
            delete(remove_reaction_route),
        )
        .route("/api/v1/notes/{id}/renotes", post(renote_route))
        .route("/api/v1/notifications", get(list_notifications))
        .route(
            "/api/v1/notifications/read-all",
            post(read_all_notifications),
        )
        .route("/api/v1/notifications/{id}/read", post(read_notification))
        .layer(from_fn_with_state(state, auth_middleware));

    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/users/check-handle", get(check_handle))
        .merge(protected)
}

/// フロントの `request::<(), _>` は JSON `null` を期待する
fn ok_null() -> Json<Value> {
    Json(Value::Null)
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

async fn health() -> Json<Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct RefreshClaims {
    sub: String,
    exp: usize,
    typ: String,
}

fn generate_refresh_token(user_id: &str, secret: &str) -> Result<String> {
    let exp = (Utc::now().timestamp() + REFRESH_EXPIRY_HOURS * 3600) as usize;
    let claims = RefreshClaims {
        sub: user_id.to_string(),
        exp,
        typ: "refresh".to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate refresh token: {e}")))
}

fn normalize_handle(handle: &str) -> String {
    handle.trim().trim_start_matches('@').to_string()
}

fn token_pair(state: &AppState, actor: &mithic_core::models::actor::Actor) -> Result<TokenPair> {
    let access = generate_jwt(
        &actor.id.to_string(),
        state.config().jwt_secret(),
        state.config().jwt_expiry_hours,
    )?;
    let refresh = generate_refresh_token(&actor.id.to_string(), state.config().jwt_secret())?;
    Ok(TokenPair {
        access_token: access,
        refresh_token: refresh,
        user: actor_to_user(actor),
    })
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<TokenPair>> {
    let signin = SigninRequest {
        username: normalize_handle(&request.handle),
        password: request.password,
    };
    let (_token, actor) = authenticate_user(state.surreal(), signin, state.config()).await?;
    Ok(Json(token_pair(&state, &actor)?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    handle: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    email: Option<String>,
    password: String,
}

async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<TokenPair>> {
    let username = normalize_handle(&request.handle);
    if username.len() < 3
        || !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::Validation("Invalid handle".to_string()));
    }
    if request.password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let signup = SignupRequest {
        username,
        password: request.password,
        name: request.display_name,
        email: request.email,
    };
    let actor = register_user(state.surreal(), signup, &state.config().instance_url).await?;
    Ok(Json(token_pair(&state, &actor)?))
}

async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<TokenPair>> {
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

    #[derive(Debug, Deserialize)]
    struct Claims {
        sub: String,
        #[allow(dead_code)]
        exp: i64,
        #[serde(default)]
        typ: String,
    }

    let data = decode::<Claims>(
        &request.refresh_token,
        &DecodingKey::from_secret(state.config().jwt_secret().as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    if data.claims.typ != "refresh" {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    let user_id = data
        .claims
        .sub
        .parse::<ActorId>()
        .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

    let actor = get_actor_by_id(state.surreal(), &user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    Ok(Json(token_pair(&state, &actor)?))
}

async fn logout(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Value>> {
    update_actor_token(state.surreal(), &auth.user_id, None)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<User>> {
    let actor = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(actor_to_user(&actor)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMeRequest {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    bio: Option<String>,
}

async fn update_me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<UpdateMeRequest>,
) -> Result<Json<User>> {
    state
        .surreal()
        .query(
            "UPDATE user SET
                name = IF $name != None THEN $name ELSE name END,
                bio = IF $bio != None THEN $bio ELSE bio END,
                updated_at = time::now()
             WHERE id = type::record('user', $id);",
        )
        .bind(("id", auth.user_id.to_string()))
        .bind(("name", request.display_name))
        .bind(("bio", request.bio))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    me(State(state), Extension(auth)).await
}

#[derive(Debug, Deserialize)]
struct CheckHandleQuery {
    handle: String,
}

#[derive(Debug, Serialize)]
struct HandleAvailability {
    available: bool,
}

async fn check_handle(
    State(state): State<AppState>,
    Query(query): Query<CheckHandleQuery>,
) -> Result<Json<HandleAvailability>> {
    let username = normalize_handle(&query.handle);
    let actor = get_actor_by_username(state.surreal(), &username)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(HandleAvailability {
        available: actor.is_none() && username.len() >= 3,
    }))
}

async fn show_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>> {
    let actor = get_actor_by_username(state.surreal(), &normalize_handle(&username))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    Ok(Json(actor_to_user(&actor)))
}

async fn user_notes(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(paging): Query<PagingQuery>,
) -> Result<Json<Vec<NoteDto>>> {
    let actor = get_actor_by_username(state.surreal(), &normalize_handle(&username))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let notes = get_user_notes(
        state.surreal(),
        &actor.id,
        paging.limit.unwrap_or(20),
        parse_optional_note_id(&paging.until_id)?,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(rows_to_dtos(notes)))
}

fn parse_actor_id(raw: &str) -> Result<ActorId> {
    raw.parse::<ActorId>()
        .map_err(|_| AppError::Validation("Invalid user id".to_string()))
}

async fn follow_user_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let target_id = parse_actor_id(&id)?;
    if target_id == auth.user_id {
        return Err(AppError::Validation("Cannot follow yourself".to_string()));
    }

    let already = is_following(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if !already {
        relationship::follow(state.surreal(), &auth.user_id, &target_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // フォロー通知
        let sender = get_actor_by_id(state.surreal(), &auth.user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let notif = Notification::follow(target_id, auth.user_id);
        publish_notification(&state, &notif, sender.as_ref(), None).await;
    }
    Ok(ok_null())
}

async fn unfollow_user_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let target_id = parse_actor_id(&id)?;
    relationship::unfollow(state.surreal(), &auth.user_id, &target_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

// ---------------------------------------------------------------------------
// Timelines
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PagingQuery {
    limit: Option<usize>,
    since_id: Option<String>,
    until_id: Option<String>,
}

fn parse_optional_note_id(raw: &Option<String>) -> Result<Option<NoteId>> {
    match raw {
        Some(value) => value
            .parse::<NoteId>()
            .map(Some)
            .map_err(|_| AppError::Validation("Invalid note id".to_string())),
        None => Ok(None),
    }
}

fn rows_to_dtos(rows: Vec<NoteWithAuthor>) -> Vec<NoteDto> {
    rows.into_iter()
        .map(|row| note_to_dto(&row.note, actor_to_user(&row.author)))
        .collect()
}

async fn timeline(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(kind): Path<String>,
    Query(paging): Query<PagingQuery>,
) -> Result<Json<Vec<NoteDto>>> {
    let limit = paging.limit.unwrap_or(20).min(100);
    let since_id = parse_optional_note_id(&paging.since_id)?;
    let until_id = parse_optional_note_id(&paging.until_id)?;

    let rows = match kind.as_str() {
        "home" => {
            get_home_timeline(state.surreal(), &auth.user_id, limit, since_id, until_id).await
        }
        "local" => get_local_timeline(state.surreal(), limit, since_id, until_id).await,
        "global" => get_global_timeline(state.surreal(), limit, since_id, until_id).await,
        _ => return Err(AppError::Validation(format!("Unknown timeline: {kind}"))),
    }
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(rows_to_dtos(rows)))
}

// ---------------------------------------------------------------------------
// Notes
// ---------------------------------------------------------------------------

fn parse_note_id(raw: &str) -> Result<NoteId> {
    raw.parse::<NoteId>()
        .map_err(|_| AppError::Validation("Invalid note id".to_string()))
}

async fn create_note_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<NoteDto>> {
    let dto = create_note_service(&state, auth.user_id, request).await?;
    Ok(Json(dto))
}

async fn fetch_note_dto(state: &AppState, note_id: &NoteId) -> Result<(Note, NoteDto)> {
    let note = get_note_by_id(state.surreal(), note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;
    let author = get_actor_by_id(state.surreal(), &note.actor_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;
    let dto = note_to_dto(&note, actor_to_user(&author));
    Ok((note, dto))
}

async fn show_note(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<NoteDto>> {
    let note_id = parse_note_id(&id)?;
    let (_, dto) = fetch_note_dto(&state, &note_id).await?;
    Ok(Json(dto))
}

async fn delete_note_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let (note, _) = fetch_note_dto(&state, &note_id).await?;
    if note.actor_id != auth.user_id {
        return Err(AppError::Forbidden(
            "You can only delete your own notes".to_string(),
        ));
    }
    delete_note(state.surreal(), &note_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = state
        .surreal()
        .query("UPDATE user SET notes_count = <int>(notes_count OR 1) - 1 WHERE id = type::record('user', $id);")
        .bind(("id", auth.user_id.to_string()))
        .await;
    Ok(ok_null())
}

async fn note_replies(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<NoteDto>>> {
    let note_id = parse_note_id(&id)?;
    let rows = get_note_replies(state.surreal(), &note_id, 100)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(rows_to_dtos(rows)))
}

async fn note_quotes(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<NoteDto>>> {
    let note_id = parse_note_id(&id)?;
    let rows = get_note_quotes(state.surreal(), &note_id, 100)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(rows_to_dtos(rows)))
}

#[derive(Debug, Deserialize)]
struct ReactionBody {
    emoji: String,
}

async fn add_reaction_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(body): Json<ReactionBody>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    let (note, dto) = fetch_note_dto(&state, &note_id).await?;

    add_reaction(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
        &body.emoji,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // リアクション通知
    if note.actor_id != auth.user_id {
        let sender = get_actor_by_id(state.surreal(), &auth.user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let notif =
            Notification::reaction(note.actor_id, auth.user_id, note_id, body.emoji.clone());
        publish_notification(&state, &notif, sender.as_ref(), Some(dto)).await;
    }
    Ok(ok_null())
}

async fn remove_reaction_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((id, emoji)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let note_id = parse_note_id(&id)?;
    remove_reaction(
        state.surreal(),
        &note_id.to_string(),
        &auth.user_id.to_string(),
        &emoji,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

async fn renote_route(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<NoteDto>> {
    let target_note_id = parse_note_id(&id)?;
    let (target_note, target_dto) = fetch_note_dto(&state, &target_note_id).await?;

    let mut renote = Note::new(auth.user_id, None, NoteVisibility::Public);
    renote.renote_id = Some(target_note_id);

    let created = create_note(state.surreal(), &renote)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    state
        .surreal()
        .query("UPDATE note SET renote_count += 1 WHERE id = type::record('note', $id);")
        .bind(("id", target_note_id.to_string()))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let author = get_actor_by_id(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Author not found".to_string()))?;

    // リノート通知
    if target_note.actor_id != auth.user_id {
        let notif = Notification::new(
            NotificationType::Renote,
            target_note.actor_id,
            Some(auth.user_id),
            Some(created.id),
        );
        publish_notification(&state, &notif, Some(&author), Some(target_dto)).await;
    }

    let dto = note_to_dto(&created, actor_to_user(&author));
    state.publish_stream(StreamBroadcast::Note(Box::new(dto.clone())));
    Ok(Json(dto))
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

async fn list_notifications(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(paging): Query<PagingQuery>,
) -> Result<Json<Vec<NotifDto>>> {
    let limit = paging.limit.unwrap_or(20).min(100);
    let notifs = get_notifications(state.surreal(), &auth.user_id, limit)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut dtos = Vec::with_capacity(notifs.len());
    for notif in notifs {
        let sender = match notif.sender_id {
            Some(sender_id) => get_actor_by_id(state.surreal(), &sender_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?
                .map(|actor| actor_to_user(&actor)),
            None => None,
        };

        let note = match notif.note_id {
            Some(note_id) => fetch_note_dto(&state, &note_id)
                .await
                .ok()
                .map(|(_, dto)| dto),
            None => None,
        };

        dtos.push(NotifDto {
            id: notif.id.to_string(),
            created_at: notif.created_at.to_rfc3339(),
            notification_type: notif_type_to_dto(notif.notification_type),
            sender,
            note,
            reaction: notif.reaction,
            is_read: notif.is_read,
        });
    }

    Ok(Json(dtos))
}

async fn read_all_notifications(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Value>> {
    mark_all_notifications_as_read(state.surreal(), &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}

async fn read_notification(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    mark_notification_as_read(state.surreal(), &id, &auth.user_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ok_null())
}
