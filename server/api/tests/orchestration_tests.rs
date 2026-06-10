mod common;

use std::time::Duration;
use futures::StreamExt;
use api::intelligence::types::{OrchestrateRequest, OrchestrateResponse};
use api::infra::pg::intelligence_repo::PgIntelligenceRepository;
use api::intelligence::repository::IntelligenceRepository;
use api::chat::repository::ChatRepository;
use api::infra::pg::chat_repo::PgChatRepository;
use api::plan::repository::PlanRepository;

fn parse_sse_events(body_str: &str) -> Vec<(String, String)> {
    let mut events = Vec::new();
    for event_block in body_str.split("\n\n") {
        if event_block.trim().is_empty() {
            continue;
        }
        let mut event_name = String::new();
        let mut event_data = String::new();
        for line in event_block.lines() {
            if line.starts_with("event:") {
                event_name = line["event:".len()..].trim().to_string();
            } else if line.starts_with("data:") {
                event_data = line["data:".len()..].trim().to_string();
            }
        }
        if !event_name.is_empty() {
            events.push((event_name, event_data));
        }
    }
    events
}

#[tokio::test]
async fn test_orchestration_and_sse_streaming() {
    let (state, db, _) = common::setup_test_state(false, false).await;
    let (user_id, _) = common::setup_test_user(&db).await;
    let token = common::create_test_session(&db, user_id).await;
    let thread_id = common::create_test_thread(&db, user_id).await;

    // Start Axum server locally
    let router = api::app::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();

    // 1. Post Orchestrate Request
    let req_payload = OrchestrateRequest {
        thread_id,
        prompt: "Give me a meal plan".to_string(),
    };

    let res = client.post(format!("http://{}/api/v1/intelligence/orchestrate", addr))
        .bearer_auth(&token)
        .json(&req_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let orch_res: OrchestrateResponse = res.json().await.unwrap();
    assert_eq!(orch_res.status, "running");

    // 2. Read SSE Stream
    let stream_res = client.get(format!("http://{}{}", addr, orch_res.stream_url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert_eq!(stream_res.status(), reqwest::StatusCode::OK);

    let mut body = stream_res.bytes_stream();
    let mut sse_string = String::new();

    while let Some(chunk) = body.next().await {
        let chunk = chunk.unwrap();
        let chunk_str = String::from_utf8(chunk.to_vec()).unwrap();
        sse_string.push_str(&chunk_str);
        
        // As soon as we see "done" or "error", stop reading
        if sse_string.contains("event: done") || sse_string.contains("event: error") {
            break;
        }
    }

    let parsed_events = parse_sse_events(&sse_string);
    assert!(!parsed_events.is_empty());

    // Verify events sequence order
    assert_eq!(parsed_events[0].0, "trace_start");
    assert_eq!(parsed_events[1].0, "agent_step");
    assert_eq!(parsed_events[2].0, "message_delta");
    assert_eq!(parsed_events[3].0, "message_delta");
    assert_eq!(parsed_events[4].0, "ui_element");
    assert_eq!(parsed_events[5].0, "done");

    // Verify delta content concatenation
    let mut assembled_content = String::new();
    for (name, data) in &parsed_events {
        if name == "message_delta" {
            let val: serde_json::Value = serde_json::from_str(data).unwrap();
            assembled_content.push_str(val["content"].as_str().unwrap());
        }
    }
    assert_eq!(assembled_content, "Hello! Here is your muscle gain meal plan.");

    // Short sleep to allow background execution worker task to wrap up DB inserts
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify that the ChatMessage was saved in Postgres
    let chat_repo = PgChatRepository::new(db.clone());
    let (messages, _) = chat_repo.list_messages(thread_id, 1, 10).await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, assembled_content);

    // Verify trace status is completed in Postgres
    let intel_repo = PgIntelligenceRepository::new(db.clone());
    let trace = intel_repo.get_trace(orch_res.trace_id).await.unwrap().unwrap();
    assert_eq!(trace.status, "completed");

    // Verify steps were persisted in Postgres
    let steps = intel_repo.get_steps_for_trace(orch_res.trace_id).await.unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].step_name, "safety_agent");
    assert_eq!(steps[0].step_type, "validation");
    assert_eq!(steps[0].status, "completed");
    assert_eq!(steps[0].latency_ms, Some(120));

    // Verify active meal plan was persisted via the UI element parser handler trigger
    let plan_repo = api::infra::pg::plan_repo::PgPlanRepository::new(db.clone());
    let active_plan = plan_repo.get_active_meal_plan(user_id).await.unwrap();
    assert!(active_plan.is_some());
    assert_eq!(active_plan.unwrap().0.name, Some("AI Muscle Gain Plan".to_string()));
}

