use crate::common::{AppResult, AppError};
use redis::AsyncCommands;

pub async fn acquire_orchestration_lock(client: &redis::Client, user_id: String) -> AppResult<()> {
    let mut conn = crate::config::redis::get_connection(client).await?;
    let lock_key = format!("lock:orchestrate:{}", user_id);
    
    let success: bool = redis::cmd("SET")
        .arg(&lock_key)
        .arg("locked")
        .arg("NX")
        .arg("EX")
        .arg(300)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Redis(e))?;

    if success {
        Ok(())
    } else {
        Err(AppError::Conflict("An intelligence orchestration is already in progress for this user".to_string()))
    }
}

pub async fn release_orchestration_lock(client: &redis::Client, user_id: String) -> AppResult<()> {
    let mut conn = crate::config::redis::get_connection(client).await?;
    let lock_key = format!("lock:orchestrate:{}", user_id);
    let _: i32 = conn.del(&lock_key).await?;
    Ok(())
}
