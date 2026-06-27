pub mod admin;
pub mod auth;
pub mod drive;
pub mod hashtags;
pub mod i;
pub mod notes;
pub mod notifications;
pub mod oauth;
pub mod push;
pub mod relays;
pub mod streaming;
pub mod timeline;
pub mod users;

use crate::middleware::auth_middleware;
use crate::state::AppState;
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/api/i", get(auth::me))
        .route("/api/signout", post(auth::signout))
        .route("/api/i/update", post(i::update_profile))
        .route("/api/i/change-password", post(i::change_password))
        .route("/api/i/regenerate-token", post(i::regenerate_token))
        .route("/api/i/update-email", post(i::update_email))
        .route("/api/notes/create", post(notes::create))
        .route("/api/notes/delete", post(notes::delete))
        .route("/api/notes/reactions/create", post(notes::create_reaction))
        .route("/api/notes/reactions/delete", post(notes::delete_reaction))
        .route("/api/notes/favorites/create", post(notes::create_favorite))
        .route("/api/notes/favorites/delete", post(notes::delete_favorite))
        .route("/api/notes/renote", post(notes::renote))
        .route("/api/notes/unrenote", post(notes::unrenote))
        .route("/api/notes/timeline", post(notes::home_timeline))
        .route("/api/notes/search", post(notes::search_notes))
        .route("/api/notes/pin", post(notes::pin_note))
        .route("/api/notes/unpin", post(notes::unpin_note))
        .route("/api/notes/polls/vote", post(notes::vote_poll))
        .route("/api/users/show", post(users::show))
        .route("/api/users/relation", post(users::relation))
        .route("/api/users/following", post(users::following))
        .route("/api/users/followers", post(users::followers))
        .route("/api/users/notes", post(users::user_notes))
        .route("/api/users/search", post(users::search))
        .route("/api/username/available", post(users::available))
        .route("/api/following/create", post(users::follow_route))
        .route("/api/following/delete", post(users::unfollow_route))
        .route(
            "/api/following/requests/list",
            post(users::list_follow_requests),
        )
        .route(
            "/api/following/requests/accept",
            post(users::accept_follow_request),
        )
        .route(
            "/api/following/requests/reject",
            post(users::reject_follow_request),
        )
        .route(
            "/api/following/requests/cancel",
            post(users::cancel_follow_request),
        )
        .route("/api/blocking/create", post(users::block_route))
        .route("/api/blocking/delete", post(users::unblock_route))
        .route("/api/blocking/list", post(users::list_blocking))
        .route("/api/muting/create", post(users::mute_route))
        .route("/api/muting/delete", post(users::unmute_route))
        .route("/api/muting/list", post(users::list_muting))
        .route("/api/admin/relays/add", post(relays::add_relay))
        .route("/api/admin/relays/list", post(relays::list_relays_route))
        .route("/api/admin/relays/remove", post(relays::remove_relay))
        .route("/api/admin/relays/update", post(relays::update_relay))
        .route("/api/notifications/list", post(notifications::list))
        .route("/api/notifications/read", post(notifications::read))
        .route(
            "/api/notifications/mark-all-as-read",
            post(notifications::mark_all_as_read_route),
        )
        .route("/api/drive/files/create", post(drive::upload_file))
        .route("/api/drive/files/show", post(drive::show))
        .route("/api/drive/files/delete", post(drive::delete))
        .route("/api/drive/files/find", post(drive::find))
        .route("/api/drive/files/upload-from-url", post(drive::upload_from_url))
        .route("/api/drive/files/attached-notes", post(drive::attached_notes))
        .route("/api/admin/accounts/suspend", post(admin::suspend))
        .route("/api/admin/accounts/unsuspend", post(admin::unsuspend))
        .route("/api/admin/accounts/delete", post(admin::delete_account))
        .route("/api/push/subscription", post(push::subscribe))
        .route("/api/push/subscription", get(push::get_subscription))
        .route("/api/push/subscription", axum::routing::delete(push::unsubscribe))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    let public = Router::new()
        .route("/api/signup", post(auth::signup))
        .route("/api/signin", post(auth::signin))
        .route("/api/notes/show", post(notes::show))
        .route("/api/notes/local-timeline", post(timeline::local))
        .route("/api/notes/global-timeline", post(timeline::global))
        .route("/api/notes/search", post(notes::search_notes))
        .route("/api/hashtags/timeline", post(hashtags::hashtag_timeline))
        .route("/api/hashtags/trending", post(hashtags::trending))
        .route("/api/streaming", get(streaming::streaming_handler))
        .route("/uploads/{hash}", get(drive::serve_upload))
        .route("/api/apps", post(oauth::create_app))
        .route("/oauth/authorize", post(oauth::authorize))
        .route("/oauth/token", post(oauth::token))
        .route("/oauth/revoke", post(oauth::revoke));

    Router::new().merge(public).merge(protected)
}
