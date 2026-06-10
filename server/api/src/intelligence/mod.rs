use axum::{
    Router,
    routing::{get, delete, post},
};
use std::sync::Arc;
use crate::app::AppState;

pub mod models;
pub mod error;
pub mod repository;
pub mod service;
pub mod types;
pub mod handlers;
pub mod stream;
pub mod delta;
pub mod context_builder;
pub mod orchestrator;
pub mod worker;

pub fn intelligence_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/traces", get(handlers::list_traces))
        .route("/traces/{id}", get(handlers::get_trace))
        .route("/memories", get(handlers::list_active_memories))
        .route("/memories/{id}", delete(handlers::deactivate_memory))
        .route("/memories/search", post(handlers::search_memories))
        .route("/orchestrate", post(handlers::orchestrate))
        .route("/intent", post(handlers::process_intent))
        .route("/stream/{trace_id}", get(stream::get_stream))
}
