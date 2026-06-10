use sqlx::{PgPool, query_scalar};
use uuid::Uuid;
use chrono::NaiveDate;

use crate::common::AppResult;
use crate::common::id::{UserId, MealPlanId, MealPlanItemId, PantryItemId, ShoppingListId, ShoppingListItemId, RecipeId, IngredientId};
use crate::plan::models::{MealPlan, MealPlanItem, PantryItem, ShoppingList, ShoppingListItem};
use crate::plan::repository::PlanRepository;

#[derive(Clone)]
pub struct PgPlanRepository {
    pool: PgPool,
}

impl PgPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PlanRepository for PgPlanRepository {
    // Meal Plans
    async fn create_meal_plan(&self, plan: MealPlan, items: Vec<MealPlanItem>) -> AppResult<(MealPlan, Vec<MealPlanItem>)> {
        let mut tx = self.pool.begin().await?;

        let plan_id: Uuid = plan.id.into();
        let user_id_uuid: Uuid = plan.user_id.into();

        // If this plan is active, deactivate others
        if plan.is_active {
            sqlx::query!(
                "UPDATE meal_plans SET is_active = false WHERE user_id = $1",
                user_id_uuid
            )
            .execute(&mut *tx)
            .await?;
        }

        let plan_row = sqlx::query!(
            r#"
            INSERT INTO meal_plans (id, user_id, name, start_date, end_date, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id as "id: MealPlanId",
                user_id as "user_id: UserId",
                name,
                start_date,
                end_date,
                is_active,
                created_at,
                updated_at
            "#,
            plan_id,
            user_id_uuid,
            plan.name,
            plan.start_date,
            plan.end_date,
            plan.is_active,
            plan.created_at,
            plan.updated_at
        )
        .fetch_one(&mut *tx)
        .await?;

        let inserted_plan = MealPlan {
            id: plan_row.id,
            user_id: plan_row.user_id,
            name: plan_row.name,
            start_date: plan_row.start_date,
            end_date: plan_row.end_date,
            is_active: plan_row.is_active,
            created_at: plan_row.created_at,
            updated_at: plan_row.updated_at,
        };

        let mut inserted_items = Vec::new();
        for item in items {
            let item_id: Uuid = item.id.into();
            let recipe_id_uuid: Option<Uuid> = item.recipe_id.map(|id| id.into());
            let ingredient_id_uuid: Option<Uuid> = item.ingredient_id.map(|id| id.into());

            let item_row = sqlx::query!(
                r#"
                INSERT INTO meal_plan_items (id, meal_plan_id, planned_date, meal_type, recipe_id, ingredient_id, custom_food_name, servings, consumed, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8::float8, $9, $10)
                RETURNING
                    id as "id: MealPlanItemId",
                    meal_plan_id as "meal_plan_id: MealPlanId",
                    planned_date,
                    meal_type,
                    recipe_id as "recipe_id: RecipeId",
                    ingredient_id as "ingredient_id: IngredientId",
                    custom_food_name,
                    servings::float8 as "servings!",
                    consumed,
                    created_at
                "#,
                item_id,
                plan_id,
                item.planned_date,
                item.meal_type,
                recipe_id_uuid,
                ingredient_id_uuid,
                item.custom_food_name,
                item.servings,
                item.consumed,
                item.created_at
            )
            .fetch_one(&mut *tx)
            .await?;

            inserted_items.push(MealPlanItem {
                id: item_row.id,
                meal_plan_id: item_row.meal_plan_id,
                planned_date: item_row.planned_date,
                meal_type: item_row.meal_type,
                recipe_id: item_row.recipe_id,
                ingredient_id: item_row.ingredient_id,
                custom_food_name: item_row.custom_food_name,
                servings: item_row.servings,
                consumed: item_row.consumed,
                created_at: item_row.created_at,
            });
        }

        tx.commit().await?;
        Ok((inserted_plan, inserted_items))
    }

    async fn get_meal_plan(&self, id: MealPlanId) -> AppResult<Option<(MealPlan, Vec<MealPlanItem>)>> {
        let plan_uuid: Uuid = id.into();
        let plan_row = sqlx::query!(
            r#"
            SELECT
                id as "id: MealPlanId",
                user_id as "user_id: UserId",
                name,
                start_date,
                end_date,
                is_active,
                created_at,
                updated_at
            FROM meal_plans
            WHERE id = $1
            "#,
            plan_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(p) = plan_row else {
            return Ok(None);
        };

        let items_rows = sqlx::query!(
            r#"
            SELECT
                id as "id: MealPlanItemId",
                meal_plan_id as "meal_plan_id: MealPlanId",
                planned_date,
                meal_type,
                recipe_id as "recipe_id: RecipeId",
                ingredient_id as "ingredient_id: IngredientId",
                custom_food_name,
                servings::float8 as "servings!",
                consumed,
                created_at
            FROM meal_plan_items
            WHERE meal_plan_id = $1
            ORDER BY planned_date ASC, meal_type ASC
            "#,
            plan_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let plan = MealPlan {
            id: p.id,
            user_id: p.user_id,
            name: p.name,
            start_date: p.start_date,
            end_date: p.end_date,
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
        };

        let items = items_rows
            .into_iter()
            .map(|r| MealPlanItem {
                id: r.id,
                meal_plan_id: r.meal_plan_id,
                planned_date: r.planned_date,
                meal_type: r.meal_type,
                recipe_id: r.recipe_id,
                ingredient_id: r.ingredient_id,
                custom_food_name: r.custom_food_name,
                servings: r.servings,
                consumed: r.consumed,
                created_at: r.created_at,
            })
            .collect();

        Ok(Some((plan, items)))
    }

    async fn get_active_meal_plan(&self, user_id: UserId) -> AppResult<Option<(MealPlan, Vec<MealPlanItem>)>> {
        let user_uuid: Uuid = user_id.into();
        let plan_row = sqlx::query!(
            r#"
            SELECT
                id as "id: MealPlanId",
                user_id as "user_id: UserId",
                name,
                start_date,
                end_date,
                is_active,
                created_at,
                updated_at
            FROM meal_plans
            WHERE user_id = $1 AND is_active = true
            "#,
            user_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(p) = plan_row else {
            return Ok(None);
        };

        let plan_uuid: Uuid = p.id.into();
        let items_rows = sqlx::query!(
            r#"
            SELECT
                id as "id: MealPlanItemId",
                meal_plan_id as "meal_plan_id: MealPlanId",
                planned_date,
                meal_type,
                recipe_id as "recipe_id: RecipeId",
                ingredient_id as "ingredient_id: IngredientId",
                custom_food_name,
                servings::float8 as "servings!",
                consumed,
                created_at
            FROM meal_plan_items
            WHERE meal_plan_id = $1
            ORDER BY planned_date ASC, meal_type ASC
            "#,
            plan_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let plan = MealPlan {
            id: p.id,
            user_id: p.user_id,
            name: p.name,
            start_date: p.start_date,
            end_date: p.end_date,
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
        };

        let items = items_rows
            .into_iter()
            .map(|r| MealPlanItem {
                id: r.id,
                meal_plan_id: r.meal_plan_id,
                planned_date: r.planned_date,
                meal_type: r.meal_type,
                recipe_id: r.recipe_id,
                ingredient_id: r.ingredient_id,
                custom_food_name: r.custom_food_name,
                servings: r.servings,
                consumed: r.consumed,
                created_at: r.created_at,
            })
            .collect();

        Ok(Some((plan, items)))
    }

    async fn get_meal_plan_item(&self, id: MealPlanItemId) -> AppResult<Option<MealPlanItem>> {
        let item_uuid: Uuid = id.into();
        let row = sqlx::query!(
            r#"
            SELECT
                id as "id: MealPlanItemId",
                meal_plan_id as "meal_plan_id: MealPlanId",
                planned_date,
                meal_type,
                recipe_id as "recipe_id: RecipeId",
                ingredient_id as "ingredient_id: IngredientId",
                custom_food_name,
                servings::float8 as "servings!",
                consumed,
                created_at
            FROM meal_plan_items
            WHERE id = $1
            "#,
            item_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| MealPlanItem {
            id: r.id,
            meal_plan_id: r.meal_plan_id,
            planned_date: r.planned_date,
            meal_type: r.meal_type,
            recipe_id: r.recipe_id,
            ingredient_id: r.ingredient_id,
            custom_food_name: r.custom_food_name,
            servings: r.servings,
            consumed: r.consumed,
            created_at: r.created_at,
        }))
    }

    async fn list_meal_plans(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<MealPlan>, u64)> {
        let user_uuid: Uuid = user_id.into();
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let total = query_scalar!(
            r#"SELECT count(*) FROM meal_plans WHERE user_id = $1"#,
            user_uuid
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id: MealPlanId",
                user_id as "user_id: UserId",
                name,
                start_date,
                end_date,
                is_active,
                created_at,
                updated_at
            FROM meal_plans
            WHERE user_id = $1
            ORDER BY start_date DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_uuid,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let plans = rows
            .into_iter()
            .map(|r| MealPlan {
                id: r.id,
                user_id: r.user_id,
                name: r.name,
                start_date: r.start_date,
                end_date: r.end_date,
                is_active: r.is_active,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok((plans, total))
    }

    async fn set_meal_plan_active(&self, user_id: UserId, id: MealPlanId, active: bool) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        let user_uuid: Uuid = user_id.into();
        let plan_uuid: Uuid = id.into();

        if active {
            sqlx::query!(
                "UPDATE meal_plans SET is_active = false WHERE user_id = $1 AND id != $2",
                user_uuid,
                plan_uuid
            )
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query!(
            "UPDATE meal_plans SET is_active = $1, updated_at = now() WHERE user_id = $2 AND id = $3",
            active,
            user_uuid,
            plan_uuid
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn update_meal_plan_item_consumed(&self, item_id: MealPlanItemId, consumed: bool) -> AppResult<Option<MealPlanItem>> {
        let item_uuid: Uuid = item_id.into();
        let row = sqlx::query!(
            r#"
            UPDATE meal_plan_items
            SET consumed = $1
            WHERE id = $2
            RETURNING
                id as "id: MealPlanItemId",
                meal_plan_id as "meal_plan_id: MealPlanId",
                planned_date,
                meal_type,
                recipe_id as "recipe_id: RecipeId",
                ingredient_id as "ingredient_id: IngredientId",
                custom_food_name,
                servings::float8 as "servings!",
                consumed,
                created_at
            "#,
            consumed,
            item_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| MealPlanItem {
            id: r.id,
            meal_plan_id: r.meal_plan_id,
            planned_date: r.planned_date,
            meal_type: r.meal_type,
            recipe_id: r.recipe_id,
            ingredient_id: r.ingredient_id,
            custom_food_name: r.custom_food_name,
            servings: r.servings,
            consumed: r.consumed,
            created_at: r.created_at,
        }))
    }

    // Pantry
    async fn get_pantry_item(&self, id: PantryItemId) -> AppResult<Option<PantryItem>> {
        let item_uuid: Uuid = id.into();
        let row = sqlx::query!(
            r#"
            SELECT
                id as "id: PantryItemId",
                user_id as "user_id: UserId",
                ingredient_id as "ingredient_id: IngredientId",
                quantity::float8 as "quantity!",
                unit,
                expires_at,
                purchased_at,
                created_at,
                updated_at
            FROM pantry_items
            WHERE id = $1
            "#,
            item_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| PantryItem {
            id: r.id,
            user_id: r.user_id,
            ingredient_id: r.ingredient_id,
            quantity: r.quantity,
            unit: r.unit,
            expires_at: r.expires_at,
            purchased_at: r.purchased_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list_pantry_items(&self, user_id: UserId) -> AppResult<Vec<PantryItem>> {
        let user_uuid: Uuid = user_id.into();
        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id: PantryItemId",
                user_id as "user_id: UserId",
                ingredient_id as "ingredient_id: IngredientId",
                quantity::float8 as "quantity!",
                unit,
                expires_at,
                purchased_at,
                created_at,
                updated_at
            FROM pantry_items
            WHERE user_id = $1
            ORDER BY expires_at ASC NULLS LAST, created_at DESC
            "#,
            user_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let items = rows
            .into_iter()
            .map(|r| PantryItem {
                id: r.id,
                user_id: r.user_id,
                ingredient_id: r.ingredient_id,
                quantity: r.quantity,
                unit: r.unit,
                expires_at: r.expires_at,
                purchased_at: r.purchased_at,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(items)
    }

    async fn create_pantry_item(&self, item: PantryItem) -> AppResult<PantryItem> {
        let item_uuid: Uuid = item.id.into();
        let user_uuid: Uuid = item.user_id.into();
        let ing_uuid: Uuid = item.ingredient_id.into();

        let row = sqlx::query!(
            r#"
            INSERT INTO pantry_items (id, user_id, ingredient_id, quantity, unit, expires_at, purchased_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4::float8, $5, $6, $7, $8, $9)
            RETURNING
                id as "id: PantryItemId",
                user_id as "user_id: UserId",
                ingredient_id as "ingredient_id: IngredientId",
                quantity::float8 as "quantity!",
                unit,
                expires_at,
                purchased_at,
                created_at,
                updated_at
            "#,
            item_uuid,
            user_uuid,
            ing_uuid,
            item.quantity,
            item.unit,
            item.expires_at,
            item.purchased_at,
            item.created_at,
            item.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PantryItem {
            id: row.id,
            user_id: row.user_id,
            ingredient_id: row.ingredient_id,
            quantity: row.quantity,
            unit: row.unit,
            expires_at: row.expires_at,
            purchased_at: row.purchased_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn update_pantry_item(&self, id: PantryItemId, quantity: f64, expires_at: Option<NaiveDate>) -> AppResult<PantryItem> {
        let item_uuid: Uuid = id.into();
        let row = sqlx::query!(
            r#"
            UPDATE pantry_items
            SET quantity = $1::float8, expires_at = $2, updated_at = now()
            WHERE id = $3
            RETURNING
                id as "id: PantryItemId",
                user_id as "user_id: UserId",
                ingredient_id as "ingredient_id: IngredientId",
                quantity::float8 as "quantity!",
                unit,
                expires_at,
                purchased_at,
                created_at,
                updated_at
            "#,
            quantity,
            expires_at,
            item_uuid
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PantryItem {
            id: row.id,
            user_id: row.user_id,
            ingredient_id: row.ingredient_id,
            quantity: row.quantity,
            unit: row.unit,
            expires_at: row.expires_at,
            purchased_at: row.purchased_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn delete_pantry_item(&self, id: PantryItemId) -> AppResult<()> {
        let item_uuid: Uuid = id.into();
        sqlx::query!("DELETE FROM pantry_items WHERE id = $1", item_uuid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Shopping Lists
    async fn create_shopping_list(&self, list: ShoppingList, items: Vec<ShoppingListItem>) -> AppResult<(ShoppingList, Vec<ShoppingListItem>)> {
        let mut tx = self.pool.begin().await?;

        let list_uuid: Uuid = list.id.into();
        let user_uuid: Uuid = list.user_id.into();

        let list_row = sqlx::query!(
            r#"
            INSERT INTO shopping_lists (id, user_id, name, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id as "id: ShoppingListId",
                user_id as "user_id: UserId",
                name,
                status,
                created_at,
                updated_at
            "#,
            list_uuid,
            user_uuid,
            list.name,
            list.status,
            list.created_at,
            list.updated_at
        )
        .fetch_one(&mut *tx)
        .await?;

        let inserted_list = ShoppingList {
            id: list_row.id,
            user_id: list_row.user_id,
            name: list_row.name,
            status: list_row.status,
            created_at: list_row.created_at,
            updated_at: list_row.updated_at,
        };

        let mut inserted_items = Vec::new();
        for item in items {
            let item_uuid: Uuid = item.id.into();
            let ing_uuid: Option<Uuid> = item.ingredient_id.map(|id| id.into());

            let item_row = sqlx::query!(
                r#"
                INSERT INTO shopping_list_items (id, shopping_list_id, ingredient_id, custom_item_name, quantity, unit, is_acquired, category, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5::float8, $6, $7, $8, $9, $10)
                RETURNING
                    id as "id: ShoppingListItemId",
                    shopping_list_id as "shopping_list_id: ShoppingListId",
                    ingredient_id as "ingredient_id: IngredientId",
                    custom_item_name,
                    quantity::float8 as "quantity!",
                    unit,
                    is_acquired,
                    category as "category!",
                    created_at,
                    updated_at
                "#,
                item_uuid,
                list_uuid,
                ing_uuid,
                item.custom_item_name,
                item.quantity,
                item.unit,
                item.is_acquired,
                item.category,
                item.created_at,
                item.updated_at
            )
            .fetch_one(&mut *tx)
            .await?;

            inserted_items.push(ShoppingListItem {
                id: item_row.id,
                shopping_list_id: item_row.shopping_list_id,
                ingredient_id: item_row.ingredient_id,
                custom_item_name: item_row.custom_item_name,
                quantity: item_row.quantity,
                unit: item_row.unit,
                is_acquired: item_row.is_acquired,
                category: item_row.category,
                created_at: item_row.created_at,
                updated_at: item_row.updated_at,
            });
        }

        tx.commit().await?;
        Ok((inserted_list, inserted_items))
    }

    async fn get_shopping_list(&self, id: ShoppingListId) -> AppResult<Option<(ShoppingList, Vec<ShoppingListItem>)>> {
        let list_uuid: Uuid = id.into();
        let list_row = sqlx::query!(
            r#"
            SELECT
                id as "id: ShoppingListId",
                user_id as "user_id: UserId",
                name,
                status,
                created_at,
                updated_at
            FROM shopping_lists
            WHERE id = $1
            "#,
            list_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(l) = list_row else {
            return Ok(None);
        };

        let items_rows = sqlx::query!(
            r#"
            SELECT
                id as "id: ShoppingListItemId",
                shopping_list_id as "shopping_list_id: ShoppingListId",
                ingredient_id as "ingredient_id: IngredientId",
                custom_item_name,
                quantity::float8 as "quantity!",
                unit,
                is_acquired,
                category as "category!",
                created_at,
                updated_at
            FROM shopping_list_items
            WHERE shopping_list_id = $1
            ORDER BY category ASC, created_at DESC
            "#,
            list_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let list = ShoppingList {
            id: l.id,
            user_id: l.user_id,
            name: l.name,
            status: l.status,
            created_at: l.created_at,
            updated_at: l.updated_at,
        };

        let items = items_rows
            .into_iter()
            .map(|r| ShoppingListItem {
                id: r.id,
                shopping_list_id: r.shopping_list_id,
                ingredient_id: r.ingredient_id,
                custom_item_name: r.custom_item_name,
                quantity: r.quantity,
                unit: r.unit,
                is_acquired: r.is_acquired,
                category: r.category,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(Some((list, items)))
    }

    async fn list_shopping_lists(&self, user_id: UserId) -> AppResult<Vec<ShoppingList>> {
        let user_uuid: Uuid = user_id.into();
        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id: ShoppingListId",
                user_id as "user_id: UserId",
                name,
                status,
                created_at,
                updated_at
            FROM shopping_lists
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_uuid
        )
        .fetch_all(&self.pool)
        .await?;

        let lists = rows
            .into_iter()
            .map(|r| ShoppingList {
                id: r.id,
                user_id: r.user_id,
                name: r.name,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(lists)
    }

    async fn update_shopping_list_item_acquired(&self, list_id: ShoppingListId, item_id: ShoppingListItemId, is_acquired: bool) -> AppResult<()> {
        let list_uuid: Uuid = list_id.into();
        let item_uuid: Uuid = item_id.into();

        sqlx::query!(
            r#"
            UPDATE shopping_list_items
            SET is_acquired = $1, updated_at = now()
            WHERE id = $2 AND shopping_list_id = $3
            "#,
            is_acquired,
            item_uuid,
            list_uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
