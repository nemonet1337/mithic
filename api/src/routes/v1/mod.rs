mod admin;
mod auth;
mod conversations;
mod files;
mod notes;
mod notifications;
mod polls;
mod relationships;
mod search;
mod streaming;
mod timelines;
mod users;

use axum::Router;
use tower_http::compression::CompressionLayer;
use crate::state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::<AppState>::new()
        .nest("/auth", auth::router(state.clone()))
        .nest("/users", users::router(state.clone()))
        .nest("/notes", notes::router(state.clone()))
        .nest("/timelines", timelines::router(state.clone()))
        .nest("/files", files::router(state.clone()))
        .nest("/follows", relationships::router(state.clone()))
        .nest("/blocks", relationships::router(state.clone()))
        .nest("/mutes", relationships::router(state.clone()))
        .nest("/notifications", notifications::router(state.clone()))
        .nest("/admin", admin::router(state.clone()))
        .nest("/conversations", conversations::router(state.clone()))
        .nest("/search", search::router(state.clone()))
        .nest("/polls", polls::router(state.clone()))
        .nest("/streaming", streaming::router(state.clone()))
        .layer(CompressionLayer::new())
}
