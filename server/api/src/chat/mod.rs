use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use crate::app::AppState;

pub mod models;
pub mod error;
pub mod repository;
pub mod service;
pub mod types;
pub mod handlers;

pub fn threads_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::list_threads).post(handlers::create_thread))
        .route("/{id}", get(handlers::get_thread).delete(handlers::delete_thread))
        .route("/{id}/messages", get(handlers::list_messages).post(handlers::post_message))
}

pub fn feedback_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(handlers::create_feedback))
}
