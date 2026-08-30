pub mod activitypub;
pub mod ogp;
/// mithic ネイティブ REST API (`/api/v1/*`)
pub mod v1;

use axum::{Router, routing::get};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::middleware::cors_layer;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = cors_layer(&state.config().cors_allowed_origins);

    Router::new()
        .merge(v1::router(state.clone()))
        .merge(activitypub::router(state.clone()))
        .route("/notes/{id}", get(ogp::note_ogp))
        .route("/profile/{username}", get(ogp::profile_ogp))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state)
}