#[tokio::test]
async fn test_orchestration_grpc_failure_handling() {
    // Start with mock gRPC failing
    let (state, db, _) = common::setup_test_state(true, false).await;
    let (user_id, _) = common::setup_test_user(&db).await;
    let token = common::create_test_session(&db, user_id).await;
    let thread_id = common::create_test_thread(&db, user_id).await;

    let router = api::app::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let req_payload = OrchestrateRequest {
        thread_id,
        prompt: "Trigger gRPC failure".to_string(),
    };

    let res = client.post(format!("http://{}/api/v1/intelligence/orchestrate", addr))
        .bearer_auth(&token)
        .json(&req_payload)
        .send()
        .await
        .unwrap();

    let orch_res: OrchestrateResponse = res.json().await.unwrap();

    let stream_res = client.get(format!("http://{}{}", addr, orch_res.stream_url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    let mut body = stream_res.bytes_stream();
    let mut sse_string = String::new();

    while let Some(chunk) = body.next().await {
        let chunk = chunk.unwrap();
        let chunk_str = String::from_utf8(chunk.to_vec()).unwrap();
        sse_string.push_str(&chunk_str);
        if sse_string.contains("event: error") {
            break;
        }
    }

    let parsed_events = parse_sse_events(&sse_string);
    assert_eq!(parsed_events[parsed_events.len() - 1].0, "error");

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify trace is failed in Postgres
    let intel_repo = PgIntelligenceRepository::new(db.clone());
    let trace = intel_repo.get_trace(orch_res.trace_id).await.unwrap().unwrap();
    assert_eq!(trace.status, "failed");

    // Verify Redis lock is released
    let mut conn = api::config::redis::get_connection(&state.redis).await.unwrap();
    let lock_key = format!("lock:orchestrate:{}", user_id);
    let holds_lock: bool = redis::cmd("EXISTS").arg(&lock_key).query_async(&mut conn).await.unwrap();
    assert!(!holds_lock);
}

#[tokio::test]
async fn test_orchestration_idempotency_concurrency() {
    let (state, db, _) = common::setup_test_state(false, false).await;
    let (user_id, _) = common::setup_test_user(&db).await;
    let token = common::create_test_session(&db, user_id).await;
    let thread_id = common::create_test_thread(&db, user_id).await;

    let router = api::app::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let req_payload = OrchestrateRequest {
        thread_id,
        prompt: "Concurrent prompt".to_string(),
    };

    // First request
    let res1 = client.post(format!("http://{}/api/v1/intelligence/orchestrate", addr))
        .bearer_auth(&token)
        .json(&req_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res1.status(), reqwest::StatusCode::OK);

    // Immediate second request (should fail with 409 conflict since lock is held)
    let res2 = client.post(format!("http://{}/api/v1/intelligence/orchestrate", addr))
        .bearer_auth(&token)
        .json(&req_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res2.status(), reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_backward_compatibility_final_text() {
    // Start with gRPC mock sending only FinalText
    let (state, db, _) = common::setup_test_state(false, true).await;
    let (user_id, _) = common::setup_test_user(&db).await;
    let token = common::create_test_session(&db, user_id).await;
    let thread_id = common::create_test_thread(&db, user_id).await;

    let router = api::app::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let req_payload = OrchestrateRequest {
        thread_id,
        prompt: "Final text only".to_string(),
    };

    let res = client.post(format!("http://{}/api/v1/intelligence/orchestrate", addr))
        .bearer_auth(&token)
        .json(&req_payload)
        .send()
        .await
        .unwrap();

    let orch_res: OrchestrateResponse = res.json().await.unwrap();

    let stream_res = client.get(format!("http://{}{}", addr, orch_res.stream_url))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    let mut body = stream_res.bytes_stream();
    let mut sse_string = String::new();

    while let Some(chunk) = body.next().await {
        let chunk = chunk.unwrap();
        let chunk_str = String::from_utf8(chunk.to_vec()).unwrap();
        sse_string.push_str(&chunk_str);
        if sse_string.contains("event: done") {
            break;
        }
    }

    let parsed_events = parse_sse_events(&sse_string);
    
    // There should be a message_delta containing the final text synthesized dynamically
    let deltas: Vec<_> = parsed_events.iter().filter(|(name, _)| name == "message_delta").collect();
    assert_eq!(deltas.len(), 1);
    let val: serde_json::Value = serde_json::from_str(&deltas[0].1).unwrap();
    assert_eq!(val["content"].as_str().unwrap(), "Here is the final generated meal plan text without deltas.");
}

#[tokio::test]
async fn test_client_disconnect_resiliency() {
    let (state, db, _) = common::setup_test_state(false, false).await;
    let (user_id, _) = common::setup_test_user(&db).await;
    let token = common::create_test_session(&db, user_id).await;
    let thread_id = common::create_test_thread(&db, user_id).await;

    let router = api::app::router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let client = reqwest::Client::new();
    let req_payload = OrchestrateRequest {
        thread_id,
        prompt: "Disconnect test".to_string(),
    };

    let res = client.post(format!("http://{}/api/v1/intelligence/orchestrate", addr))
        .bearer_auth(&token)
        .json(&req_payload)
        .send()
        .await
        .unwrap();

    let orch_res: OrchestrateResponse = res.json().await.unwrap();

    // Connect and read only first chunk, then drop connection immediately
    {
        let stream_res = client.get(format!("http://{}{}", addr, orch_res.stream_url))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();

        let mut body = stream_res.bytes_stream();
        let _first_chunk = body.next().await.unwrap().unwrap();
        // Drop stream_res and body here
    }

    // Wait 150ms for worker to finish execution asynchronously in background
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify trace is still completed successfully in Postgres
    let intel_repo = PgIntelligenceRepository::new(db.clone());
    let trace = intel_repo.get_trace(orch_res.trace_id).await.unwrap().unwrap();
    assert_eq!(trace.status, "completed");

    // Verify ChatMessage is saved in Postgres
    let chat_repo = PgChatRepository::new(db.clone());
    let (messages, _) = chat_repo.list_messages(thread_id, 1, 10).await.unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "Hello! Here is your muscle gain meal plan.");

    // Verify Redis lock was released
    let mut conn = api::config::redis::get_connection(&state.redis).await.unwrap();
    let lock_key = format!("lock:orchestrate:{}", user_id);
    let holds_lock: bool = redis::cmd("EXISTS").arg(&lock_key).query_async(&mut conn).await.unwrap();
    assert!(!holds_lock);
}
