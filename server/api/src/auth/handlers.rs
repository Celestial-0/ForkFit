use std::{net::IpAddr, sync::Arc};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use chrono::{Duration as ChronoDuration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    app::AppState,
    common::{
        AppError, AppResult, generate_otp, generate_session_token, hash_secret, normalize_email,
        password_hash, validate_password, verify_password, verify_secret,
    },
    middleware::{CurrentUser, require_permission},
    user::{Session, User},
};
use crate::auth::{
    types::*,
    utils::{clean_key, device_name_from, user_agent},
};

// --- Constants ---
pub const PURPOSE_EMAIL_VERIFICATION: &str = "email_verification";
pub const PURPOSE_PASSWORD_RESET: &str = "password_reset";

pub const DEFAULT_PERMISSIONS: &[(&str, &str)] = &[
    ("roles", "read"),
    ("roles", "create"),
    ("role_permissions", "manage"),
    ("permissions", "read"),
    ("permissions", "create"),
    ("user_roles", "manage"),
];

// --- Seeding & DB Helper Functions ---

pub async fn seed_defaults(db: &PgPool, admin_email: Option<&str>) -> AppResult<()> {
    let admin_role_id = upsert_role(db, "admin", Some("Full platform administrator")).await?;
    upsert_role(db, "user", Some("Default application user")).await?;

    for (resource, action) in DEFAULT_PERMISSIONS {
        let permission_id = upsert_permission(db, resource, action).await?;
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(admin_role_id)
        .bind(permission_id)
        .execute(db)
        .await?;
    }

    if let Some(email) = admin_email {
        assign_admin_if_user_exists(db, email).await?;
    }

    Ok(())
}

pub async fn assign_admin_if_user_exists(db: &PgPool, email: &str) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO user_roles (user_id, role_id)
        SELECT users.id, roles.id
        FROM users
        CROSS JOIN roles
        WHERE users.email = $1
          AND roles.name = 'admin'
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(email)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn find_active_user_by_email(state: &AppState, email: &str) -> AppResult<User> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email::text AS email, email_verified, status, created_at, updated_at, deleted_at
        FROM users
        WHERE email = $1
          AND deleted_at IS NULL
          AND status = 'active'
        "#,
    )
    .bind(email)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)
}

pub async fn create_session_response(
    state: &AppState,
    user: User,
    device_name: Option<String>,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
) -> AppResult<AuthResponse> {
    let token = generate_session_token();
    let token_hash = hash_secret(&token);
    let expires_at = Utc::now()
        + ChronoDuration::from_std(state.config.session_ttl)
            .unwrap_or_else(|_| ChronoDuration::days(30));

    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, token_hash, ip_address, user_agent, device_name, expires_at, last_seen_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        "#,
    )
    .bind(user.id)
    .bind(token_hash)
    .bind(ip_address)
    .bind(user_agent)
    .bind(device_name)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(AuthResponse {
        access_token: token,
        token_type: "Bearer",
        expires_at,
        user: user.into(),
    })
}

pub async fn create_and_send_otp(state: &AppState, email: &str, purpose: &str) -> AppResult<()> {
    let otp = generate_otp();
    let expires_at = Utc::now()
        + ChronoDuration::from_std(state.config.otp_ttl)
            .unwrap_or_else(|_| ChronoDuration::minutes(10));

    sqlx::query(
        r#"
        INSERT INTO otp_verifications (email, otp_hash, purpose, attempts, expires_at)
        VALUES ($1, $2, $3, 0, $4)
        "#,
    )
    .bind(email)
    .bind(hash_secret(&otp))
    .bind(purpose)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    state.mailer.send_otp(email, purpose, &otp).await;
    Ok(())
}

pub async fn consume_otp(state: &AppState, email: &str, purpose: &str, otp: &str) -> AppResult<()> {
    let mut tx = state.db.begin().await?;
    let row = sqlx::query_as::<_, OtpRow>(
        r#"
        SELECT id, otp_hash, attempts, expires_at, consumed_at
        FROM otp_verifications
        WHERE email = $1 AND purpose = $2
        ORDER BY created_at DESC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(email)
    .bind(purpose)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::BadRequest("invalid otp".into()))?;

    let now = Utc::now();
    if row.consumed_at.is_some() || row.expires_at <= now {
        return Err(AppError::BadRequest(
            "otp expired or already consumed".into(),
        ));
    }

    if row.attempts >= state.config.otp_max_attempts {
        return Err(AppError::BadRequest("otp attempts exceeded".into()));
    }

    if !verify_secret(otp.trim(), &row.otp_hash) {
        sqlx::query("UPDATE otp_verifications SET attempts = attempts + 1 WHERE id = $1")
            .bind(row.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Err(AppError::BadRequest("invalid otp".into()));
    }

    sqlx::query("UPDATE otp_verifications SET consumed_at = now() WHERE id = $1")
        .bind(row.id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn upsert_role(db: &PgPool, name: &str, description: Option<&str>) -> AppResult<Uuid> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO roles (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO UPDATE
        SET description = COALESCE(EXCLUDED.description, roles.description)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(description)
    .fetch_one(db)
    .await?)
}

pub async fn upsert_permission(db: &PgPool, resource: &str, action: &str) -> AppResult<Uuid> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO permissions (resource, action)
        VALUES ($1, $2)
        ON CONFLICT (resource, action) DO UPDATE
        SET resource = EXCLUDED.resource
        RETURNING id
        "#,
    )
    .bind(resource)
    .bind(action)
    .fetch_one(db)
    .await?)
}

