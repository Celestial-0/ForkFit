use crate::common::AppResult;
use crate::common::id::{UserId, TraceId, MemoryId, RecipeId};
use super::models::{AiExecutionTrace, AiExecutionStep, AgentMemory};

pub trait IntelligenceRepository: Send + Sync {
    // Traces & Steps
    async fn create_trace(&self, trace: AiExecutionTrace) -> AppResult<AiExecutionTrace>;
    async fn get_trace(&self, id: TraceId) -> AppResult<Option<AiExecutionTrace>>;
    async fn list_traces(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<AiExecutionTrace>, u64)>;
    async fn update_trace_status(
        &self,
        id: TraceId,
        status: String,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
        latency_ms: Option<i32>,
        total_tokens: Option<i32>,
        total_cost: Option<f64>,
    ) -> AppResult<AiExecutionTrace>;

    async fn create_step(&self, step: AiExecutionStep) -> AppResult<AiExecutionStep>;
    async fn get_steps_for_trace(&self, trace_id: TraceId) -> AppResult<Vec<AiExecutionStep>>;

    // Memory
    async fn create_memory(&self, memory: AgentMemory) -> AppResult<AgentMemory>;
    async fn get_memory(&self, id: MemoryId) -> AppResult<Option<AgentMemory>>;
    async fn get_active_memories(&self, user_id: UserId) -> AppResult<Vec<AgentMemory>>;
    async fn deactivate_memory(&self, id: MemoryId) -> AppResult<()>;

    // pgvector Cosine Distance Search
    async fn search_recipes_by_embedding(&self, embedding_str: String, limit: i64) -> AppResult<Vec<(RecipeId, String, f64)>>;
    async fn search_memories_by_embedding(&self, user_id: UserId, embedding_str: String, limit: i64) -> AppResult<Vec<(MemoryId, String, f64)>>;
}
