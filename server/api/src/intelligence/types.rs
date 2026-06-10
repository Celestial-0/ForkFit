use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::common::id::{UserId, TraceId, MemoryId, ChatThreadId};
use super::models::{AiExecutionTrace, AiExecutionStep, AgentMemory};
use uuid::Uuid;

// Traces
#[derive(Debug, Clone, Serialize)]
pub struct TraceResponse {
    pub id: TraceId,
    pub user_id: Option<UserId>,
    pub session_type: String,
    pub session_id: Option<Uuid>,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_latency_ms: Option<i32>,
    pub total_tokens: Option<i32>,
    pub total_cost: Option<f64>,
}

impl From<AiExecutionTrace> for TraceResponse {
    fn from(t: AiExecutionTrace) -> Self {
        Self {
            id: t.id,
            user_id: t.user_id,
            session_type: t.session_type,
            session_id: t.session_id,
            status: t.status,
            started_at: t.started_at,
            completed_at: t.completed_at,
            total_latency_ms: t.total_latency_ms,
            total_tokens: t.total_tokens,
            total_cost: t.total_cost,
        }
    }
}

// Steps
#[derive(Debug, Clone, Serialize)]
pub struct StepResponse {
    pub id: Uuid,
    pub trace_id: TraceId,
    pub parent_step_id: Option<Uuid>,
    pub step_name: String,
    pub step_type: String,
    pub status: String,
    pub input_payload: serde_json::Value,
    pub output_payload: serde_json::Value,
    pub model_name: Option<String>,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub latency_ms: Option<i32>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<AiExecutionStep> for StepResponse {
    fn from(s: AiExecutionStep) -> Self {
        Self {
            id: s.id,
            trace_id: s.trace_id,
            parent_step_id: s.parent_step_id,
            step_name: s.step_name,
            step_type: s.step_type,
            status: s.status,
            input_payload: s.input_payload,
            output_payload: s.output_payload,
            model_name: s.model_name,
            prompt_tokens: s.prompt_tokens,
            completion_tokens: s.completion_tokens,
            latency_ms: s.latency_ms,
            error_message: s.error_message,
            started_at: s.started_at,
            completed_at: s.completed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceDetailResponse {
    pub trace: TraceResponse,
    pub steps: Vec<StepResponse>,
}

// Memories
#[derive(Debug, Clone, Serialize)]
pub struct MemoryResponse {
    pub id: MemoryId,
    pub user_id: UserId,
    pub memory_type: String,
    pub content: String,
    pub confidence: f64,
    pub importance: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AgentMemory> for MemoryResponse {
    fn from(m: AgentMemory) -> Self {
        Self {
            id: m.id,
            user_id: m.user_id,
            memory_type: m.memory_type,
            content: m.content,
            confidence: m.confidence,
            importance: m.importance,
            is_active: m.is_active,
            metadata: m.metadata,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

// Vector Search
#[derive(Debug, Clone, Deserialize)]
pub struct MemorySearchRequest {
    pub embedding: Vec<f32>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VectorSearchResultResponse {
    pub id: Uuid,
    pub chunk_text: String,
    pub distance: f64,
}

// Orchestration & Intent requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrateRequest {
    pub thread_id: ChatThreadId,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrateResponse {
    pub trace_id: TraceId,
    pub stream_url: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntentRequestPayload {
    pub prompt: String,
}
