use axum::{
    Router,
    routing::{get, patch},
};
use std::sync::Arc;
use crate::app::AppState;

pub mod models;
pub mod error;
pub mod repository;
pub mod service;
pub mod types;
pub mod handlers;

pub fn meal_plans_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::list_meal_plans).post(handlers::create_meal_plan))
        .route("/active", get(handlers::get_active_meal_plan))
        .route("/{id}", get(handlers::get_meal_plan))
        .route("/{id}/status", patch(handlers::update_meal_plan_status))
        .route("/{id}/items/{item_id}/consumed", patch(handlers::update_meal_plan_item_consumed))
}

pub fn pantry_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::list_pantry_items).post(handlers::create_pantry_item))
        .route("/{id}", patch(handlers::update_pantry_item).delete(handlers::delete_pantry_item))
}

pub fn shopping_lists_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::list_shopping_lists).post(handlers::create_shopping_list))
        .route("/{id}", get(handlers::get_shopping_list))
        .route("/{id}/items/{item_id}", patch(handlers::update_shopping_list_item_acquired))
}
