use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    app::AppState,
    common::{
        AppError, AppResult, hash_secret,
        id::{UserId, SessionId},
    },
    user::{User, UserResponse},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromRow)]
pub struct CurrentUser {
    pub id: UserId,
    pub session_id: SessionId,
    pub email: String,
    pub email_verified: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct AuthSessionRow {
    session_id: Uuid,
    user_id: Uuid,
    email: String,
    email_verified: bool,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl CurrentUser {
    pub fn user_response(&self) -> UserResponse {
        UserResponse {
            id: self.id.0,
            email: self.email.clone(),
            email_verified: self.email_verified,
            status: self.status.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRequestParts<Arc<AppState>> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts)?;
        let token_hash = hash_secret(token);

        // Check Redis cache first
        if let Ok(Some(cached_user)) = crate::infra::redis::session_cache::get_cached_session(&state.redis, &token_hash).await {
            let session_uuid = cached_user.session_id.0;
            let db_clone = state.db.clone();
            tokio::spawn(async move {
                let _ = sqlx::query("UPDATE sessions SET last_seen_at = now() WHERE id = $1")
                    .bind(session_uuid)
                    .execute(&db_clone)
                    .await;
            });
            return Ok(cached_user);
        }

        let row = sqlx::query_as::<_, AuthSessionRow>(
            r#"
            SELECT
                sessions.id AS session_id,
                users.id AS user_id,
                users.email::text AS email,
                users.email_verified,
                users.status,
                users.created_at,
                users.updated_at,
                users.deleted_at,
                sessions.expires_at,
                sessions.revoked_at
            FROM sessions
            JOIN users ON users.id = sessions.user_id
            WHERE sessions.token_hash = $1
            "#,
        )
        .bind(token_hash.clone())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

        let now = Utc::now();
        if row.revoked_at.is_some()
            || row.expires_at <= now
            || row.deleted_at.is_some()
            || row.status != "active"
        {
            return Err(AppError::Unauthorized);
        }

        sqlx::query("UPDATE sessions SET last_seen_at = now() WHERE id = $1")
            .bind(row.session_id)
            .execute(&state.db)
            .await?;

        let current_user = Self {
            id: UserId(row.user_id),
            session_id: SessionId(row.session_id),
            email: row.email,
            email_verified: row.email_verified,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        // Cache the session in Redis (aligning TTL with session expiry)
        let ttl_secs = if row.expires_at > now {
            (row.expires_at - now).num_seconds() as u64
        } else {
            0
        };
        if ttl_secs > 0 {
            let _ = crate::infra::redis::session_cache::set_cached_session(
                &state.redis,
                &token_hash,
                row.session_id,
                &current_user,
                ttl_secs,
            )
            .await;
        }

        Ok(current_user)
    }
}

pub async fn require_permission(
    state: &AppState,
    user_id: UserId,
    resource: &str,
    action: &str,
) -> AppResult<()> {
    let allowed = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM user_roles
            JOIN role_permissions ON role_permissions.role_id = user_roles.role_id
            JOIN permissions ON permissions.id = role_permissions.permission_id
            WHERE user_roles.user_id = $1
              AND permissions.resource = $2
              AND permissions.action = $3
        )
        "#,
    )
    .bind(user_id)
    .bind(resource)
    .bind(action)
    .fetch_one(&state.db)
    .await?;

    if allowed {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn bearer_token(parts: &Parts) -> AppResult<&str> {
    let header = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    header
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or(AppError::Unauthorized)
}

#[allow(dead_code)]
pub fn current_user_from_user(user: User, session_id: Uuid) -> CurrentUser {
    CurrentUser {
        id: UserId(user.id),
        session_id: SessionId(session_id),
        email: user.email,
        email_verified: user.email_verified,
        status: user.status,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderValue, Request};

    use super::*;

    #[test]
    fn reads_bearer_token() {
        let mut request = Request::builder().body(()).unwrap();
        request.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc123"),
        );
        let (parts, _) = request.into_parts();

        assert_eq!(bearer_token(&parts).unwrap(), "abc123");
    }

    #[test]
    fn rejects_missing_bearer_prefix() {
        let mut request = Request::builder().body(()).unwrap();
        request
            .headers_mut()
            .insert(header::AUTHORIZATION, HeaderValue::from_static("abc123"));
        let (parts, _) = request.into_parts();

        assert!(bearer_token(&parts).is_err());
    }
}
