pub mod v1;

use crate::state::AppState;
use axum::Router;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new().merge(v1::router(state))
}
