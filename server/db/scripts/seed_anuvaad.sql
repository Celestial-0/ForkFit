-- ForkFit Database Seed Ingestion Script
-- Module: Pure SQL Ingestion for Anuvaad INDB 2024.11 CSV
--
-- How to run this script:
-- 1. Ensure you are in the directory containing this script:
--    cd server/db/scripts
-- 2. Run the script using psql (uses default relative path to seeds/Anuvaad_INDB_2024.11.csv):
--    psql "postgres://yash:yash1234@localhost:5432/forkfit" -f seed_anuvaad.sql
--
-- Alternatively, specify the CSV file path dynamically from any directory:
--    psql "postgres://yash:yash1234@localhost:5432/forkfit" -v csv_file='server/db/seeds/Anuvaad_INDB_2024.11.csv' -f server/db/scripts/seed_anuvaad.sql
--
-- Alternatively, if running inside a Docker setup (e.g., postgres container):
--    docker cp server/db postgres:/tmp/db
--    docker exec -i postgres psql -U yash -d forkfit -f /tmp/db/scripts/seed_anuvaad.sql

-- 1. Setup default file path if not set
\if :{?csv_file}
\else
  \set csv_file '../seeds/Anuvaad_INDB_2024.11.csv'
\endif

\echo 'Starting SQL ingestion of Anuvaad INDB from: ' :csv_file

BEGIN;

-- 2. Create Temporary Staging Table matching CSV columns exactly
CREATE TEMP TABLE temp_csv_nutrition (
    food_code text,
    food_name text,
    primarysource text,
    energy_kj numeric,
    energy_kcal numeric,
    carb_g numeric,
    protein_g numeric,
    fat_g numeric,
    freesugar_g numeric,
    fibre_g numeric,
    sfa_mg numeric,
    mufa_mg numeric,
    pufa_mg numeric,
    cholesterol_mg numeric,
    calcium_mg numeric,
    phosphorus_mg numeric,
    magnesium_mg numeric,
    sodium_mg numeric,
    potassium_mg numeric,
    iron_mg numeric,
    copper_mg numeric,
    selenium_ug numeric,
    chromium_mg numeric,
    manganese_mg numeric,
    molybdenum_mg numeric,
    zinc_mg numeric,
    vita_ug numeric,
    vite_mg numeric,
    vitd2_ug numeric,
    vitd3_ug numeric,
    vitk1_ug numeric,
    vitk2_ug numeric,
    folate_ug numeric,
    vitb1_mg numeric,
    vitb2_mg numeric,
    vitb3_mg numeric,
    vitb5_mg numeric,
    vitb6_mg numeric,
    vitb7_ug numeric,
    vitb9_ug numeric,
    vitc_mg numeric,
    carotenoids_ug numeric,
    servings_unit text,
    unit_serving_energy_kj numeric,
    unit_serving_energy_kcal numeric,
    unit_serving_carb_g numeric,
    unit_serving_protein_g numeric,
    unit_serving_fat_g numeric,
    unit_serving_freesugar_g numeric,
    unit_serving_fibre_g numeric,
    unit_serving_sfa_mg numeric,
    unit_serving_mufa_mg numeric,
    unit_serving_pufa_mg numeric,
    unit_serving_cholesterol_mg numeric,
    unit_serving_calcium_mg numeric,
    unit_serving_phosphorus_mg numeric,
    unit_serving_magnesium_mg numeric,
    unit_serving_sodium_mg numeric,
    unit_serving_potassium_mg numeric,
    unit_serving_iron_mg numeric,
    unit_serving_copper_mg numeric,
    unit_serving_selenium_ug numeric,
    unit_serving_chromium_mg numeric,
    unit_serving_manganese_mg numeric,
    unit_serving_molybdenum_mg numeric,
    unit_serving_zinc_mg numeric,
    unit_serving_vita_ug numeric,
    unit_serving_vite_mg numeric,
    unit_serving_vitd2_ug numeric,
    unit_serving_vitd3_ug numeric,
    unit_serving_vitk1_ug numeric,
    unit_serving_vitk2_ug numeric,
    unit_serving_folate_ug numeric,
    unit_serving_vitb1_mg numeric,
    unit_serving_vitb2_mg numeric,
    unit_serving_vitb3_mg numeric,
    unit_serving_vitb5_mg numeric,
    unit_serving_vitb6_mg numeric,
    unit_serving_vitb7_ug numeric,
    unit_serving_vitb9_ug numeric,
    unit_serving_vitc_mg numeric,
    unit_serving_carotenoids_ug numeric
);

