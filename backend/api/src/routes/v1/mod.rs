//! mithic ネイティブ REST API (`/api/v1/*`)
//!
//! WebUI / PWA 専用。Misskey / Mastodon クライアント互換は持たない。

pub mod admin;
pub mod auth;
pub mod common;
pub mod drive;
pub mod instance;
pub mod notes;
pub mod notifications;
pub mod push;
pub mod streaming;
pub mod timelines;
pub mod users;

use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    middleware::from_fn_with_state,
    routing::{delete, get, patch, post},
};
use serde_json::Value;

use crate::middleware::{auth_middleware, optional_auth_middleware, rate_limit_middleware};
use crate::state::AppState;

const MAX_UPLOAD_BYTES: usize = 32 * 1024 * 1024;

async fn health() -> Json<Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub fn router(state: AppState) -> Router<AppState> {
    // --- 認証必須 ---
    let protected = Router::new()
        // auth
        .route("/api/v1/auth/logout", post(auth::logout))
        // users me
        .route("/api/v1/users/me", get(users::me).patch(users::update_me))
        .route("/api/v1/users/me/password", post(users::change_password))
        // follow / block / mute
        .route(
            "/api/v1/users/{id}/follow",
            post(users::follow_user).delete(users::unfollow_user),
        )
        .route(
            "/api/v1/users/{id}/block",
            post(users::block_user).delete(users::unblock_user),
        )
        .route(
            "/api/v1/users/{id}/mute",
            post(users::mute_user).delete(users::unmute_user),
        )
        .route("/api/v1/users/{id}/relation", get(users::relation))
        // follow requests
        .route("/api/v1/follow-requests", get(users::list_follow_requests))
        .route(
            "/api/v1/follow-requests/{id}/accept",
            post(users::accept_follow_request),
        )
        .route(
            "/api/v1/follow-requests/{id}/reject",
            post(users::reject_follow_request),
        )
        .route(
            "/api/v1/follow-requests/{id}",
            delete(users::cancel_follow_request),
        )
        .route("/api/v1/blocks", get(users::list_blocks))
        .route("/api/v1/mutes", get(users::list_mutes))
        // home timeline (auth)
        .route("/api/v1/timelines/home", get(timelines::timeline_home))
        // notes write
        .route("/api/v1/notes", post(notes::create_note_route))
        .route("/api/v1/notes/{id}", delete(notes::delete_note_route))
        .route(
            "/api/v1/notes/{id}/reactions",
            post(notes::add_reaction_route),
        )
        .route(
            "/api/v1/notes/{id}/reactions/{emoji}",
            delete(notes::remove_reaction_route),
        )
        .route("/api/v1/notes/{id}/renotes", post(notes::renote_route))
        .route("/api/v1/notes/{id}/renote", delete(notes::unrenote_route))
        .route(
            "/api/v1/notes/{id}/favorite",
            post(notes::favorite_route).delete(notes::unfavorite_route),
        )
        .route(
            "/api/v1/notes/{id}/pin",
            post(notes::pin_route).delete(notes::unpin_route),
        )
        .route("/api/v1/notes/{id}/vote", post(notes::vote_route))
        // notifications
        .route(
            "/api/v1/notifications",
            get(notifications::list_notifications),
        )
        .route(
            "/api/v1/notifications/read-all",
            post(notifications::read_all_notifications),
        )
        .route(
            "/api/v1/notifications/{id}/read",
            post(notifications::read_notification),
        )
        // drive
        .route(
            "/api/v1/drive/files",
            post(drive::upload_file).get(drive::find),
        )
        .route("/api/v1/drive/files/from-url", post(drive::upload_from_url))
        .route(
            "/api/v1/drive/files/{id}",
            get(drive::show).delete(drive::delete),
        )
        .route("/api/v1/drive/files/{id}/notes", get(drive::attached_notes))
        // push
        .route(
            "/api/v1/push/subscription",
            post(push::subscribe)
                .get(push::get_subscription)
                .delete(push::unsubscribe),
        )
        // admin
        .route("/api/v1/admin/accounts/{id}/suspend", post(admin::suspend))
        .route(
            "/api/v1/admin/accounts/{id}/unsuspend",
            post(admin::unsuspend),
        )
        .route("/api/v1/admin/accounts/{id}", delete(admin::delete_account))
        .route(
            "/api/v1/admin/relays",
            get(admin::list_relays_route).post(admin::add_relay),
        )
        .route(
            "/api/v1/admin/relays/{id}",
            patch(admin::update_relay).delete(admin::remove_relay),
        )
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    // --- 認証 + レート制限 (login/register) ---
    let auth_public = Router::new()
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/refresh", post(auth::refresh))
        .route("/api/v1/auth/register", post(auth::register))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware));

    // --- 公開 ---
    let public = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/instance", get(instance::get_instance))
        .route("/api/v1/users/check-handle", get(users::check_handle))
        .route("/api/v1/users/search", get(users::search_users))
        .route("/api/v1/users/suggested", get(users::suggested_users))
        .route("/api/v1/users/{username}", get(users::show_user))
        .route("/api/v1/users/{username}/notes", get(users::user_notes))
        .route("/api/v1/users/{id}/following", get(users::following))
        .route("/api/v1/users/{id}/followers", get(users::followers))
        .route("/api/v1/timelines/local", get(timelines::timeline_local))
        .route("/api/v1/timelines/global", get(timelines::timeline_global))
        .route(
            "/api/v1/timelines/hashtag/{tag}",
            get(timelines::timeline_hashtag),
        )
        .route(
            "/api/v1/hashtags/trending",
            get(timelines::trending_hashtags),
        )
        .route("/api/v1/notes/search", get(notes::search_notes))
        .route("/api/v1/notes/{id}", get(notes::show_note))
        .route("/api/v1/notes/{id}/replies", get(notes::note_replies))
        .route("/api/v1/notes/{id}/quotes", get(notes::note_quotes))
        .route("/api/v1/streaming", get(streaming::streaming_handler))
        .route("/uploads/{hash}", get(drive::serve_upload))
        .layer(from_fn_with_state(state.clone(), optional_auth_middleware));

    Router::new()
        .merge(auth_public)
        .merge(public)
        .merge(protected)
}
