use std::sync::Arc;

use axum::{Router, routing::get};

use crate::app::AppState;
use crate::gateway::auth;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/v1", auth::router())
        .nest("/api/v1/profile", crate::profile::router())
        .nest("/api/v1/food-items", crate::recipe::food_items_router())
        .nest("/api/v1/recipes", crate::recipe::recipes_router())
        .nest("/api/v1/food-logs", crate::recipe::food_logs_router())
        .nest("/api/v1/meal-plans", crate::plan::meal_plans_router())
        .nest("/api/v1/pantry", crate::plan::pantry_router())
        .nest("/api/v1/shopping-lists", crate::plan::shopping_lists_router())
        .nest("/api/v1/threads", crate::chat::threads_router())
        .nest("/api/v1/feedback", crate::chat::feedback_router())
        .nest("/api/v1/intelligence", crate::intelligence::intelligence_router())
        .nest("/api/v1/admin", crate::admin::router())
}
