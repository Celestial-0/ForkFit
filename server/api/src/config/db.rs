use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::common::AppResult;

pub async fn connect(database_url: &str) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub fn connect_lazy(database_url: &str) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .connect_lazy(database_url)?;
    Ok(pool)
}