-- 3. Load raw data into staging table
COPY temp_csv_nutrition FROM '/tmp/db/seeds/Anuvaad_INDB_2024.11.csv' WITH (FORMAT csv, HEADER true);

-- 4. Update Existing Ingredients (conflict match on food_code)
\echo 'Updating existing ingredients by food_code...'
UPDATE ingredients i
SET 
    name = t.food_name,
    calories_per_100g = COALESCE(t.energy_kcal, 0.0),
    protein_per_100g = COALESCE(t.protein_g, 0.0),
    carbs_per_100g = COALESCE(t.carb_g, 0.0),
    fat_per_100g = COALESCE(t.fat_g, 0.0),
    fiber_per_100g = COALESCE(t.fibre_g, 0.0),
    sodium_mg_per_100g = COALESCE(t.sodium_mg, 0.0),
    primary_source = COALESCE(t.primarysource, i.primary_source),
    updated_at = now(),
    -- Build micronutrient JSONB dynamically, excluding 0.0 or null entries to optimize space
    micronutrients = jsonb_strip_nulls(jsonb_build_object(
        'freesugar_g', CASE WHEN t.freesugar_g > 0 THEN t.freesugar_g END,
        'sfa_mg', CASE WHEN t.sfa_mg > 0 THEN t.sfa_mg END,
        'mufa_mg', CASE WHEN t.mufa_mg > 0 THEN t.mufa_mg END,
        'pufa_mg', CASE WHEN t.pufa_mg > 0 THEN t.pufa_mg END,
        'cholesterol_mg', CASE WHEN t.cholesterol_mg > 0 THEN t.cholesterol_mg END,
        'calcium_mg', CASE WHEN t.calcium_mg > 0 THEN t.calcium_mg END,
        'phosphorus_mg', CASE WHEN t.phosphorus_mg > 0 THEN t.phosphorus_mg END,
        'magnesium_mg', CASE WHEN t.magnesium_mg > 0 THEN t.magnesium_mg END,
        'potassium_mg', CASE WHEN t.potassium_mg > 0 THEN t.potassium_mg END,
        'iron_mg', CASE WHEN t.iron_mg > 0 THEN t.iron_mg END,
        'copper_mg', CASE WHEN t.copper_mg > 0 THEN t.copper_mg END,
        'selenium_ug', CASE WHEN t.selenium_ug > 0 THEN t.selenium_ug END,
        'chromium_mg', CASE WHEN t.chromium_mg > 0 THEN t.chromium_mg END,
        'manganese_mg', CASE WHEN t.manganese_mg > 0 THEN t.manganese_mg END,
        'molybdenum_mg', CASE WHEN t.molybdenum_mg > 0 THEN t.molybdenum_mg END,
        'zinc_mg', CASE WHEN t.zinc_mg > 0 THEN t.zinc_mg END,
        'vita_ug', CASE WHEN t.vita_ug > 0 THEN t.vita_ug END,
        'vite_mg', CASE WHEN t.vite_mg > 0 THEN t.vite_mg END,
        'vitd2_ug', CASE WHEN t.vitd2_ug > 0 THEN t.vitd2_ug END,
        'vitd3_ug', CASE WHEN t.vitd3_ug > 0 THEN t.vitd3_ug END,
        'vitk1_ug', CASE WHEN t.vitk1_ug > 0 THEN t.vitk1_ug END,
        'vitk2_ug', CASE WHEN t.vitk2_ug > 0 THEN t.vitk2_ug END,
        'folate_ug', CASE WHEN t.folate_ug > 0 THEN t.folate_ug END,
        'vitb1_mg', CASE WHEN t.vitb1_mg > 0 THEN t.vitb1_mg END,
        'vitb2_mg', CASE WHEN t.vitb2_mg > 0 THEN t.vitb2_mg END,
        'vitb3_mg', CASE WHEN t.vitb3_mg > 0 THEN t.vitb3_mg END,
        'vitb5_mg', CASE WHEN t.vitb5_mg > 0 THEN t.vitb5_mg END,
        'vitb6_mg', CASE WHEN t.vitb6_mg > 0 THEN t.vitb6_mg END,
        'vitb7_ug', CASE WHEN t.vitb7_ug > 0 THEN t.vitb7_ug END,
        'vitb9_ug', CASE WHEN t.vitb9_ug > 0 THEN t.vitb9_ug END,
        'vitc_mg', CASE WHEN t.vitc_mg > 0 THEN t.vitc_mg END,
        'carotenoids_ug', CASE WHEN t.carotenoids_ug > 0 THEN t.carotenoids_ug END
    ))
