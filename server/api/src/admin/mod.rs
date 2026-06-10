use axum::{
    Router,
    routing::get,
};
use std::sync::Arc;
use crate::app::AppState;

pub mod handlers;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/jobs", get(handlers::list_jobs))
        .route("/audit-logs", get(handlers::list_audit_logs))
}
