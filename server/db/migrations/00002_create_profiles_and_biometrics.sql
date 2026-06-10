-- ForkFit Database Migration 00002
-- Module: Profiles, Settings, Safety, Biometrics, and Goals

CREATE TABLE IF NOT EXISTS profiles (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    full_name text,
    avatar_url text,
    gender text, -- 'male', 'female'
    dob date,
    timezone text DEFAULT 'UTC',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS user_preferences (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme text NOT NULL DEFAULT 'system',
    language text NOT NULL DEFAULT 'en',
    measurement_system text NOT NULL DEFAULT 'metric', -- 'metric', 'imperial'
    preferences jsonb NOT NULL DEFAULT '{}'::jsonb, -- dynamic settings
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS medical_safety_profiles (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    allergies text[] NOT NULL DEFAULT '{}', -- e.g. {'peanuts', 'shellfish'}
    medical_conditions text[] NOT NULL DEFAULT '{}', -- e.g. {'diabetes', 'hypertension'}
    is_pregnant boolean NOT NULL DEFAULT false,
    is_lactating boolean NOT NULL DEFAULT false,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS biometric_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    logged_at timestamptz NOT NULL DEFAULT now(),
    metric_type text NOT NULL, -- 'weight_kg', 'height_cm', 'body_fat_pct'
    value numeric(5,2) NOT NULL,
    notes text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS workout_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    logged_at timestamptz NOT NULL DEFAULT now(),
    activity_name text NOT NULL, -- e.g. 'Weightlifting', 'Running'
    duration_minutes integer NOT NULL,
    calories_burned numeric(6,2) NOT NULL DEFAULT 0.00,
    notes text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS user_goals (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category text NOT NULL, -- 'nutrition', 'weight', 'activity'
    target_type text NOT NULL, -- 'weight_loss', 'calorie_surplus', 'macro_maintenance', 'daily_steps'
    target_value numeric(8,2) NOT NULL,
    unit text NOT NULL, -- 'kg', 'kcal', 'steps'
    config jsonb NOT NULL DEFAULT '{}'::jsonb, -- e.g. {"protein_g": 150, "carbs_g": 200, "fats_g": 60}
    start_date date NOT NULL DEFAULT CURRENT_DATE,
    target_date date,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
