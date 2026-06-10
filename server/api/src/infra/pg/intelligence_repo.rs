use sqlx::{PgPool, query_scalar, Row};
use uuid::Uuid;

use crate::common::AppResult;
use crate::common::id::{UserId, TraceId, MemoryId, RecipeId};
use crate::intelligence::models::{AiExecutionTrace, AiExecutionStep, AgentMemory};
use crate::intelligence::repository::IntelligenceRepository;

#[derive(Clone)]
pub struct PgIntelligenceRepository {
    pool: PgPool,
}

impl PgIntelligenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl IntelligenceRepository for PgIntelligenceRepository {
    // Traces
    async fn create_trace(&self, trace: AiExecutionTrace) -> AppResult<AiExecutionTrace> {
        let trace_uuid: Uuid = trace.id.into();
        let user_uuid: Option<Uuid> = trace.user_id.map(|id| id.into());
        let cost_val = trace.total_cost;

        let row = sqlx::query!(
            r#"
            INSERT INTO ai_execution_traces (id, user_id, session_type, session_id, status, started_at, completed_at, total_latency_ms, total_tokens, total_cost)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::float8::numeric)
            RETURNING
                id as "id: TraceId",
                user_id as "user_id: UserId",
                session_type,
                session_id,
                status,
                started_at,
                completed_at,
                total_latency_ms,
                total_tokens,
                total_cost::float8 as "total_cost: f64"
            "#,
            trace_uuid,
            user_uuid,
            trace.session_type,
            trace.session_id,
            trace.status,
            trace.started_at,
            trace.completed_at,
            trace.total_latency_ms,
            trace.total_tokens,
            cost_val
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AiExecutionTrace {
            id: row.id,
            user_id: row.user_id,
            session_type: row.session_type,
            session_id: row.session_id,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            total_latency_ms: row.total_latency_ms,
            total_tokens: row.total_tokens,
            total_cost: row.total_cost,
        })
    }

    async fn get_trace(&self, id: TraceId) -> AppResult<Option<AiExecutionTrace>> {
        let trace_uuid: Uuid = id.into();

        let row = sqlx::query!(
            r#"
            SELECT
                id as "id: TraceId",
                user_id as "user_id: UserId",
                session_type,
                session_id,
                status,
                started_at,
                completed_at,
                total_latency_ms,
                total_tokens,
                total_cost::float8 as "total_cost: f64"
            FROM ai_execution_traces
            WHERE id = $1
            "#,
            trace_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| AiExecutionTrace {
            id: r.id,
            user_id: r.user_id,
            session_type: r.session_type,
            session_id: r.session_id,
            status: r.status,
            started_at: r.started_at,
            completed_at: r.completed_at,
            total_latency_ms: r.total_latency_ms,
            total_tokens: r.total_tokens,
            total_cost: r.total_cost,
        }))
    }

    async fn list_traces(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<AiExecutionTrace>, u64)> {
        let user_uuid: Uuid = user_id.into();
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let total = query_scalar!(
            r#"SELECT count(*) FROM ai_execution_traces WHERE user_id = $1"#,
            user_uuid
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id: TraceId",
                user_id as "user_id: UserId",
                session_type,
                session_id,
                status,
                started_at,
                completed_at,
                total_latency_ms,
                total_tokens,
                total_cost::float8 as "total_cost: f64"
            FROM ai_execution_traces
            WHERE user_id = $1
            ORDER BY started_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_uuid,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let traces = rows
            .into_iter()
            .map(|r| AiExecutionTrace {
                id: r.id,
                user_id: r.user_id,
                session_type: r.session_type,
                session_id: r.session_id,
                status: r.status,
                started_at: r.started_at,
                completed_at: r.completed_at,
                total_latency_ms: r.total_latency_ms,
                total_tokens: r.total_tokens,
                total_cost: r.total_cost,
            })
            .collect();

        Ok((traces, total))
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
        let trace_uuid: Uuid = id.into();
        let cost_val = total_cost;

        let row = sqlx::query!(
            r#"
            UPDATE ai_execution_traces
            SET status = $2,
                completed_at = COALESCE($3, completed_at),
                total_latency_ms = COALESCE($4, total_latency_ms),
                total_tokens = COALESCE($5, total_tokens),
                total_cost = COALESCE($6::float8::numeric, total_cost)
            WHERE id = $1
            RETURNING
                id as "id: TraceId",
                user_id as "user_id: UserId",
                session_type,
                session_id,
                status,
                started_at,
                completed_at,
                total_latency_ms,
                total_tokens,
                total_cost::float8 as "total_cost: f64"
            "#,
            trace_uuid,
            status,
            completed_at,
            latency_ms,
            total_tokens,
            cost_val
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AiExecutionTrace {
            id: row.id,
            user_id: row.user_id,
            session_type: row.session_type,
            session_id: row.session_id,
            status: row.status,
            started_at: row.started_at,
            completed_at: row.completed_at,
            total_latency_ms: row.total_latency_ms,
            total_tokens: row.total_tokens,
            total_cost: row.total_cost,
        })
    }

    // Steps
    async fn create_step(&self, step: AiExecutionStep) -> AppResult<AiExecutionStep> {
        let step_uuid: Uuid = step.id;
        let trace_uuid: Uuid = step.trace_id.into();

        let row = sqlx::query!(
            r#"
            INSERT INTO ai_execution_steps (id, trace_id, parent_step_id, step_name, step_type, status, input_payload, output_payload, model_name, prompt_tokens, completion_tokens, latency_ms, error_message, started_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING
                id,
                trace_id as "trace_id: TraceId",
                parent_step_id,
                step_name,
                step_type,
                status,
                input_payload,
                output_payload,
                model_name,
                prompt_tokens,
                completion_tokens,
                latency_ms,
                error_message,
                started_at,
                completed_at
            "#,
            step_uuid,
            trace_uuid,
            step.parent_step_id,
            step.step_name,
            step.step_type,
            step.status,
            step.input_payload,
            step.output_payload,
            step.model_name,
            step.prompt_tokens,
            step.completion_tokens,
            step.latency_ms,
            step.error_message,
            step.started_at,
            step.completed_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AiExecutionStep {
            id: row.id,
            trace_id: row.trace_id,
            parent_step_id: row.parent_step_id,
            step_name: row.step_name,
            step_type: row.step_type,
            status: row.status,
            input_payload: row.input_payload,
            output_payload: row.output_payload,
            model_name: row.model_name,
            prompt_tokens: row.prompt_tokens,
            completion_tokens: row.completion_tokens,
            latency_ms: row.latency_ms,
            error_message: row.error_message,
            started_at: row.started_at,
            completed_at: row.completed_at,
        })
    }

    async fn get_steps_for_trace(&self, trace_id: TraceId) -> AppResult<Vec<AiExecutionStep>> {
        let trace_uuid: Uuid = trace_id.into();

        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                trace_id as "trace_id: TraceId",
                parent_step_id,
                step_name,
                step_type,
                status,
                input_payload,
                output_payload,
                model_name,
                prompt_tokens,
                completion_tokens,
                latency_ms,
                error_message,
                started_at,
                completed_at
            FROM ai_execution_steps
            WHERE trace_id = $1
            ORDER BY started_at ASC
            "#,
            trace_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let steps = rows
            .into_iter()
            .map(|row| AiExecutionStep {
                id: row.id,
                trace_id: row.trace_id,
                parent_step_id: row.parent_step_id,
                step_name: row.step_name,
                step_type: row.step_type,
                status: row.status,
                input_payload: row.input_payload,
                output_payload: row.output_payload,
                model_name: row.model_name,
                prompt_tokens: row.prompt_tokens,
                completion_tokens: row.completion_tokens,
                latency_ms: row.latency_ms,
                error_message: row.error_message,
                started_at: row.started_at,
                completed_at: row.completed_at,
            })
            .collect();

        Ok(steps)
    }

    // Memories
    async fn create_memory(&self, memory: AgentMemory) -> AppResult<AgentMemory> {
        let memory_id: Uuid = memory.id.into();
        let user_uuid: Uuid = memory.user_id.into();

        let row = sqlx::query!(
            r#"
            INSERT INTO agent_memories (id, user_id, memory_type, content, confidence, importance, is_active, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5::float8::numeric, $6, $7, $8, $9, $10)
            RETURNING
                id as "id: MemoryId",
                user_id as "user_id: UserId",
                memory_type,
                content,
                confidence::float8 as "confidence!",
                importance,
                is_active,
                metadata,
                created_at,
                updated_at
            "#,
            memory_id,
            user_uuid,
            memory.memory_type,
            memory.content,
            memory.confidence,
            memory.importance,
            memory.is_active,
            memory.metadata,
            memory.created_at,
            memory.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AgentMemory {
            id: row.id,
            user_id: row.user_id,
            memory_type: row.memory_type,
            content: row.content,
            confidence: row.confidence,
            importance: row.importance,
            is_active: row.is_active,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn get_memory(&self, id: MemoryId) -> AppResult<Option<AgentMemory>> {
        let memory_uuid: Uuid = id.into();

        let row = sqlx::query!(
            r#"
            SELECT
                id as "id: MemoryId",
                user_id as "user_id: UserId",
                memory_type,
                content,
                confidence::float8 as "confidence!",
                importance,
                is_active,
                metadata,
                created_at,
                updated_at
            FROM agent_memories
            WHERE id = $1
            "#,
            memory_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| AgentMemory {
            id: r.id,
            user_id: r.user_id,
            memory_type: r.memory_type,
            content: r.content,
            confidence: r.confidence,
            importance: r.importance,
            is_active: r.is_active,
            metadata: r.metadata,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn get_active_memories(&self, user_id: UserId) -> AppResult<Vec<AgentMemory>> {
        let user_uuid: Uuid = user_id.into();

        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id: MemoryId",
                user_id as "user_id: UserId",
                memory_type,
                content,
                confidence::float8 as "confidence!",
                importance,
                is_active,
                metadata,
                created_at,
                updated_at
            FROM agent_memories
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
            user_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let memories = rows
            .into_iter()
            .map(|r| AgentMemory {
                id: r.id,
                user_id: r.user_id,
                memory_type: r.memory_type,
                content: r.content,
                confidence: r.confidence,
                importance: r.importance,
                is_active: r.is_active,
                metadata: r.metadata,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(memories)
    }

    async fn deactivate_memory(&self, id: MemoryId) -> AppResult<()> {
        let memory_uuid: Uuid = id.into();

        sqlx::query!(
            "UPDATE agent_memories SET is_active = false, updated_at = now() WHERE id = $1",
            memory_uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // pgvector Cosine Distance Search
    async fn search_recipes_by_embedding(&self, embedding_str: String, limit: i64) -> AppResult<Vec<(RecipeId, String, f64)>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                recipe_id, 
                chunk_text, 
                (embedding <=> $1::vector)::float8 as distance
            FROM recipe_embeddings
            ORDER BY embedding <=> $1::vector
            LIMIT $2
            "#,
        )
        .bind(&embedding_str)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for r in rows {
            let recipe_uuid: Uuid = r.try_get("recipe_id")?;
            let chunk_text: String = r.try_get("chunk_text")?;
            let distance: f64 = r.try_get("distance")?;
            results.push((RecipeId(recipe_uuid), chunk_text, distance));
        }

        Ok(results)
    }

    async fn search_memories_by_embedding(&self, user_id: UserId, embedding_str: String, limit: i64) -> AppResult<Vec<(MemoryId, String, f64)>> {
        let user_uuid: Uuid = user_id.into();

        let rows = sqlx::query(
            r#"
            SELECT 
                m.id as memory_id, 
                me.chunk_text, 
                (me.embedding <=> $1::vector)::float8 as distance
            FROM agent_memory_embeddings me
            JOIN agent_memories m ON m.id = me.memory_id
            WHERE m.user_id = $2 AND m.is_active = true
            ORDER BY me.embedding <=> $1::vector
            LIMIT $3
            "#,
        )
        .bind(&embedding_str)
        .bind(user_uuid)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for r in rows {
            let memory_uuid: Uuid = r.try_get("memory_id")?;
            let chunk_text: String = r.try_get("chunk_text")?;
            let distance: f64 = r.try_get("distance")?;
            results.push((MemoryId(memory_uuid), chunk_text, distance));
        }

        Ok(results)
    }
}
