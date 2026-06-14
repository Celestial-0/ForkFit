use sqlx::{PgPool, query, query_scalar};
use uuid::Uuid;

use crate::common::AppResult;
use crate::common::id::{UserId, RecipeId, FoodItemId, FoodLogId, FoodItemPortionId, RawFoodCostId};
use crate::recipe::models::{
    FoodItem, Recipe, RecipeFoodItem, RecipeFoodItemDetail, RecipeWithFoodItems, FoodLog, FoodItemPortion, RawFoodCost,
};
use crate::recipe::repository::RecipeRepository;

#[derive(Clone)]
pub struct PgRecipeRepository {
    pool: PgPool,
}

impl PgRecipeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RecipeRepository for PgRecipeRepository {
    async fn create_food_item(&self, food: FoodItem) -> AppResult<FoodItem> {
        let raw_id: Uuid = food.id.into();
        let raw_cost_id: Option<Uuid> = food.raw_food_cost_id.map(|id| id.into());
        let inserted_id = query_scalar!(
            r#"
            INSERT INTO food_items (id, name, description, calories_per_100g, protein_per_100g, carbs_per_100g, fat_per_100g, fiber_per_100g, sodium_mg_per_100g, micronutrients, barcode, is_verified, food_code, primary_source, raw_food_cost_id)
            VALUES ($1, $2, $3, $4::float8, $5::float8, $6::float8, $7::float8, $8::float8, $9::float8, $10, $11, $12, $13, $14, $15)
            RETURNING id as "id: FoodItemId"
            "#,
            raw_id,
            food.name,
            food.description,
            food.calories_per_100g,
            food.protein_per_100g,
            food.carbs_per_100g,
            food.fat_per_100g,
            food.fiber_per_100g,
            food.sodium_mg_per_100g,
            food.micronutrients,
            food.barcode,
            food.is_verified,
            food.food_code,
            food.primary_source,
            raw_cost_id
        )
        .fetch_one(&self.pool)
        .await?;

        let full_food = self.get_food_item(inserted_id).await?;
        full_food.ok_or(sqlx::Error::RowNotFound.into())
    }

