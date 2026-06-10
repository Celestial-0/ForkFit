use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::common::id::{UserId, TraceId, MemoryId};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiExecutionTrace {
    pub id: TraceId,
    pub user_id: Option<UserId>,
    pub session_type: String, // 'chat_response', 'meal_plan_generation'
    pub session_id: Option<Uuid>, // links to thread_id, etc.
    pub status: String, // 'running', 'completed', 'failed'
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub total_latency_ms: Option<i32>,
    pub total_tokens: Option<i32>,
    pub total_cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiExecutionStep {
    pub id: Uuid,
    pub trace_id: TraceId,
    pub parent_step_id: Option<Uuid>,
    pub step_name: String, // e.g. 'Safety Agent check'
    pub step_type: String, // 'llm_call', 'tool_call', 'retrieval'
    pub status: String, // 'running', 'completed', 'failed'
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    pub id: MemoryId,
    pub user_id: UserId,
    pub memory_type: String, // 'preference', 'restriction', 'habit'
    pub content: String,
    pub confidence: f64,
    pub importance: i32, // 1 to 10
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
