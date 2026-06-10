-- ForkFit Database Migration 00007
-- Module: Background Jobs Queue and Notification Logs

CREATE TABLE IF NOT EXISTS background_jobs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    queue_name text NOT NULL DEFAULT 'default',
    job_type text NOT NULL, -- e.g. 'meal_plan_generation'
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    status text NOT NULL DEFAULT 'queued', -- 'queued', 'processing', 'completed', 'failed'
    attempts integer NOT NULL DEFAULT 0,
    max_attempts integer NOT NULL DEFAULT 3,
    run_at timestamptz NOT NULL DEFAULT now(), -- scheduled execution support
    locked_at timestamptz,
    locked_by uuid,
    started_at timestamptz,
    completed_at timestamptz,
    error_log text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS notification_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    channel text NOT NULL, -- 'email', 'sms'
    recipient text NOT NULL, -- destination email/phone
    status text NOT NULL DEFAULT 'queued', -- 'queued', 'sent', 'failed'
    provider text, -- e.g. 'sendgrid'
    provider_message_id text,
    error_message text,
    sent_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
