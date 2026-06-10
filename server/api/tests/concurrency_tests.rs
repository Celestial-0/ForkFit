mod common;

use std::time::Instant;
use api::infra::redis::session_cache::{get_cached_session, set_cached_session, invalidate_session_by_id};
use api::middleware::CurrentUser;
use api::common::id::SessionId;

#[tokio::test]
async fn bench_session_validation() {
    let (_state, db, redis) = common::setup_test_state(false, false).await;
    let (user_id, email) = common::setup_test_user(&db).await;
    let token = common::create_test_session(&db, user_id).await;
    let token_hash = api::common::crypto::hash_secret(&token);

    // 1. Establish cache context
    let session_id = uuid::Uuid::new_v4();
    let current_user = CurrentUser {
        id: user_id,
        session_id: SessionId(session_id),
        email,
        email_verified: true,
        status: "active".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Benchmark DB Lookup (Simulated check)
    let db_start = Instant::now();
    for _ in 0..50 {
        let _user_db: Option<CurrentUser> = sqlx::query_as(
            r#"
            SELECT u.id, s.id as session_id, u.email, u.email_verified, u.status, u.created_at, u.updated_at
            FROM users u
            JOIN sessions s ON s.user_id = u.id
            WHERE s.token_hash = $1 AND s.revoked_at IS NULL AND s.expires_at > now()
            "#
        )
        .bind(&token_hash)
        .fetch_optional(&db)
        .await
        .unwrap();
    }
    let db_duration = db_start.elapsed();
    println!("Database session verification latency (50 runs): {:?}", db_duration);

    // Seed session in Redis
    set_cached_session(&redis, &token_hash, session_id, &current_user, 3600).await.unwrap();

    // Benchmark Cache Lookup
    let cache_start = Instant::now();
    for _ in 0..50 {
        let _user_cache = get_cached_session(&redis, &token_hash).await.unwrap().unwrap();
    }
    let cache_duration = cache_start.elapsed();
    println!("Redis Cache session verification latency (50 runs): {:?}", cache_duration);

    // Cache lookup should typically be faster, print the comparison
    println!("Cache lookup ({:?}) vs DB lookup ({:?})", cache_duration, db_duration);

    // Verify invalidation works correctly
    invalidate_session_by_id(&redis, &db, session_id).await.unwrap();
    let cached_after_invalidation = get_cached_session(&redis, &token_hash).await.unwrap();
    assert!(cached_after_invalidation.is_none(), "Session cache should be empty after invalidation");
}
