use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{app::AppState, auth::handlers};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/auth", auth_router())
        .merge(authorization_router())
}

fn auth_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signup", post(handlers::signup))
        .route("/signin", post(handlers::signin))
        .route("/signout", post(handlers::signout))
        .route("/signout-all", post(handlers::signout_all))
        .route("/send-verification-otp", post(handlers::send_verification_otp))
        .route("/verify-email", post(handlers::verify_email))
        .route("/forgot-password", post(handlers::forgot_password))
        .route("/reset-password", post(handlers::reset_password))
        .route("/change-password", post(handlers::change_password))
        .route("/sessions", get(handlers::list_sessions))
        .route("/sessions/{id}", delete(handlers::revoke_session))
        .route("/oauth/{provider}/authorize", get(crate::auth::oauth::authorize))
        .route("/oauth/{provider}/callback", get(crate::auth::oauth::callback))
}

fn authorization_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/roles", get(handlers::list_roles).post(handlers::create_role))
        .route(
            "/permissions",
            get(handlers::list_permissions).post(handlers::create_permission),
        )
        .route(
            "/roles/{role_id}/permissions/{permission_id}",
            post(handlers::assign_permission_to_role).delete(handlers::remove_permission_from_role),
        )
        .route(
            "/users/{user_id}/roles/{role_id}",
            post(handlers::assign_role_to_user).delete(handlers::remove_role_from_user),
        )
}