// --- Authentication Handlers ---

pub async fn signup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SignupRequest>,
) -> AppResult<Json<AuthResponse>> {
    let email = normalize_email(&payload.email)?;
    validate_password(&payload.password)?;

    let existing =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
            .bind(&email)
            .fetch_one(&state.db)
            .await?;

    if existing {
        return Err(AppError::Conflict("email is already registered".into()));
    }

    let mut tx = state.db.begin().await?;
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, email_verified, status)
        VALUES ($1, false, 'active')
        RETURNING id, email::text AS email, email_verified, status, created_at, updated_at, deleted_at
        "#,
    )
    .bind(&email)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO user_credentials (user_id, password_hash) VALUES ($1, $2)")
        .bind(user.id)
        .bind(password_hash(&payload.password)?)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO profiles (user_id, full_name, timezone)
        VALUES ($1, $2, 'UTC')
        "#,
    )
    .bind(user.id)
    .bind(payload.full_name)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO user_preferences (user_id, theme, language, measurement_system, preferences)
        VALUES ($1, 'system', 'en', 'metric', '{}'::jsonb)
        "#
    )
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    assign_admin_if_user_exists(&state.db, &email).await?;
    create_and_send_otp(&state, &email, PURPOSE_EMAIL_VERIFICATION).await?;

    let auth = create_session_response(
        &state,
        user,
        device_name_from(&headers, None),
        None,
        user_agent(&headers),
    )
    .await?;

    Ok(Json(auth))
}

pub async fn signin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SigninRequest>,
) -> AppResult<Json<AuthResponse>> {
    let email = normalize_email(&payload.email)?;
    let user = find_active_user_by_email(&state, &email).await?;
    let credentials = sqlx::query_as::<_, CredentialRow>(
        "SELECT user_id, password_hash FROM user_credentials WHERE user_id = $1",
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if credentials.user_id != user.id
        || !verify_password(&payload.password, &credentials.password_hash)?
    {
        return Err(AppError::Unauthorized);
    }

    let auth = create_session_response(
        &state,
        user,
        device_name_from(&headers, payload.device_name),
        None,
        user_agent(&headers),
    )
    .await?;

    Ok(Json(auth))
}

pub async fn signout(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<serde_json::Value>> {
    crate::infra::redis::session_cache::invalidate_session_by_id(&state.redis, &state.db, user.session_id.0).await?;

    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL")
        .bind(user.session_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "signed_out": true })))
}

pub async fn signout_all(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<serde_json::Value>> {
    crate::infra::redis::session_cache::invalidate_all_user_sessions(&state.redis, &state.db, user.id).await?;

    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "signed_out": true })))
}

pub async fn send_verification_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EmailRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let email = normalize_email(&payload.email)?;
    find_active_user_by_email(&state, &email).await?;
    create_and_send_otp(&state, &email, PURPOSE_EMAIL_VERIFICATION).await?;

    Ok(Json(serde_json::json!({ "sent": true })))
}

pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyEmailRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let email = normalize_email(&payload.email)?;
    consume_otp(&state, &email, PURPOSE_EMAIL_VERIFICATION, &payload.otp).await?;

    sqlx::query("UPDATE users SET email_verified = true, updated_at = now() WHERE email = $1")
        .bind(email)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "verified": true })))
}

pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EmailRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let email = normalize_email(&payload.email)?;

    if find_active_user_by_email(&state, &email).await.is_ok() {
        create_and_send_otp(&state, &email, PURPOSE_PASSWORD_RESET).await?;
    }

    Ok(Json(serde_json::json!({ "sent": true })))
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResetPasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let email = normalize_email(&payload.email)?;
    validate_password(&payload.new_password)?;
    let user = find_active_user_by_email(&state, &email).await?;
    consume_otp(&state, &email, PURPOSE_PASSWORD_RESET, &payload.otp).await?;

    sqlx::query(
        r#"
        UPDATE user_credentials
        SET password_hash = $1, password_changed_at = now(), updated_at = now()
        WHERE user_id = $2
        "#,
    )
    .bind(password_hash(&payload.new_password)?)
    .bind(user.id)
    .execute(&state.db)
    .await?;

    crate::infra::redis::session_cache::invalidate_all_user_sessions(&state.redis, &state.db, crate::common::id::UserId(user.id)).await?;

    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "reset": true })))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    validate_password(&payload.new_password)?;

    let credentials = sqlx::query_as::<_, CredentialRow>(
        "SELECT user_id, password_hash FROM user_credentials WHERE user_id = $1",
    )
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;

    if !verify_password(&payload.current_password, &credentials.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    sqlx::query(
        r#"
        UPDATE user_credentials
        SET password_hash = $1, password_changed_at = now(), updated_at = now()
        WHERE user_id = $2
        "#,
    )
    .bind(password_hash(&payload.new_password)?)
    .bind(user.id)
    .execute(&state.db)
    .await?;

    let session_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM sessions WHERE user_id = $1 AND id <> $2 AND revoked_at IS NULL"
    )
    .bind(user.id.0)
    .bind(user.session_id.0)
    .fetch_all(&state.db)
    .await?;
    for sid in session_ids {
        let _ = crate::infra::redis::session_cache::invalidate_session_by_id(&state.redis, &state.db, sid).await;
    }

    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE user_id = $1 AND id <> $2 AND revoked_at IS NULL")
        .bind(user.id)
        .bind(user.session_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "changed": true })))
}

pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<Vec<SessionResponse>>> {
    let sessions = sqlx::query_as::<_, Session>(
        r#"
        SELECT id, user_id, device_name, expires_at, revoked_at, last_seen_at, created_at
        FROM sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        sessions
            .into_iter()
            .map(|session| SessionResponse {
                id: session.id,
                device_name: session.device_name,
                expires_at: session.expires_at,
                revoked_at: session.revoked_at,
                last_seen_at: session.last_seen_at,
                created_at: session.created_at,
            })
            .collect(),
    ))
}

pub async fn revoke_session(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    crate::infra::redis::session_cache::invalidate_session_by_id(&state.redis, &state.db, session_id).await?;

    sqlx::query("UPDATE sessions SET revoked_at = now() WHERE id = $1 AND user_id = $2")
        .bind(session_id)
        .bind(user.id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "revoked": true })))
}

// --- Authorization Handlers ---

pub async fn list_roles(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<Vec<Role>>> {
    require_permission(&state, user.id, "roles", "read").await?;

    let roles = sqlx::query_as::<_, Role>(
        "SELECT id, name, description, created_at FROM roles ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(roles))
}

pub async fn create_role(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateRoleRequest>,
) -> AppResult<Json<Role>> {
    require_permission(&state, user.id, "roles", "create").await?;
    let name = clean_key(&payload.name, "role name")?;

    let role = sqlx::query_as::<_, Role>(
        r#"
        INSERT INTO roles (name, description)
        VALUES ($1, $2)
        ON CONFLICT (name) DO UPDATE
        SET description = EXCLUDED.description
        RETURNING id, name, description, created_at
        "#,
    )
    .bind(name)
    .bind(payload.description)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(role))
}

pub async fn list_permissions(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<Vec<Permission>>> {
    require_permission(&state, user.id, "permissions", "read").await?;

    let permissions = sqlx::query_as::<_, Permission>(
        "SELECT id, resource, action, created_at FROM permissions ORDER BY resource, action",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(permissions))
}

pub async fn create_permission(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreatePermissionRequest>,
) -> AppResult<Json<Permission>> {
    require_permission(&state, user.id, "permissions", "create").await?;
    let resource = clean_key(&payload.resource, "resource")?;
    let action = clean_key(&payload.action, "action")?;

    let permission = sqlx::query_as::<_, Permission>(
        r#"
        INSERT INTO permissions (resource, action)
        VALUES ($1, $2)
        ON CONFLICT (resource, action) DO UPDATE
        SET resource = EXCLUDED.resource
        RETURNING id, resource, action, created_at
        "#,
    )
    .bind(resource)
    .bind(action)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(permission))
}

pub async fn assign_permission_to_role(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((role_id, permission_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    require_permission(&state, user.id, "role_permissions", "manage").await?;

    sqlx::query(
        "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(role_id)
    .bind(permission_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "assigned": true })))
}

pub async fn remove_permission_from_role(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((role_id, permission_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    require_permission(&state, user.id, "role_permissions", "manage").await?;

    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2")
        .bind(role_id)
        .bind(permission_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "removed": true })))
}

pub async fn assign_role_to_user(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    require_permission(&state, user.id, "user_roles", "manage").await?;

    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(role_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "assigned": true })))
}

pub async fn remove_role_from_user(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    require_permission(&state, user.id, "user_roles", "manage").await?;

    sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
        .bind(user_id)
        .bind(role_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "removed": true })))
}
