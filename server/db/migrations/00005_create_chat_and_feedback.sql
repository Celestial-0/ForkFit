-- ForkFit Database Migration 00005
-- Module: Communication, Chat, and User Feedback

CREATE TABLE IF NOT EXISTS chat_threads (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title text,
    agent_type text NOT NULL DEFAULT 'nutritionist', -- 'nutritionist', 'chef'
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    thread_id uuid NOT NULL REFERENCES chat_threads(id) ON DELETE CASCADE,
    sender_role text NOT NULL, -- 'user', 'assistant', 'system'
    content text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb, -- token details, intents
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS user_feedbacks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category text NOT NULL, -- 'chat_response', 'meal_plan', 'recipe'
    reference_id uuid, -- links to messages, recipes, etc.
    rating integer NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);
