pub mod auth;
pub mod notes;
pub mod timeline;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/api/i", get(auth::me))
        .route("/api/notes/create", post(notes::create))
        .route("/api/notes/delete", post(notes::delete))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    let public = Router::new()
        .route("/api/signup", post(auth::signup))
        .route("/api/signin", post(auth::signin))
        .route("/api/notes/show", post(notes::show))
        .route("/api/notes/local-timeline", post(timeline::local))
        .route("/api/notes/global-timeline", post(timeline::global));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
}
