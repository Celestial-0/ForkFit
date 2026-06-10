#![allow(dead_code)]
pub mod mock_grpc;

use std::sync::Arc;
use sqlx::PgPool;
use uuid::Uuid;
use api::app::AppState;
use api::config::Config;
use api::gateway::grpc_client::GrpcIntelligenceClient;
use api::notification::Mailer;
use api::common::id::{UserId, ChatThreadId};

pub async fn get_free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind to free port");
    listener.local_addr().unwrap().port()
}

pub async fn setup_test_state(fail: bool, final_text_only: bool) -> (Arc<AppState>, PgPool, redis::Client) {
    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://yash:yash1234@127.0.0.1:5432/forkfit".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://:yash1234@127.0.0.1:6379".to_string());

    // Database connection
    let db = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Redis connection
    let redis = redis::Client::open(redis_url.clone())
        .expect("Failed to connect to Redis");

    // Set up mock gRPC server on a dynamic port
    let grpc_port = get_free_port().await;
    let grpc_addr = format!("127.0.0.1:{}", grpc_port);
    
    // Set environment variable so the grpc client connects here
    unsafe {
        std::env::set_var("INTELLIGENCE_GRPC_URL", format!("http://{}", grpc_addr));
    }

    let service = mock_grpc::MockIntelligenceService {
        fail_orchestrate: fail,
        final_text_only,
    };
    let addr = grpc_addr.parse().unwrap();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(
                api::gateway::grpc_client::intelligence::intelligence_service_server::IntelligenceServiceServer::new(service)
            )
            .serve(addr)
            .await
            .unwrap();
    });

    // Short sleep to ensure the gRPC server starts
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Config Setup
    let config = Config {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: database_url.clone(),
        redis_url: redis_url.clone(),
        google_client_id: "test".to_string(),
        google_client_secret: "test".to_string(),
        github_client_id: "test".to_string(),
        github_client_secret: "test".to_string(),
        admin_email: None,
        session_ttl: std::time::Duration::from_secs(3600),
        otp_ttl: std::time::Duration::from_secs(600),
        otp_max_attempts: 5,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
        smtp_password: None,
        smtp_from: None,
    };

    let mailer = Mailer::new(&config);
    let grpc = GrpcIntelligenceClient::new().await.expect("Failed to connect to mock gRPC");

    let state = Arc::new(AppState {
        db: db.clone(),
        redis: redis.clone(),
        grpc,
        config,
        mailer,
        trace_channels: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
    });

    (state, db, redis)
}

pub async fn setup_test_user(db: &PgPool) -> (UserId, String) {
    let user_id = UserId::new();
    let email = format!("user_{}@example.com", Uuid::new_v4());
    
    let mut tx = db.begin().await.expect("Failed to start transaction");

    sqlx::query(
        "INSERT INTO users (id, email, email_verified, status) VALUES ($1, $2, true, 'active')"
    )
    .bind(user_id)
    .bind(&email)
    .execute(&mut *tx)
    .await
    .expect("Failed to seed test user");

    sqlx::query(
        "INSERT INTO user_credentials (user_id, password_hash) VALUES ($1, $2)"
    )
    .bind(user_id)
    .bind("mock_password_hash")
    .execute(&mut *tx)
    .await
    .expect("Failed to seed test user credentials");

    tx.commit().await.expect("Failed to commit transaction");

    (user_id, email)
}

pub async fn create_test_session(db: &sqlx::PgPool, user_id: UserId) -> String {
    let token = api::common::crypto::generate_session_token();
    let token_hash = api::common::crypto::hash_secret(&token);
    
    let session_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, device_name, expires_at, created_at, last_seen_at) VALUES ($1, $2, $3, 'Test Device', now() + interval '1 hour', now(), now())"
    )
    .bind(session_id)
    .bind(user_id)
    .bind(&token_hash)
    .execute(db)
    .await
    .expect("Failed to create test session");

    token
}

pub async fn create_test_thread(db: &sqlx::PgPool, user_id: UserId) -> ChatThreadId {
    let thread_id = ChatThreadId::new();
    sqlx::query(
        "INSERT INTO chat_threads (id, user_id, agent_type, created_at, updated_at) VALUES ($1, $2, 'nutritionist', now(), now())"
    )
    .bind(thread_id)
    .bind(user_id)
    .execute(db)
    .await
    .expect("Failed to create test thread");

    thread_id
}
