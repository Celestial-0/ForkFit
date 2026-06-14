-- ForkFit Database Migration 00006
-- Module: AI Orchestration, Tracing, Long-Term Memory, and pgvector RAG

CREATE TABLE IF NOT EXISTS ai_execution_traces (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    session_type text NOT NULL, -- 'chat_response', 'meal_plan_generation'
    session_id uuid, -- links back to triggering entity (e.g. thread_id)
    status text NOT NULL DEFAULT 'running', -- 'running', 'completed', 'failed'
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    total_latency_ms integer,
    total_tokens integer DEFAULT 0,
    total_cost numeric(10, 6) DEFAULT 0.000000
);

CREATE TABLE IF NOT EXISTS ai_execution_steps (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    trace_id uuid NOT NULL REFERENCES ai_execution_traces(id) ON DELETE CASCADE,
    parent_step_id uuid REFERENCES ai_execution_steps(id) ON DELETE CASCADE, -- nested sub-agent graphs
    step_name text NOT NULL, -- e.g. 'Safety Agent check'
    step_type text NOT NULL, -- 'llm_call', 'tool_call', 'retrieval'
    status text NOT NULL DEFAULT 'running',
    input_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    output_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    model_name text,
    prompt_tokens integer,
    completion_tokens integer,
    latency_ms integer,
    error_message text,
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz
);

CREATE TABLE IF NOT EXISTS agent_memories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    memory_type text NOT NULL, -- 'preference', 'restriction', 'habit'
    content text NOT NULL,
    confidence numeric(3,2) NOT NULL DEFAULT 1.00,
    importance integer NOT NULL DEFAULT 5 CHECK (importance >= 1 AND importance <= 10),
    is_active boolean NOT NULL DEFAULT true,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- pgvector RAG Embedding Tables

CREATE TABLE IF NOT EXISTS recipe_embeddings (
    recipe_id uuid PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
    embedding vector(2560) NOT NULL, -- 2560 standard dimensions
    chunk_text text NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS food_item_embeddings (
    food_item_id uuid PRIMARY KEY REFERENCES food_items(id) ON DELETE CASCADE,
    embedding vector(2560) NOT NULL,
    chunk_text text NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS agent_memory_embeddings (
    memory_id uuid PRIMARY KEY REFERENCES agent_memories(id) ON DELETE CASCADE,
    embedding vector(2560) NOT NULL,
    chunk_text text NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);
