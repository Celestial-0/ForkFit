use redis::AsyncCommands;
use uuid::Uuid;
use crate::common::AppResult;
use crate::common::id::UserId;
use crate::middleware::CurrentUser;

pub async fn get_cached_session(
    client: &redis::Client,
    token_hash: &str,
) -> AppResult<Option<CurrentUser>> {
    let mut conn = crate::config::redis::get_connection(client).await?;
    let key = format!("session:by_token:{}", token_hash);
    let cached: Option<String> = conn.get(&key).await.ok();
    
    if let Some(json_str) = cached {
        if let Ok(user) = serde_json::from_str::<CurrentUser>(&json_str) {
            return Ok(Some(user));
        }
    }
    Ok(None)
}

pub async fn set_cached_session(
    client: &redis::Client,
    token_hash: &str,
    session_id: Uuid,
    user: &CurrentUser,
    ttl_secs: u64,
) -> AppResult<()> {
    let mut conn = crate::config::redis::get_connection(client).await?;
    let token_key = format!("session:by_token:{}", token_hash);
    let id_key = format!("session:by_id:{}", session_id);
    let json_str = serde_json::to_string(user).unwrap_or_default();

    if !json_str.is_empty() {
        let _: () = redis::pipe()
            .atomic()
            .set_ex(&token_key, &json_str, ttl_secs)
            .set_ex(&id_key, token_hash, ttl_secs)
            .query_async(&mut conn)
            .await
            .unwrap_or(());
    }
    Ok(())
}

pub async fn invalidate_session_by_id(
    client: &redis::Client,
    db: &sqlx::PgPool,
    session_id: Uuid,
) -> AppResult<()> {
    let mut conn = crate::config::redis::get_connection(client).await?;
    let id_key = format!("session:by_id:{}", session_id);
    let token_hash: Option<String> = conn.get(&id_key).await.ok();

    if let Some(hash) = token_hash {
        let token_key = format!("session:by_token:{}", hash);
        let _: () = conn.del(&[id_key, token_key]).await.unwrap_or(());
    } else {
        // Fallback: lookup in db to find token_hash in case it expired from Redis
        if let Some(hash) = sqlx::query_scalar::<_, String>(
            "SELECT token_hash FROM sessions WHERE id = $1"
        )
        .bind(session_id)
        .fetch_optional(db)
        .await? {
            let token_key = format!("session:by_token:{}", hash);
            let _: () = conn.del(&[id_key, token_key]).await.unwrap_or(());
        }
    }
    Ok(())
}

pub async fn invalidate_all_user_sessions(
    client: &redis::Client,
    db: &sqlx::PgPool,
    user_id: UserId,
) -> AppResult<()> {
    // 1. Get all active sessions for the user from Postgres
    let session_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM sessions WHERE user_id = $1 AND revoked_at IS NULL"
    )
    .bind(user_id.0)
    .fetch_all(db)
    .await?;

    for sid in session_ids {
        let _ = invalidate_session_by_id(client, db, sid).await;
    }
    Ok(())
}
