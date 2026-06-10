use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::user::UserResponse;

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmailRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub email: String,
    pub otp: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub otp: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_at: DateTime<Utc>,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub device_name: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct CredentialRow {
    pub user_id: Uuid,
    pub password_hash: String,
}

#[derive(Debug, FromRow)]
pub struct OtpRow {
    pub id: Uuid,
    pub otp_hash: String,
    pub attempts: i32,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub resource: String,
    pub action: String,
}