    async fn get_food_item(&self, id: FoodItemId) -> AppResult<Option<FoodItem>> {
        let raw_id: Uuid = id.into();
        let res = query!(
            r#"
            SELECT
                i.id as "id!: FoodItemId",
                i.name as "name!",
                i.description,
                i.calories_per_100g::float8 as "calories_per_100g!",
                i.protein_per_100g::float8 as "protein_per_100g!",
                i.carbs_per_100g::float8 as "carbs_per_100g!",
                i.fat_per_100g::float8 as "fat_per_100g!",
                i.fiber_per_100g::float8 as "fiber_per_100g!",
                i.sodium_mg_per_100g::float8 as "sodium_mg_per_100g!",
                i.micronutrients as "micronutrients!",
                COALESCE(fc.cost_per_100g, 0.00)::float8 as "estimated_cost_per_100g!",
                COALESCE(fc.price_currency, 'INR') as "price_currency!",
                i.barcode,
                i.is_verified as "is_verified!",
                i.food_code,
                i.primary_source,
                i.raw_food_cost_id as "raw_food_cost_id: RawFoodCostId",
                i.created_at as "created_at!",
                i.updated_at as "updated_at!"
            FROM food_items i
            LEFT JOIN raw_food_costs fc ON i.raw_food_cost_id = fc.id
            WHERE i.id = $1
            "#,
            raw_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(|r| FoodItem {
            id: r.id,
            name: r.name,
            description: r.description,
            calories_per_100g: r.calories_per_100g,
            protein_per_100g: r.protein_per_100g,
            carbs_per_100g: r.carbs_per_100g,
            fat_per_100g: r.fat_per_100g,
            fiber_per_100g: r.fiber_per_100g,
            sodium_mg_per_100g: r.sodium_mg_per_100g,
            micronutrients: r.micronutrients,
            estimated_cost_per_100g: r.estimated_cost_per_100g,
            price_currency: r.price_currency,
            barcode: r.barcode,
            is_verified: r.is_verified,
            food_code: r.food_code,
            primary_source: r.primary_source,
            raw_food_cost_id: r.raw_food_cost_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn search_food_items(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<FoodItem>, u64)> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;
        let like_query = format!("%{}%", query);

        let total = query_scalar!(
            r#"SELECT count(*) FROM food_items WHERE name ILIKE $1"#,
            like_query
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = query!(
            r#"
            SELECT
                i.id as "id!: FoodItemId",
                i.name as "name!",
                i.description,
                i.calories_per_100g::float8 as "calories_per_100g!",
                i.protein_per_100g::float8 as "protein_per_100g!",
                i.carbs_per_100g::float8 as "carbs_per_100g!",
                i.fat_per_100g::float8 as "fat_per_100g!",
                i.fiber_per_100g::float8 as "fiber_per_100g!",
                i.sodium_mg_per_100g::float8 as "sodium_mg_per_100g!",
                i.micronutrients as "micronutrients!",
                COALESCE(fc.cost_per_100g, 0.00)::float8 as "estimated_cost_per_100g!",
                COALESCE(fc.price_currency, 'INR') as "price_currency!",
                i.barcode,
                i.is_verified as "is_verified!",
                i.food_code,
                i.primary_source,
                i.raw_food_cost_id as "raw_food_cost_id: RawFoodCostId",
                i.created_at as "created_at!",
                i.updated_at as "updated_at!"
            FROM food_items i
            LEFT JOIN raw_food_costs fc ON i.raw_food_cost_id = fc.id
            WHERE i.name ILIKE $1
            ORDER BY i.name ASC
            LIMIT $2 OFFSET $3
            "#,
            like_query,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let food_items = rows
            .into_iter()
            .map(|r| FoodItem {
                id: r.id,
                name: r.name,
                description: r.description,
                calories_per_100g: r.calories_per_100g,
                protein_per_100g: r.protein_per_100g,
                carbs_per_100g: r.carbs_per_100g,
                fat_per_100g: r.fat_per_100g,
                fiber_per_100g: r.fiber_per_100g,
                sodium_mg_per_100g: r.sodium_mg_per_100g,
                micronutrients: r.micronutrients,
                estimated_cost_per_100g: r.estimated_cost_per_100g,
                price_currency: r.price_currency,
                barcode: r.barcode,
                is_verified: r.is_verified,
                food_code: r.food_code,
                primary_source: r.primary_source,
                raw_food_cost_id: r.raw_food_cost_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok((food_items, total))
    }

    async fn add_food_item_portion(&self, portion: FoodItemPortion) -> AppResult<FoodItemPortion> {
        let raw_id: Uuid = portion.id.into();
        let raw_food_item_id: Uuid = portion.food_item_id.into();
        let res = query!(
            r#"
            INSERT INTO food_item_portions (id, food_item_id, serving_unit, grams_equivalent)
            VALUES ($1, $2, $3, $4::float8)
            RETURNING
                id as "id: FoodItemPortionId",
                food_item_id as "food_item_id: FoodItemId",
                serving_unit,
                grams_equivalent::float8 as "grams_equivalent!",
                created_at,
                updated_at
            "#,
            raw_id,
            raw_food_item_id,
            portion.serving_unit,
            portion.grams_equivalent
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(FoodItemPortion {
            id: res.id,
            food_item_id: res.food_item_id,
            serving_unit: res.serving_unit,
            grams_equivalent: res.grams_equivalent,
            created_at: res.created_at,
            updated_at: res.updated_at,
        })
    }

    async fn get_food_item_portions(&self, food_item_id: FoodItemId) -> AppResult<Vec<FoodItemPortion>> {
        let raw_food_item_id: Uuid = food_item_id.into();
        let rows = query!(
            r#"
            SELECT
                id as "id: FoodItemPortionId",
                food_item_id as "food_item_id: FoodItemId",
                serving_unit,
                grams_equivalent::float8 as "grams_equivalent!",
                created_at,
                updated_at
            FROM food_item_portions
            WHERE food_item_id = $1
            ORDER BY serving_unit ASC
            "#,
            raw_food_item_id
        )
        .fetch_all(&self.pool)
        .await?;

        let portions = rows
            .into_iter()
            .map(|r| FoodItemPortion {
                id: r.id,
                food_item_id: r.food_item_id,
                serving_unit: r.serving_unit,
                grams_equivalent: r.grams_equivalent,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(portions)
    }

    async fn create_recipe(&self, recipe: Recipe, food_items: Vec<RecipeFoodItem>) -> AppResult<RecipeWithFoodItems> {
        let mut tx = self.pool.begin().await?;
        let raw_recipe_id: Uuid = recipe.id.into();
        
        // 1. Insert Recipe
        query!(
            r#"
            INSERT INTO recipes (id, owner_id, parent_recipe_id, title, description, instructions, prep_time_minutes, cook_time_minutes, servings, cuisine, course, dietary_tags, source_url, is_public)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::float8, $10, $11, $12, $13, $14)
            "#,
            raw_recipe_id,
            recipe.owner_id as Option<UserId>,
            recipe.parent_recipe_id as Option<RecipeId>,
            recipe.title,
            recipe.description,
            &recipe.instructions,
            recipe.prep_time_minutes,
            recipe.cook_time_minutes,
            recipe.servings,
            recipe.cuisine,
            recipe.course,
            &recipe.dietary_tags,
            recipe.source_url,
            recipe.is_public
        )
        .execute(&mut *tx)
        .await?;

        // 2. Insert Recipe Food Items
        for rf in food_items {
            query!(
                r#"
                INSERT INTO recipe_food_items (recipe_id, food_item_id, quantity, unit, grams_equivalent, notes)
                VALUES ($1, $2, $3::float8, $4, $5::float8, $6)
                "#,
                raw_recipe_id,
                rf.food_item_id as FoodItemId,
                rf.quantity,
                rf.unit,
                rf.grams_equivalent,
                rf.notes
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // 3. Fetch composite detail
        let details = self.get_recipe(recipe.id).await?;
        details.ok_or(sqlx::Error::RowNotFound.into())
    }

    async fn get_recipe(&self, id: RecipeId) -> AppResult<Option<RecipeWithFoodItems>> {
        let raw_id: Uuid = id.into();
        let recipe_row = query!(
            r#"
            SELECT
                id as "id: RecipeId",
                owner_id as "owner_id: UserId",
                parent_recipe_id as "parent_recipe_id: RecipeId",
                title,
                description,
                instructions,
                prep_time_minutes,
                cook_time_minutes,
                servings::float8 as "servings!",
                cuisine,
                course,
                dietary_tags,
                source_url,
                is_public,
                created_at,
                updated_at
            FROM recipes
            WHERE id = $1
            "#,
            raw_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let recipe = match recipe_row {
            Some(r) => Recipe {
                id: r.id,
                owner_id: r.owner_id,
                parent_recipe_id: r.parent_recipe_id,
                title: r.title,
                description: r.description,
                instructions: r.instructions,
                prep_time_minutes: r.prep_time_minutes,
                cook_time_minutes: r.cook_time_minutes,
                servings: r.servings,
                cuisine: r.cuisine,
                course: r.course,
                dietary_tags: r.dietary_tags,
                source_url: r.source_url,
                is_public: r.is_public,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            None => return Ok(None),
        };

        let food_item_rows = query!(
            r#"
            SELECT
                ri.food_item_id as "food_item_id: FoodItemId",
                i.name,
                ri.quantity::float8 as "quantity!",
                ri.unit,
                ri.grams_equivalent::float8 as "grams_equivalent!",
                i.calories_per_100g::float8 as "calories_per_100g!",
                i.protein_per_100g::float8 as "protein_per_100g!",
                i.carbs_per_100g::float8 as "carbs_per_100g!",
                i.fat_per_100g::float8 as "fat_per_100g!",
                i.fiber_per_100g::float8 as "fiber_per_100g!",
                i.sodium_mg_per_100g::float8 as "sodium_mg_per_100g!",
                COALESCE(fc.cost_per_100g, 0.00)::float8 as "estimated_cost_per_100g!",
                ri.notes
            FROM recipe_food_items ri
            JOIN food_items i ON ri.food_item_id = i.id
            LEFT JOIN raw_food_costs fc ON i.raw_food_cost_id = fc.id
            WHERE ri.recipe_id = $1
            "#,
            raw_id
        )
        .fetch_all(&self.pool)
        .await?;

        let food_items = food_item_rows
            .into_iter()
            .map(|r| RecipeFoodItemDetail {
                food_item_id: r.food_item_id,
                name: r.name,
                quantity: r.quantity,
                unit: r.unit,
                grams_equivalent: r.grams_equivalent,
                calories_per_100g: r.calories_per_100g,
                protein_per_100g: r.protein_per_100g,
                carbs_per_100g: r.carbs_per_100g,
                fat_per_100g: r.fat_per_100g,
                fiber_per_100g: r.fiber_per_100g,
                sodium_mg_per_100g: r.sodium_mg_per_100g,
                estimated_cost_per_100g: r.estimated_cost_per_100g,
                notes: r.notes,
            })
            .collect();

        Ok(Some(RecipeWithFoodItems { recipe, food_items }))
    }

    async fn list_recipes(&self, owner_id: Option<UserId>, page: u64, per_page: u64) -> AppResult<(Vec<Recipe>, u64)> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let (total, recipes) = if let Some(owner) = owner_id {
            let total = query_scalar!(
                r#"SELECT count(*) FROM recipes WHERE owner_id = $1 OR is_public = true"#,
                owner as UserId
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0) as u64;

            let rows = query!(
                r#"
                SELECT
                    id as "id: RecipeId",
                    owner_id as "owner_id: UserId",
                    parent_recipe_id as "parent_recipe_id: RecipeId",
                    title,
                    description,
                    instructions,
                    prep_time_minutes,
                    cook_time_minutes,
                    servings::float8 as "servings!",
                    cuisine,
                    course,
                    dietary_tags,
                    source_url,
                    is_public,
                    created_at,
                    updated_at
                FROM recipes
                WHERE owner_id = $1 OR is_public = true
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                owner as UserId,
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await?;

            let recs = rows
                .into_iter()
                .map(|r| Recipe {
                    id: r.id,
                    owner_id: r.owner_id,
                    parent_recipe_id: r.parent_recipe_id,
                    title: r.title,
                    description: r.description,
                    instructions: r.instructions,
                    prep_time_minutes: r.prep_time_minutes,
                    cook_time_minutes: r.cook_time_minutes,
                    servings: r.servings,
                    cuisine: r.cuisine,
                    course: r.course,
                    dietary_tags: r.dietary_tags,
                    source_url: r.source_url,
                    is_public: r.is_public,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
                .collect();

            (total, recs)
        } else {
            let total = query_scalar!(
                r#"SELECT count(*) FROM recipes WHERE is_public = true"#
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0) as u64;

            let rows = query!(
                r#"
                SELECT
                    id as "id: RecipeId",
                    owner_id as "owner_id: UserId",
                    parent_recipe_id as "parent_recipe_id: RecipeId",
                    title,
                    description,
                    instructions,
                    prep_time_minutes,
                    cook_time_minutes,
                    servings::float8 as "servings!",
                    cuisine,
                    course,
                    dietary_tags,
                    source_url,
                    is_public,
                    created_at,
                    updated_at
                FROM recipes
                WHERE is_public = true
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await?;

            let recs = rows
                .into_iter()
                .map(|r| Recipe {
                    id: r.id,
                    owner_id: r.owner_id,
                    parent_recipe_id: r.parent_recipe_id,
                    title: r.title,
                    description: r.description,
                    instructions: r.instructions,
                    prep_time_minutes: r.prep_time_minutes,
                    cook_time_minutes: r.cook_time_minutes,
                    servings: r.servings,
                    cuisine: r.cuisine,
                    course: r.course,
                    dietary_tags: r.dietary_tags,
                    source_url: r.source_url,
                    is_public: r.is_public,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                })
                .collect();

            (total, recs)
        };

        Ok((recipes, total))
    }

    async fn get_recipes_by_ids(&self, ids: &[RecipeId]) -> AppResult<Vec<Recipe>> {
        let raw_ids: Vec<Uuid> = ids.iter().map(|id| id.0).collect();
        let rows = query!(
            r#"
            SELECT
                id as "id: RecipeId",
                owner_id as "owner_id: UserId",
                parent_recipe_id as "parent_recipe_id: RecipeId",
                title,
                description,
                instructions,
                prep_time_minutes,
                cook_time_minutes,
                servings::float8 as "servings!",
                cuisine,
                course,
                dietary_tags,
                source_url,
                is_public,
                created_at,
                updated_at
            FROM recipes
            WHERE id = Any($1)
            ORDER BY created_at DESC
            "#,
            &raw_ids
        )
        .fetch_all(&self.pool)
        .await?;

        let recipes = rows
            .into_iter()
            .map(|r| Recipe {
                id: r.id,
                owner_id: r.owner_id,
                parent_recipe_id: r.parent_recipe_id,
                title: r.title,
                description: r.description,
                instructions: r.instructions,
                prep_time_minutes: r.prep_time_minutes,
                cook_time_minutes: r.cook_time_minutes,
                servings: r.servings,
                cuisine: r.cuisine,
                course: r.course,
                dietary_tags: r.dietary_tags,
                source_url: r.source_url,
                is_public: r.is_public,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(recipes)
    }

    async fn log_food(&self, log: FoodLog) -> AppResult<FoodLog> {
        let raw_id: Uuid = log.id.into();
        let res = query!(
            r#"
            INSERT INTO food_logs (id, user_id, logged_at, meal_type, recipe_id, food_item_id, custom_food_name, quantity, unit, calories, protein, carbs, fats, micronutrients_snapshot)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::float8, $9, $10::float8, $11::float8, $12::float8, $13::float8, $14)
            RETURNING
                id as "id: FoodLogId",
                user_id as "user_id: UserId",
                logged_at,
                meal_type,
                recipe_id as "recipe_id: RecipeId",
                food_item_id as "food_item_id: FoodItemId",
                custom_food_name,
                quantity::float8 as "quantity!",
                unit,
                calories::float8 as "calories!",
                protein::float8 as "protein!",
                carbs::float8 as "carbs!",
                fats::float8 as "fats!",
                micronutrients_snapshot,
                created_at
            "#,
            raw_id,
            log.user_id as UserId,
            log.logged_at,
            log.meal_type,
            log.recipe_id as Option<RecipeId>,
            log.food_item_id as Option<FoodItemId>,
            log.custom_food_name,
            log.quantity,
            log.unit,
            log.calories,
            log.protein,
            log.carbs,
            log.fats,
            log.micronutrients_snapshot
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(FoodLog {
            id: res.id,
            user_id: res.user_id,
            logged_at: res.logged_at,
            meal_type: res.meal_type,
            recipe_id: res.recipe_id,
            food_item_id: res.food_item_id,
            custom_food_name: res.custom_food_name,
            quantity: res.quantity,
            unit: res.unit,
            calories: res.calories,
            protein: res.protein,
            carbs: res.carbs,
            fats: res.fats,
            micronutrients_snapshot: res.micronutrients_snapshot,
            created_at: res.created_at,
        })
    }

    async fn get_food_logs(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<FoodLog>, u64)> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let total = query_scalar!(
            r#"SELECT count(*) FROM food_logs WHERE user_id = $1"#,
            user_id as UserId
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = query!(
            r#"
            SELECT
                id as "id: FoodLogId",
                user_id as "user_id: UserId",
                logged_at,
                meal_type,
                recipe_id as "recipe_id: RecipeId",
                food_item_id as "food_item_id: FoodItemId",
                custom_food_name,
                quantity::float8 as "quantity!",
                unit,
                calories::float8 as "calories!",
                protein::float8 as "protein!",
                carbs::float8 as "carbs!",
                fats::float8 as "fats!",
                micronutrients_snapshot,
                created_at
            FROM food_logs
            WHERE user_id = $1
            ORDER BY logged_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id as UserId,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|r| FoodLog {
                id: r.id,
                user_id: r.user_id,
                logged_at: r.logged_at,
                meal_type: r.meal_type,
                recipe_id: r.recipe_id,
                food_item_id: r.food_item_id,
                custom_food_name: r.custom_food_name,
                quantity: r.quantity,
                unit: r.unit,
                calories: r.calories,
                protein: r.protein,
                carbs: r.carbs,
                fats: r.fats,
                micronutrients_snapshot: r.micronutrients_snapshot,
                created_at: r.created_at,
            })
            .collect();

        Ok((logs, total))
    }

    async fn get_daily_macros(&self, user_id: UserId, date: chrono::NaiveDate) -> AppResult<(f64, f64, f64, f64)> {
        let start_time = date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(chrono::Utc).unwrap();
        let end_time = date.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(chrono::Utc).unwrap();

        let row = query!(
            r#"
            SELECT
                COALESCE(SUM(calories), 0.0)::float8 as "calories!",
                COALESCE(SUM(protein), 0.0)::float8 as "protein!",
                COALESCE(SUM(carbs), 0.0)::float8 as "carbs!",
                COALESCE(SUM(fats), 0.0)::float8 as "fats!"
            FROM food_logs
            WHERE user_id = $1 AND logged_at >= $2 AND logged_at <= $3
            "#,
            user_id as UserId,
            start_time,
            end_time
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((row.calories, row.protein, row.carbs, row.fats))
    }

    // --- Raw Food Cost Methods (Admin) ---

    async fn create_raw_food_cost(&self, pattern: &str, cost: f64, currency: &str) -> AppResult<RawFoodCost> {
        let mut tx = self.pool.begin().await?;
        let id = RawFoodCostId::new();
        let raw_id: Uuid = id.into();

        query!(
            r#"
            INSERT INTO raw_food_costs (id, food_pattern, cost_per_100g, price_currency)
            VALUES ($1, $2, $3::float8, $4)
            "#,
            raw_id,
            pattern,
            cost,
            currency
        )
        .execute(&mut *tx)
        .await?;

        // Auto-link food items matching the pattern
        query!(
            r#"
            UPDATE food_items
            SET raw_food_cost_id = $1
            WHERE name ILIKE '%' || $2 || '%'
              AND raw_food_cost_id IS NULL
            "#,
            raw_id,
            pattern
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let res = query!(
            r#"
            SELECT
                id as "id: RawFoodCostId",
                food_pattern,
                cost_per_100g::float8 as "cost_per_100g!",
                price_currency,
                created_at,
                updated_at
            FROM raw_food_costs
            WHERE id = $1
            "#,
            raw_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(RawFoodCost {
            id: res.id,
            food_pattern: res.food_pattern,
            cost_per_100g: res.cost_per_100g,
            price_currency: res.price_currency,
            created_at: res.created_at,
            updated_at: res.updated_at,
        })
    }

    async fn get_raw_food_cost(&self, id: RawFoodCostId) -> AppResult<Option<RawFoodCost>> {
        let raw_id: Uuid = id.into();
        let res = query!(
            r#"
            SELECT
                id as "id: RawFoodCostId",
                food_pattern,
                cost_per_100g::float8 as "cost_per_100g!",
                price_currency,
                created_at,
                updated_at
            FROM raw_food_costs
            WHERE id = $1
            "#,
            raw_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(|r| RawFoodCost {
            id: r.id,
            food_pattern: r.food_pattern,
            cost_per_100g: r.cost_per_100g,
            price_currency: r.price_currency,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list_raw_food_costs(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<RawFoodCost>, u64)> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;
        let like_query = format!("%{}%", query);

        let total = query_scalar!(
            r#"SELECT count(*) FROM raw_food_costs WHERE food_pattern ILIKE $1"#,
            like_query
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = query!(
            r#"
            SELECT
                id as "id: RawFoodCostId",
                food_pattern,
                cost_per_100g::float8 as "cost_per_100g!",
                price_currency,
                created_at,
                updated_at
            FROM raw_food_costs
            WHERE food_pattern ILIKE $1
            ORDER BY food_pattern ASC
            LIMIT $2 OFFSET $3
            "#,
            like_query,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let costs = rows
            .into_iter()
            .map(|r| RawFoodCost {
                id: r.id,
                food_pattern: r.food_pattern,
                cost_per_100g: r.cost_per_100g,
                price_currency: r.price_currency,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok((costs, total))
    }

    async fn update_raw_food_cost(&self, id: RawFoodCostId, pattern: &str, cost: f64, currency: &str) -> AppResult<RawFoodCost> {
        let mut tx = self.pool.begin().await?;
        let raw_id: Uuid = id.into();

        query!(
            r#"
            UPDATE raw_food_costs
            SET food_pattern = $2, cost_per_100g = $3::float8, price_currency = $4, updated_at = NOW()
            WHERE id = $1
            "#,
            raw_id,
            pattern,
            cost,
            currency
        )
        .execute(&mut *tx)
        .await?;

        // Re-link food items that match the new pattern
        query!(
            r#"
            UPDATE food_items
            SET raw_food_cost_id = $1
            WHERE name ILIKE '%' || $2 || '%'
              AND (raw_food_cost_id IS NULL OR raw_food_cost_id = $1)
            "#,
            raw_id,
            pattern
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let res = query!(
            r#"
            SELECT
                id as "id: RawFoodCostId",
                food_pattern,
                cost_per_100g::float8 as "cost_per_100g!",
                price_currency,
                created_at,
                updated_at
            FROM raw_food_costs
            WHERE id = $1
            "#,
            raw_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(RawFoodCost {
            id: res.id,
            food_pattern: res.food_pattern,
            cost_per_100g: res.cost_per_100g,
            price_currency: res.price_currency,
            created_at: res.created_at,
            updated_at: res.updated_at,
        })
    }

    async fn delete_raw_food_cost(&self, id: RawFoodCostId) -> AppResult<()> {
        let raw_id: Uuid = id.into();
        query!(
            r#"
            DELETE FROM raw_food_costs
            WHERE id = $1
            "#,
            raw_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn link_food_item_to_cost(&self, food_item_id: FoodItemId, cost_id: Option<RawFoodCostId>) -> AppResult<()> {
        let raw_food_item_id: Uuid = food_item_id.into();
        let raw_cost_id: Option<Uuid> = cost_id.map(|id| id.into());

        query!(
            r#"
            UPDATE food_items
            SET raw_food_cost_id = $2
            WHERE id = $1
            "#,
            raw_food_item_id,
            raw_cost_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
