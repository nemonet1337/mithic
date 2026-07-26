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
        .route("/metrics", get(metrics_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state)
}

async fn metrics_handler() -> axum::response::Response {
    // Simple text response for now - metrics recording happens elsewhere
    let body = "ok\n";
    axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(axum::body::Body::from(body))
        .unwrap()
}
