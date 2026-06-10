use crate::common::AppResult;
use crate::common::id::{UserId, TraceId, MemoryId};

use crate::intelligence::repository::IntelligenceRepository;
use crate::intelligence::error::IntelligenceError;
use crate::intelligence::types::{
    TraceResponse, StepResponse, TraceDetailResponse, MemoryResponse, VectorSearchResultResponse,
};

#[derive(Clone)]
pub struct IntelligenceService<R> {
    repo: R,
}

impl<R: IntelligenceRepository> IntelligenceService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    // Traces
    pub async fn get_trace(&self, user_id: UserId, id: TraceId) -> AppResult<TraceDetailResponse> {
        let trace = self.repo.get_trace(id).await?
            .ok_or(IntelligenceError::NotFound)?;

        if let Some(trace_user_id) = trace.user_id {
            if trace_user_id != user_id {
                return Err(IntelligenceError::Unauthorized.into());
            }
        }

        let steps = self.repo.get_steps_for_trace(id).await?;
        let step_responses = steps.into_iter().map(StepResponse::from).collect();

        Ok(TraceDetailResponse {
            trace: TraceResponse::from(trace),
            steps: step_responses,
        })
    }

    pub async fn list_traces(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<TraceResponse>, u64)> {
        let (traces, total) = self.repo.list_traces(user_id, page, per_page).await?;
        let responses = traces.into_iter().map(TraceResponse::from).collect();
        Ok((responses, total))
    }

    // Memories
    pub async fn list_active_memories(&self, user_id: UserId) -> AppResult<Vec<MemoryResponse>> {
        let memories = self.repo.get_active_memories(user_id).await?;
        Ok(memories.into_iter().map(MemoryResponse::from).collect())
    }

    pub async fn deactivate_memory(&self, user_id: UserId, id: MemoryId) -> AppResult<()> {
        let memory = self.repo.get_memory(id).await?
            .ok_or(IntelligenceError::NotFound)?;

        if memory.user_id != user_id {
            return Err(IntelligenceError::Unauthorized.into());
        }

        self.repo.deactivate_memory(id).await?;
        Ok(())
    }

    // Vector Similarity Search
    pub async fn search_memories(
        &self,
        user_id: UserId,
        embedding: Vec<f32>,
        limit: Option<i64>,
    ) -> AppResult<Vec<VectorSearchResultResponse>> {
        let limit_val = limit.unwrap_or(5);
        if limit_val <= 0 {
            return Err(IntelligenceError::ValidationError("search limit must be greater than zero".to_string()).into());
        }

        let embedding_str = self.format_embedding(&embedding)?;
        let results = self.repo.search_memories_by_embedding(user_id, embedding_str, limit_val).await?;

        Ok(results
            .into_iter()
            .map(|(id, text, dist)| VectorSearchResultResponse {
                id: id.into(),
                chunk_text: text,
                distance: dist,
            })
            .collect())
        }

    // Helper to format float vectors into pgvector string representations
    pub fn format_embedding(&self, embedding: &[f32]) -> Result<String, IntelligenceError> {
        if embedding.is_empty() {
            return Err(IntelligenceError::ValidationError("embedding vector cannot be empty".to_string()));
        }
        let parts: Vec<String> = embedding.iter().map(|f| f.to_string()).collect();
        Ok(format!("[{}]", parts.join(",")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use crate::intelligence::repository::IntelligenceRepository;
    use crate::common::id::RecipeId;
    use crate::intelligence::models::{AiExecutionTrace, AiExecutionStep, AgentMemory};
    use uuid::Uuid;
    use chrono::Utc;

    #[derive(Default)]
    struct MockIntelligenceRepository {
        traces: Mutex<Vec<AiExecutionTrace>>,
        steps: Mutex<Vec<AiExecutionStep>>,
        memories: Mutex<Vec<AgentMemory>>,
    }

    impl IntelligenceRepository for MockIntelligenceRepository {
        async fn create_trace(&self, trace: AiExecutionTrace) -> AppResult<AiExecutionTrace> {
            self.traces.lock().unwrap().push(trace.clone());
            Ok(trace)
        }

        async fn get_trace(&self, id: TraceId) -> AppResult<Option<AiExecutionTrace>> {
            let traces = self.traces.lock().unwrap();
            let t = traces.iter().find(|x| x.id == id).cloned();
            Ok(t)
        }

        async fn list_traces(&self, user_id: UserId, _page: u64, _per_page: u64) -> AppResult<(Vec<AiExecutionTrace>, u64)> {
            let traces = self.traces.lock().unwrap();
            let list: Vec<AiExecutionTrace> = traces.iter().filter(|x| x.user_id == Some(user_id)).cloned().collect();
            let len = list.len() as u64;
            Ok((list, len))
        }

        async fn update_trace_status(
            &self,
            id: TraceId,
            status: String,
            completed_at: Option<chrono::DateTime<chrono::Utc>>,
            latency_ms: Option<i32>,
            total_tokens: Option<i32>,
            total_cost: Option<f64>,
        ) -> AppResult<AiExecutionTrace> {
            let mut traces = self.traces.lock().unwrap();
            let trace = traces.iter_mut().find(|x| x.id == id).unwrap();
            trace.status = status;
            trace.completed_at = completed_at;
            trace.total_latency_ms = latency_ms;
            trace.total_tokens = total_tokens;
            trace.total_cost = total_cost;
            Ok(trace.clone())
        }

        async fn create_step(&self, step: AiExecutionStep) -> AppResult<AiExecutionStep> {
            self.steps.lock().unwrap().push(step.clone());
            Ok(step)
        }

        async fn get_steps_for_trace(&self, trace_id: TraceId) -> AppResult<Vec<AiExecutionStep>> {
            let steps = self.steps.lock().unwrap();
            let list: Vec<AiExecutionStep> = steps.iter().filter(|x| x.trace_id == trace_id).cloned().collect();
            Ok(list)
        }

        async fn create_memory(&self, memory: AgentMemory) -> AppResult<AgentMemory> {
            self.memories.lock().unwrap().push(memory.clone());
            Ok(memory)
        }

        async fn get_memory(&self, id: MemoryId) -> AppResult<Option<AgentMemory>> {
            let memories = self.memories.lock().unwrap();
            let m = memories.iter().find(|x| x.id == id).cloned();
            Ok(m)
        }

        async fn get_active_memories(&self, user_id: UserId) -> AppResult<Vec<AgentMemory>> {
            let memories = self.memories.lock().unwrap();
            let list: Vec<AgentMemory> = memories.iter().filter(|x| x.user_id == user_id && x.is_active).cloned().collect();
            Ok(list)
        }

        async fn deactivate_memory(&self, id: MemoryId) -> AppResult<()> {
            let mut memories = self.memories.lock().unwrap();
            let m = memories.iter_mut().find(|x| x.id == id).unwrap();
            m.is_active = false;
            Ok(())
        }

        async fn search_recipes_by_embedding(&self, _embedding_str: String, _limit: i64) -> AppResult<Vec<(RecipeId, String, f64)>> {
            Ok(vec![])
        }

        async fn search_memories_by_embedding(&self, _user_id: UserId, _embedding_str: String, _limit: i64) -> AppResult<Vec<(MemoryId, String, f64)>> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_format_embedding() {
        let repo = MockIntelligenceRepository::default();
        let service = IntelligenceService::new(repo);

        let valid = vec![0.123f32, -4.56f32, 99.0f32];
        let res = service.format_embedding(&valid);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "[0.123,-4.56,99]");

        let empty: Vec<f32> = vec![];
        let res = service.format_embedding(&empty);
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_trace_ownership() {
        let repo = MockIntelligenceRepository::default();
        let user1 = UserId::new();
        let user2 = UserId::new();
        let trace_id = TraceId::new();

        repo.create_trace(AiExecutionTrace {
            id: trace_id,
            user_id: Some(user1),
            session_type: "chat_response".to_string(),
            session_id: Some(Uuid::new_v4()),
            status: "running".to_string(),
            started_at: Utc::now(),
            completed_at: None,
            total_latency_ms: None,
            total_tokens: None,
            total_cost: None,
        }).await.unwrap();

        let service = IntelligenceService::new(repo);

        // Fetch by owner should work
        let res = service.get_trace(user1, trace_id).await;
        assert!(res.is_ok());

        // Fetch by other user should be Forbidden (Unauthorized)
        let res = service.get_trace(user2, trace_id).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_deactivate_memory_ownership() {
        let repo = MockIntelligenceRepository::default();
        let user1 = UserId::new();
        let user2 = UserId::new();
        let memory_id = MemoryId::new();

        repo.create_memory(AgentMemory {
            id: memory_id,
            user_id: user1,
            memory_type: "preference".to_string(),
            content: "Likes paneer".to_string(),
            confidence: 1.0,
            importance: 5,
            is_active: true,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }).await.unwrap();

        let service = IntelligenceService::new(repo);

        // Deactivate other user's memory should be Forbidden
        let res = service.deactivate_memory(user2, memory_id).await;
        assert!(res.is_err());

        // Deactivate own memory should succeed
        let res = service.deactivate_memory(user1, memory_id).await;
        assert!(res.is_ok());

        // Check if no longer active
        let active = service.list_active_memories(user1).await.unwrap();
        assert_eq!(active.len(), 0);
    }
}
