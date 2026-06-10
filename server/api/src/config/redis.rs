use crate::common::{AppError, AppResult};

pub fn connect(redis_url: &str) -> AppResult<redis::Client> {
    let client = redis::Client::open(redis_url)
        .map_err(|error| AppError::BadRequest(format!("failed to open redis client: {error}")))?;
    Ok(client)
}

pub async fn get_connection(
    client: &redis::Client,
) -> AppResult<redis::aio::MultiplexedConnection> {
    let connection = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|error| {
            AppError::BadRequest(format!("failed to establish redis connection: {error}"))
        })?;
    Ok(connection)
}
