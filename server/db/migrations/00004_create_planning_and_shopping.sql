-- ForkFit Database Migration 00004
-- Module: Meal Planning, Pantry Inventory, and Shopping Lists

CREATE TABLE IF NOT EXISTS meal_plans (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name text,
    start_date date NOT NULL,
    end_date date NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS meal_plan_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    meal_plan_id uuid NOT NULL REFERENCES meal_plans(id) ON DELETE CASCADE,
    planned_date date NOT NULL,
    meal_type text NOT NULL, -- 'breakfast', 'lunch', 'dinner', 'snack'
    recipe_id uuid REFERENCES recipes(id) ON DELETE SET NULL,
    food_item_id uuid REFERENCES food_items(id) ON DELETE SET NULL,
    custom_food_name text,
    servings numeric(4,2) NOT NULL DEFAULT 1.00,
    consumed boolean NOT NULL DEFAULT false, -- triggers food log creation if checked
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS pantry_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    food_item_id uuid NOT NULL REFERENCES food_items(id) ON DELETE CASCADE,
    quantity numeric(6,2) NOT NULL,
    unit text NOT NULL,
    expires_at date,
    purchased_at date,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS shopping_lists (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name text NOT NULL,
    status text NOT NULL DEFAULT 'active', -- 'active', 'completed', 'archived'
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS shopping_list_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    shopping_list_id uuid NOT NULL REFERENCES shopping_lists(id) ON DELETE CASCADE,
    food_item_id uuid REFERENCES food_items(id) ON DELETE SET NULL,
    custom_item_name text,
    quantity numeric(6,2) NOT NULL,
    unit text NOT NULL,
    is_acquired boolean NOT NULL DEFAULT false,
    category text DEFAULT 'Other', -- 'Produce', 'Meat', 'Pantry'
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
