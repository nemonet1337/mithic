pub mod activitypub;
pub mod auth;
pub mod drive;
pub mod notes;
pub mod notifications;
pub mod streaming;
pub mod timeline;
pub mod users;
pub mod v1;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::middleware::{auth_middleware, cors_layer};
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/i", get(auth::me))
        .route("/api/signout", post(auth::signout))
        .route("/api/notes/create", post(notes::create))
        .route("/api/notes/delete", post(notes::delete))
        .route("/api/notes/reactions/create", post(notes::create_reaction))
        .route("/api/notes/reactions/delete", post(notes::delete_reaction))
        .route("/api/notes/favorites/create", post(notes::create_favorite))
        .route("/api/notes/favorites/delete", post(notes::delete_favorite))
        .route("/api/notes/renote", post(notes::renote))
        .route("/api/notes/unrenote", post(notes::unrenote))
        .route("/api/notes/timeline", post(notes::home_timeline))
        .route("/api/users/show", post(users::show))
        .route("/api/users/relation", post(users::relation))
        .route("/api/users/following", post(users::following))
        .route("/api/users/followers", post(users::followers))
        .route("/api/users/notes", post(users::user_notes))
        .route("/api/users/search", post(users::search))
        .route("/api/username/available", post(users::available))
        .route("/api/following/create", post(users::follow_route))
        .route("/api/following/delete", post(users::unfollow_route))
        .route("/api/blocking/create", post(users::block_route))
        .route("/api/blocking/delete", post(users::unblock_route))
        .route("/api/muting/create", post(users::mute_route))
        .route("/api/muting/delete", post(users::unmute_route))
        .route("/api/notifications/list", post(notifications::list))
        .route("/api/notifications/read", post(notifications::read))
        .route(
            "/api/notifications/mark-all-as-read",
            post(notifications::mark_all_as_read_route),
        )
        .route("/api/drive/files/create", post(drive::upload_file))
        .route("/api/drive/files/show", post(drive::show))
        .route("/api/drive/files/delete", post(drive::delete))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    let public = Router::new()
        .route("/api/signup", post(auth::signup))
        .route("/api/signin", post(auth::signin))
        .route("/api/notes/show", post(notes::show))
        .route("/api/notes/local-timeline", post(timeline::local))
        .route("/api/notes/global-timeline", post(timeline::global))
        .route("/api/streaming", get(streaming::streaming_handler));

    let cors = cors_layer(&state.config().cors_allowed_origins);

    Router::new()
        .merge(public)
        .merge(protected)
        .merge(v1::router(state.clone()))
        .merge(activitypub::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state)
}
