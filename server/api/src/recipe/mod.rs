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

pub fn food_items_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(handlers::create_food_item))
        .route("/search", get(handlers::search_food_items))
        .route("/{id}", get(handlers::get_food_item))
}

pub fn recipes_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::list_recipes).post(handlers::create_recipe))
        .route("/{id}", get(handlers::get_recipe))
}

pub fn food_logs_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::get_food_logs).post(handlers::log_food))
        .route("/daily", get(handlers::get_daily_macros))
}
