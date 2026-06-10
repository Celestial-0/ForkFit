use sqlx::{PgPool, query, query_scalar};
use uuid::Uuid;

use crate::common::AppResult;
use crate::common::id::{UserId, RecipeId, IngredientId, FoodLogId};
use crate::recipe::models::{
    Ingredient, Recipe, RecipeIngredient, RecipeIngredientDetail, RecipeWithIngredients, FoodLog,
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
    async fn create_ingredient(&self, ing: Ingredient) -> AppResult<Ingredient> {
        let raw_id: Uuid = ing.id.into();
        let res = query!(
            r#"
            INSERT INTO ingredients (id, name, description, calories_per_100g, protein_per_100g, carbs_per_100g, fat_per_100g, fiber_per_100g, sodium_mg_per_100g, micronutrients, estimated_cost_per_100g, price_currency, barcode, is_verified)
            VALUES ($1, $2, $3, $4::float8, $5::float8, $6::float8, $7::float8, $8::float8, $9::float8, $10, $11::float8, $12, $13, $14)
            RETURNING
                id as "id: IngredientId",
                name,
                description,
                calories_per_100g::float8 as "calories_per_100g!",
                protein_per_100g::float8 as "protein_per_100g!",
                carbs_per_100g::float8 as "carbs_per_100g!",
                fat_per_100g::float8 as "fat_per_100g!",
                fiber_per_100g::float8 as "fiber_per_100g!",
                sodium_mg_per_100g::float8 as "sodium_mg_per_100g!",
                micronutrients,
                estimated_cost_per_100g::float8 as "estimated_cost_per_100g!",
                price_currency,
                barcode,
                is_verified,
                created_at,
                updated_at
            "#,
            raw_id,
            ing.name,
            ing.description,
            ing.calories_per_100g,
            ing.protein_per_100g,
            ing.carbs_per_100g,
            ing.fat_per_100g,
            ing.fiber_per_100g,
            ing.sodium_mg_per_100g,
            ing.micronutrients,
            ing.estimated_cost_per_100g,
            ing.price_currency,
            ing.barcode,
            ing.is_verified
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Ingredient {
            id: res.id,
            name: res.name,
            description: res.description,
            calories_per_100g: res.calories_per_100g,
            protein_per_100g: res.protein_per_100g,
            carbs_per_100g: res.carbs_per_100g,
            fat_per_100g: res.fat_per_100g,
            fiber_per_100g: res.fiber_per_100g,
            sodium_mg_per_100g: res.sodium_mg_per_100g,
            micronutrients: res.micronutrients,
            estimated_cost_per_100g: res.estimated_cost_per_100g,
            price_currency: res.price_currency,
            barcode: res.barcode,
            is_verified: res.is_verified,
            created_at: res.created_at,
            updated_at: res.updated_at,
        })
    }

    async fn get_ingredient(&self, id: IngredientId) -> AppResult<Option<Ingredient>> {
        let raw_id: Uuid = id.into();
        let res = query!(
            r#"
            SELECT
                id as "id: IngredientId",
                name,
                description,
                calories_per_100g::float8 as "calories_per_100g!",
                protein_per_100g::float8 as "protein_per_100g!",
                carbs_per_100g::float8 as "carbs_per_100g!",
                fat_per_100g::float8 as "fat_per_100g!",
                fiber_per_100g::float8 as "fiber_per_100g!",
                sodium_mg_per_100g::float8 as "sodium_mg_per_100g!",
                micronutrients,
                estimated_cost_per_100g::float8 as "estimated_cost_per_100g!",
                price_currency,
                barcode,
                is_verified,
                created_at,
                updated_at
            FROM ingredients
            WHERE id = $1
            "#,
            raw_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(res.map(|r| Ingredient {
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
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn search_ingredients(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<Ingredient>, u64)> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;
        let like_query = format!("%{}%", query);

        let total = query_scalar!(
            r#"SELECT count(*) FROM ingredients WHERE name ILIKE $1"#,
            like_query
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = query!(
            r#"
            SELECT
                id as "id: IngredientId",
                name,
                description,
                calories_per_100g::float8 as "calories_per_100g!",
                protein_per_100g::float8 as "protein_per_100g!",
                carbs_per_100g::float8 as "carbs_per_100g!",
                fat_per_100g::float8 as "fat_per_100g!",
                fiber_per_100g::float8 as "fiber_per_100g!",
                sodium_mg_per_100g::float8 as "sodium_mg_per_100g!",
                micronutrients,
                estimated_cost_per_100g::float8 as "estimated_cost_per_100g!",
                price_currency,
                barcode,
                is_verified,
                created_at,
                updated_at
            FROM ingredients
            WHERE name ILIKE $1
            ORDER BY name ASC
            LIMIT $2 OFFSET $3
            "#,
            like_query,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let ingredients = rows
            .into_iter()
            .map(|r| Ingredient {
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
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok((ingredients, total))
    }

    async fn create_recipe(&self, recipe: Recipe, ingredients: Vec<RecipeIngredient>) -> AppResult<RecipeWithIngredients> {
        let mut tx = self.pool.begin().await?;
        let raw_recipe_id: Uuid = recipe.id.into();
        
        // 1. Insert Recipe
        query!(
            r#"
            INSERT INTO recipes (id, owner_id, parent_recipe_id, title, description, instructions, prep_time_minutes, cook_time_minutes, servings, cuisine, dietary_tags, is_public)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::float8, $10, $11, $12)
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
            &recipe.dietary_tags,
            recipe.is_public
        )
        .execute(&mut *tx)
        .await?;

        // 2. Insert Recipe Ingredients
        for ri in ingredients {
            query!(
                r#"
                INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity, unit, grams_equivalent, notes)
                VALUES ($1, $2, $3::float8, $4, $5::float8, $6)
                "#,
                raw_recipe_id,
                ri.ingredient_id as IngredientId,
                ri.quantity,
                ri.unit,
                ri.grams_equivalent,
                ri.notes
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // 3. Fetch composite detail
        let details = self.get_recipe(recipe.id).await?;
        details.ok_or(sqlx::Error::RowNotFound.into())
    }

    async fn get_recipe(&self, id: RecipeId) -> AppResult<Option<RecipeWithIngredients>> {
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
                dietary_tags,
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
                dietary_tags: r.dietary_tags,
                is_public: r.is_public,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            None => return Ok(None),
        };

        let ingredient_rows = query!(
            r#"
            SELECT
                ri.ingredient_id as "ingredient_id: IngredientId",
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
                i.estimated_cost_per_100g::float8 as "estimated_cost_per_100g!",
                ri.notes
            FROM recipe_ingredients ri
            JOIN ingredients i ON ri.ingredient_id = i.id
            WHERE ri.recipe_id = $1
            "#,
            raw_id
        )
        .fetch_all(&self.pool)
        .await?;

        let ingredients = ingredient_rows
            .into_iter()
            .map(|r| RecipeIngredientDetail {
                ingredient_id: r.ingredient_id,
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

        Ok(Some(RecipeWithIngredients { recipe, ingredients }))
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
                    dietary_tags,
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
                    dietary_tags: r.dietary_tags,
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
                    dietary_tags,
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
                    dietary_tags: r.dietary_tags,
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
                dietary_tags,
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
                dietary_tags: r.dietary_tags,
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
            INSERT INTO food_logs (id, user_id, logged_at, meal_type, recipe_id, ingredient_id, custom_food_name, quantity, unit, calories, protein, carbs, fats)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::float8, $9, $10::float8, $11::float8, $12::float8, $13::float8)
            RETURNING
                id as "id: FoodLogId",
                user_id as "user_id: UserId",
                logged_at,
                meal_type,
                recipe_id as "recipe_id: RecipeId",
                ingredient_id as "ingredient_id: IngredientId",
                custom_food_name,
                quantity::float8 as "quantity!",
                unit,
                calories::float8 as "calories!",
                protein::float8 as "protein!",
                carbs::float8 as "carbs!",
                fats::float8 as "fats!",
                created_at
            "#,
            raw_id,
            log.user_id as UserId,
            log.logged_at,
            log.meal_type,
            log.recipe_id as Option<RecipeId>,
            log.ingredient_id as Option<IngredientId>,
            log.custom_food_name,
            log.quantity,
            log.unit,
            log.calories,
            log.protein,
            log.carbs,
            log.fats
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(FoodLog {
            id: res.id,
            user_id: res.user_id,
            logged_at: res.logged_at,
            meal_type: res.meal_type,
            recipe_id: res.recipe_id,
            ingredient_id: res.ingredient_id,
            custom_food_name: res.custom_food_name,
            quantity: res.quantity,
            unit: res.unit,
            calories: res.calories,
            protein: res.protein,
            carbs: res.carbs,
            fats: res.fats,
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
                ingredient_id as "ingredient_id: IngredientId",
                custom_food_name,
                quantity::float8 as "quantity!",
                unit,
                calories::float8 as "calories!",
                protein::float8 as "protein!",
                carbs::float8 as "carbs!",
                fats::float8 as "fats!",
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
                ingredient_id: r.ingredient_id,
                custom_food_name: r.custom_food_name,
                quantity: r.quantity,
                unit: r.unit,
                calories: r.calories,
                protein: r.protein,
                carbs: r.carbs,
                fats: r.fats,
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
}
