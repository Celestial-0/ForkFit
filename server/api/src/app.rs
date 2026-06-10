use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::GlobalKeyExtractor,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    auth,
    common::{AppError, AppResult},
    config::Config,
    gateway::grpc_client::GrpcIntelligenceClient,
    notification::Mailer,
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: redis::Client,
    pub grpc: GrpcIntelligenceClient,
    pub config: Config,
    pub mailer: Mailer,
    pub trace_channels: Arc<tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, tokio::sync::broadcast::Sender<crate::intelligence::stream::SseEvent>>>>,
}

pub async fn build_app(config: Config) -> AppResult<Router> {
    // Database connection
    let db = crate::config::db::connect(&config.database_url).await?;

    sqlx::migrate!("./../db/migrations").run(&db).await?;
    auth::seed_defaults(&db, config.admin_email.as_deref()).await?;

    // Redis connection
    let redis = crate::config::redis::connect(&config.redis_url)?;

    // Verify Redis connection with a PING command at startup
    {
        let mut conn = crate::config::redis::get_connection(&redis).await?;
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|err| {
                AppError::BadRequest(format!("Failed to connect to Redis on startup: {err}"))
            })?;
        tracing::info!("redis_connection_verified");
    }

    // gRPC client connection
    let grpc = GrpcIntelligenceClient::new().await?;
    tracing::info!("grpc_intelligence_client_connected");

    let mailer = Mailer::new(&config);

    let state = Arc::new(AppState {
        db,
        redis,
        grpc,
        config,
        mailer,
        trace_channels: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
    });

    // Spawn background worker
    let runner_state = state.clone();
    tokio::spawn(async move {
        crate::background::runner::run_background_jobs(runner_state).await;
    });

    Ok(router(state))
}

pub fn router(state: Arc<AppState>) -> Router {
    let rate_limit = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(120)
        .key_extractor(GlobalKeyExtractor)
        .finish()
        .expect("valid rate limit config");

    Router::new()
        .merge(crate::gateway::router())
        .layer(axum::middleware::from_fn(crate::middleware::request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(GovernorLayer::new(rate_limit))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, time::Duration};

    use super::*;

    #[tokio::test]
    async fn builds_router_without_panicking() {
        let db = crate::config::db::connect_lazy("postgres://user:password@localhost/forkfit")
            .unwrap();
        let redis = redis::Client::open("redis://localhost:6379").unwrap();
        let grpc = GrpcIntelligenceClient::new_lazy().unwrap();

        let config = Config {
            bind_addr: "127.0.0.1:3001".parse::<SocketAddr>().unwrap(),
            database_url: "postgres://user:password@localhost/forkfit".into(),
            redis_url: "redis://localhost:6379".into(),
            google_client_id: "google-id".to_string(),
            google_client_secret: "google-secret".to_string(),
            github_client_id: "github-id".to_string(),
            github_client_secret: "github-secret".to_string(),
            admin_email: None,
            session_ttl: Duration::from_secs(60),
            otp_ttl: Duration::from_secs(60),
            otp_max_attempts: 5,
            smtp_host: None,
            smtp_port: None,
            smtp_user: None,
            smtp_password: None,
            smtp_from: None,
        };
        let mailer = Mailer::new(&config);

        let state = Arc::new(AppState {
            db,
            redis,
            grpc,
            config,
            mailer,
            trace_channels: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        });

        let _router = router(state);
    }
}
