use std::sync::Arc;

use axum::{Router, routing::get};

use crate::app::AppState;
use crate::gateway::auth;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/v1", auth::router())
        .nest("/api/v1/profile", crate::profile::router())
}