FROM temp_csv_nutrition t
WHERE i.food_code = t.food_code;

-- 5. Insert New Ingredients (ignoring duplicates by food_code or name)
\echo 'Inserting new ingredients...'
INSERT INTO ingredients (
    id, name, calories_per_100g, protein_per_100g, carbs_per_100g, 
    fat_per_100g, fiber_per_100g, sodium_mg_per_100g, micronutrients, 
    food_code, primary_source, estimated_cost_per_100g
)
SELECT 
    gen_random_uuid(),
    t.food_name,
    COALESCE(t.energy_kcal, 0.0),
    COALESCE(t.protein_g, 0.0),
    COALESCE(t.carb_g, 0.0),
    COALESCE(t.fat_g, 0.0),
    COALESCE(t.fibre_g, 0.0),
    COALESCE(t.sodium_mg, 0.0),
    -- Build micronutrient JSONB dynamically
    jsonb_strip_nulls(jsonb_build_object(
        'freesugar_g', CASE WHEN t.freesugar_g > 0 THEN t.freesugar_g END,
        'sfa_mg', CASE WHEN t.sfa_mg > 0 THEN t.sfa_mg END,
        'mufa_mg', CASE WHEN t.mufa_mg > 0 THEN t.mufa_mg END,
        'pufa_mg', CASE WHEN t.pufa_mg > 0 THEN t.pufa_mg END,
        'cholesterol_mg', CASE WHEN t.cholesterol_mg > 0 THEN t.cholesterol_mg END,
        'calcium_mg', CASE WHEN t.calcium_mg > 0 THEN t.calcium_mg END,
        'phosphorus_mg', CASE WHEN t.phosphorus_mg > 0 THEN t.phosphorus_mg END,
        'magnesium_mg', CASE WHEN t.magnesium_mg > 0 THEN t.magnesium_mg END,
        'potassium_mg', CASE WHEN t.potassium_mg > 0 THEN t.potassium_mg END,
        'iron_mg', CASE WHEN t.iron_mg > 0 THEN t.iron_mg END,
        'copper_mg', CASE WHEN t.copper_mg > 0 THEN t.copper_mg END,
        'selenium_ug', CASE WHEN t.selenium_ug > 0 THEN t.selenium_ug END,
        'chromium_mg', CASE WHEN t.chromium_mg > 0 THEN t.chromium_mg END,
        'manganese_mg', CASE WHEN t.manganese_mg > 0 THEN t.manganese_mg END,
        'molybdenum_mg', CASE WHEN t.molybdenum_mg > 0 THEN t.molybdenum_mg END,
        'zinc_mg', CASE WHEN t.zinc_mg > 0 THEN t.zinc_mg END,
        'vita_ug', CASE WHEN t.vita_ug > 0 THEN t.vita_ug END,
        'vite_mg', CASE WHEN t.vite_mg > 0 THEN t.vite_mg END,
        'vitd2_ug', CASE WHEN t.vitd2_ug > 0 THEN t.vitd2_ug END,
        'vitd3_ug', CASE WHEN t.vitd3_ug > 0 THEN t.vitd3_ug END,
        'vitk1_ug', CASE WHEN t.vitk1_ug > 0 THEN t.vitk1_ug END,
        'vitk2_ug', CASE WHEN t.vitk2_ug > 0 THEN t.vitk2_ug END,
        'folate_ug', CASE WHEN t.folate_ug > 0 THEN t.folate_ug END,
        'vitb1_mg', CASE WHEN t.vitb1_mg > 0 THEN t.vitb1_mg END,
        'vitb2_mg', CASE WHEN t.vitb2_mg > 0 THEN t.vitb2_mg END,
        'vitb3_mg', CASE WHEN t.vitb3_mg > 0 THEN t.vitb3_mg END,
        'vitb5_mg', CASE WHEN t.vitb5_mg > 0 THEN t.vitb5_mg END,
        'vitb6_mg', CASE WHEN t.vitb6_mg > 0 THEN t.vitb6_mg END,
        'vitb7_ug', CASE WHEN t.vitb7_ug > 0 THEN t.vitb7_ug END,
        'vitb9_ug', CASE WHEN t.vitb9_ug > 0 THEN t.vitb9_ug END,
        'vitc_mg', CASE WHEN t.vitc_mg > 0 THEN t.vitc_mg END,
        'carotenoids_ug', CASE WHEN t.carotenoids_ug > 0 THEN t.carotenoids_ug END
    )),
    t.food_code,
    t.primarysource,
    0.00
