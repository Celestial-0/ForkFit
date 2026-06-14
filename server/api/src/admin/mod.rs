use axum::{
    Router,
    routing::{get, post, put},
};
use std::sync::Arc;
use crate::app::AppState;

pub mod handlers;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/jobs", get(handlers::list_jobs))
        .route("/audit-logs", get(handlers::list_audit_logs))
        .route("/raw-food-costs", get(handlers::list_raw_food_costs).post(handlers::create_raw_food_cost))
        .route("/raw-food-costs/{id}", put(handlers::update_raw_food_cost).delete(handlers::delete_raw_food_cost))
        .route("/raw-food-costs/link", post(handlers::link_food_item_to_cost))
}
