-- ForkFit Database Migration 00008
-- Module: Security Audit Logging and Global Performance Indexes

CREATE TABLE IF NOT EXISTS audit_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    session_id uuid REFERENCES sessions(id) ON DELETE SET NULL,
    action text NOT NULL, -- e.g. 'auth.login'
    resource_type text NOT NULL,
    resource_id uuid,
    ip_address inet,
    user_agent text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- ==========================================
-- PERFORMANCE INDEXES
-- ==========================================

-- IAM indices
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(user_id, expires_at) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_oauth_accounts_user_id ON oauth_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_otp_verifications_email ON otp_verifications(email, purpose, created_at DESC);

-- Profiles & Activity logs indices
CREATE INDEX IF NOT EXISTS idx_user_goals_active ON user_goals(user_id) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_biometric_logs_user_date ON biometric_logs(user_id, logged_at DESC);
CREATE INDEX IF NOT EXISTS idx_workout_logs_user_date ON workout_logs(user_id, logged_at DESC);

-- Recipe, Nutrition, and Logging indices
CREATE INDEX IF NOT EXISTS idx_recipes_owner ON recipes(owner_id);
CREATE INDEX IF NOT EXISTS idx_recipe_food_items_food ON recipe_food_items(food_item_id);
CREATE INDEX IF NOT EXISTS idx_food_logs_user_date ON food_logs(user_id, logged_at DESC);

-- Scheduling and Inventory indices
CREATE INDEX IF NOT EXISTS idx_meal_plan_items_date ON meal_plan_items(meal_plan_id, planned_date);
CREATE INDEX IF NOT EXISTS idx_pantry_items_user ON pantry_items(user_id);
CREATE INDEX IF NOT EXISTS idx_shopping_list_items_list ON shopping_list_items(shopping_list_id);

-- Chat and AI Tracing indices
CREATE INDEX IF NOT EXISTS idx_chat_messages_thread ON chat_messages(thread_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ai_execution_steps_trace ON ai_execution_steps(trace_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_date ON audit_logs(actor_user_id, created_at DESC);

-- HNSW Cosine Similarity Vector search indices (pgvector)
CREATE INDEX IF NOT EXISTS idx_recipe_embeddings_vector ON recipe_embeddings USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_agent_memory_embeddings_vector ON agent_memory_embeddings USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS idx_food_item_embeddings_vector ON food_item_embeddings USING hnsw (embedding vector_cosine_ops);