FROM temp_csv_nutrition t
WHERE NOT EXISTS (
    SELECT 1 FROM ingredients i2 
    WHERE i2.food_code = t.food_code OR i2.name = t.food_name
);

-- 6. Insert/Upsert Portion Mappings
\echo 'Inserting portion configurations...'
INSERT INTO ingredient_portions (ingredient_id, serving_unit, grams_equivalent)
SELECT 
    i.id,
    TRIM(t.servings_unit),
    -- Calculate equivalent serving weight in grams from calorie or macro ratios
    COALESCE(
      CASE WHEN t.energy_kcal > 0 AND t.unit_serving_energy_kcal > 0 THEN ROUND((t.unit_serving_energy_kcal / t.energy_kcal) * 100, 2) END,
      CASE WHEN t.carb_g > 0 AND t.unit_serving_carb_g > 0 THEN ROUND((t.unit_serving_carb_g / t.carb_g) * 100, 2) END,
      CASE WHEN t.protein_g > 0 AND t.unit_serving_protein_g > 0 THEN ROUND((t.unit_serving_protein_g / t.protein_g) * 100, 2) END,
      CASE WHEN t.fat_g > 0 AND t.unit_serving_fat_g > 0 THEN ROUND((t.unit_serving_fat_g / t.fat_g) * 100, 2) END,
      100.0
    )
FROM temp_csv_nutrition t
JOIN ingredients i ON (i.food_code = t.food_code OR i.name = t.food_name)
WHERE NULLIF(TRIM(t.servings_unit), '') IS NOT NULL
ON CONFLICT (ingredient_id, serving_unit) DO UPDATE SET
    grams_equivalent = EXCLUDED.grams_equivalent,
    updated_at = now();

COMMIT;

\echo 'Ingestion of Anuvaad INDB completed successfully.'
