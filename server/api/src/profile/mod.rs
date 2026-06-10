use axum::{
    Router,
    routing::{get, delete},
};
use std::sync::Arc;
use crate::app::AppState;

pub mod models;
pub mod error;
pub mod repository;
pub mod service;
pub mod types;
pub mod handlers;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::get_profile).put(handlers::update_profile))
        .route("/safety", get(handlers::get_medical_safety).put(handlers::update_medical_safety))
        .route("/preferences", get(handlers::get_preferences).put(handlers::update_preferences))
        .route("/biometrics", get(handlers::get_biometric_history).post(handlers::log_biometric))
        .route("/workouts", get(handlers::get_recent_workouts).post(handlers::log_workout))
        .route("/goals", get(handlers::get_active_goals).post(handlers::create_goal))
        .route("/goals/{category}", delete(handlers::deactivate_goal))
}
