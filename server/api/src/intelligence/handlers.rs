use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::app::AppState;
use crate::common::AppResult;
use crate::common::id::{TraceId, MemoryId};
use crate::common::pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
use crate::middleware::CurrentUser;
use crate::infra::pg::intelligence_repo::PgIntelligenceRepository;
use crate::intelligence::models::AiExecutionTrace;
use crate::intelligence::repository::IntelligenceRepository;

use super::service::IntelligenceService;
use super::types::*;

fn make_service(state: &AppState) -> IntelligenceService<PgIntelligenceRepository> {
    let repo = PgIntelligenceRepository::new(state.db.clone());
    IntelligenceService::new(repo)
}

// Helper struct for standard success responses
#[derive(serde::Serialize)]
pub struct StatusCodeResponse {
    pub success: bool,
}

// --- Traces & Steps Handlers ---

pub async fn list_traces(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<TraceResponse>>> {
    let service = make_service(&state);
    let page = params.page();
    let per_page = params.per_page();

    let (traces, total) = service.list_traces(user.id, page, per_page).await?;
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data: traces, meta }))
}

pub async fn get_trace(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<TraceId>,
) -> AppResult<Json<TraceDetailResponse>> {
    let service = make_service(&state);
    let detail = service.get_trace(user.id, id).await?;
    Ok(Json(detail))
}

// --- Memory Handlers ---

pub async fn list_active_memories(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<Vec<MemoryResponse>>> {
    let service = make_service(&state);
    let memories = service.list_active_memories(user.id).await?;
    Ok(Json(memories))
}

pub async fn deactivate_memory(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<MemoryId>,
) -> AppResult<Json<StatusCodeResponse>> {
    let service = make_service(&state);
    service.deactivate_memory(user.id, id).await?;
    Ok(Json(StatusCodeResponse { success: true }))
}

pub async fn search_memories(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<MemorySearchRequest>,
) -> AppResult<Json<Vec<VectorSearchResultResponse>>> {
    let service = make_service(&state);
    let results = service.search_memories(user.id, payload.embedding, payload.limit).await?;
    Ok(Json(results))
}

pub async fn orchestrate(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<OrchestrateRequest>,
) -> AppResult<Json<OrchestrateResponse>> {
    // 1. Acquire Redis lock
    crate::infra::redis::idempotency::acquire_orchestration_lock(&state.redis, user.id.to_string()).await?;

    let trace_id = TraceId::new();

    // 2. Create AI Trace record in DB
    let intel_repo = PgIntelligenceRepository::new(state.db.clone());
    let trace = AiExecutionTrace {
        id: trace_id,
        user_id: Some(user.id),
        session_type: "chat_response".to_string(),
        session_id: Some(payload.thread_id.0),
        status: "running".to_string(),
        started_at: chrono::Utc::now(),
        completed_at: None,
        total_latency_ms: None,
        total_tokens: None,
        total_cost: None,
    };
    intel_repo.create_trace(trace).await?;

    // 3. Create broadcast channel
    let (tx, _) = tokio::sync::broadcast::channel::<super::stream::SseEvent>(1024);

    // 4. Register in shared state registry
    state.trace_channels.lock().await.insert(trace_id.0, tx.clone());

    // 5. Spawn background execution worker
    super::worker::spawn_orchestration_worker(
        state.clone(),
        trace_id,
        user.id,
        user.session_id,
        payload.thread_id,
        payload.prompt,
        tx,
    );

    Ok(Json(OrchestrateResponse {
        trace_id,
        stream_url: format!("/api/v1/intelligence/stream/{}", trace_id),
        status: "running".to_string(),
    }))
}

pub async fn process_intent(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<IntentRequestPayload>,
) -> AppResult<Json<crate::gateway::grpc_client::intelligence::IntentResponse>> {
    let resp = state.grpc.process_intent(payload.prompt, user.id.to_string()).await?;
    Ok(Json(resp))
}
