-- ForkFit Database Migration 00003
-- Module: Food Items, Recipes, and Food Logging

CREATE TABLE IF NOT EXISTS raw_food_costs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    food_pattern text NOT NULL UNIQUE,
    cost_per_100g numeric(6,2) NOT NULL DEFAULT 0.00,
    price_currency text NOT NULL DEFAULT 'INR',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS food_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL UNIQUE,
    description text,
    calories_per_100g numeric(6,2) NOT NULL,
    protein_per_100g numeric(5,2) NOT NULL,
    carbs_per_100g numeric(5,2) NOT NULL,
    fat_per_100g numeric(5,2) NOT NULL,
    fiber_per_100g numeric(5,2) NOT NULL DEFAULT 0.00,
    sodium_mg_per_100g numeric(10,2) NOT NULL DEFAULT 0.00,
    micronutrients jsonb NOT NULL DEFAULT '{}'::jsonb,
    barcode text,
    is_verified boolean NOT NULL DEFAULT false,
    food_code text UNIQUE,
    primary_source text,
    raw_food_cost_id uuid REFERENCES raw_food_costs(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS food_item_portions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    food_item_id uuid NOT NULL REFERENCES food_items(id) ON DELETE CASCADE,
    serving_unit text NOT NULL, -- e.g., 'slice', 'cup', 'piece'
    grams_equivalent numeric(6,2) NOT NULL CHECK (grams_equivalent > 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(food_item_id, serving_unit)
);

CREATE TABLE IF NOT EXISTS recipes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id uuid REFERENCES users(id) ON DELETE SET NULL, -- NULL means public system recipe
    parent_recipe_id uuid REFERENCES recipes(id) ON DELETE SET NULL, -- For Recipe Agent variations/alternatives
    title text NOT NULL,
    description text,
    instructions text[] NOT NULL,
    prep_time_minutes integer,
    cook_time_minutes integer,
    servings numeric(4,2) NOT NULL DEFAULT 1.00,
    cuisine text, -- Required for Culture Agent regional alignment
    course text,
    dietary_tags text[] NOT NULL DEFAULT '{}', -- E.g. {'Halal', 'Kosher'}
    source_url text,
    is_public boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recipe_food_items (
    recipe_id uuid NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    food_item_id uuid NOT NULL REFERENCES food_items(id) ON DELETE RESTRICT,
    quantity numeric(6,2) NOT NULL,
    unit text NOT NULL, -- 'g', 'ml', 'piece'
    grams_equivalent numeric(6,2) NOT NULL, -- Used to compute macro values
    notes text,
    PRIMARY KEY (recipe_id, food_item_id)
);

CREATE TABLE IF NOT EXISTS food_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    logged_at timestamptz NOT NULL DEFAULT now(),
    meal_type text NOT NULL, -- 'breakfast', 'lunch', 'dinner', 'snack'
    recipe_id uuid REFERENCES recipes(id) ON DELETE SET NULL,
    food_item_id uuid REFERENCES food_items(id) ON DELETE SET NULL,
    custom_food_name text,
    quantity numeric(6,2) NOT NULL,
    unit text NOT NULL, -- 'servings', 'grams'
    calories numeric(6,2) NOT NULL, -- Denormalized nutrient stats for historical freeze
    protein numeric(5,2) NOT NULL,
    carbs numeric(5,2) NOT NULL,
    fats numeric(5,2) NOT NULL,
    micronutrients_snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);
