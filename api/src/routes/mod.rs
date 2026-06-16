pub mod activitypub;
pub mod mastodon;
pub mod misskey;
pub mod ogp;

use axum::{Router, routing::get};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::middleware::cors_layer;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = cors_layer(&state.config().cors_allowed_origins);

    Router::new()
        .merge(misskey::router(state.clone()))
        .merge(mastodon::router(state.clone()))
        .merge(activitypub::router())
        .route("/notes/{id}", get(ogp::note_ogp))
        .route("/profile/{username}", get(ogp::profile_ogp))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state)
}
