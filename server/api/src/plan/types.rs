use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::common::id::{UserId, MealPlanId, MealPlanItemId, PantryItemId, ShoppingListId, ShoppingListItemId, RecipeId, IngredientId};
use super::models::{MealPlan, MealPlanItem, PantryItem, ShoppingList, ShoppingListItem};

// Meal Plans
#[derive(Debug, Clone, Deserialize)]
pub struct MealPlanItemInput {
    pub planned_date: NaiveDate,
    pub meal_type: String, // 'breakfast', 'lunch', 'dinner', 'snack'
    pub recipe_id: Option<RecipeId>,
    pub ingredient_id: Option<IngredientId>,
    pub custom_food_name: Option<String>,
    pub servings: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMealPlanRequest {
    pub name: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_active: bool,
    pub items: Vec<MealPlanItemInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MealPlanResponse {
    pub id: MealPlanId,
    pub user_id: UserId,
    pub name: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MealPlan> for MealPlanResponse {
    fn from(p: MealPlan) -> Self {
        Self {
            id: p.id,
            user_id: p.user_id,
            name: p.name,
            start_date: p.start_date,
            end_date: p.end_date,
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MealPlanItemResponse {
    pub id: MealPlanItemId,
    pub meal_plan_id: MealPlanId,
    pub planned_date: NaiveDate,
    pub meal_type: String,
    pub recipe_id: Option<RecipeId>,
    pub ingredient_id: Option<IngredientId>,
    pub custom_food_name: Option<String>,
    pub servings: f64,
    pub consumed: bool,
    pub created_at: DateTime<Utc>,
}

impl From<MealPlanItem> for MealPlanItemResponse {
    fn from(item: MealPlanItem) -> Self {
        Self {
            id: item.id,
            meal_plan_id: item.meal_plan_id,
            planned_date: item.planned_date,
            meal_type: item.meal_type,
            recipe_id: item.recipe_id,
            ingredient_id: item.ingredient_id,
            custom_food_name: item.custom_food_name,
            servings: item.servings,
            consumed: item.consumed,
            created_at: item.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MealPlanDetailResponse {
    pub plan: MealPlanResponse,
    pub items: Vec<MealPlanItemResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMealPlanStatusRequest {
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMealPlanItemConsumedRequest {
    pub consumed: bool,
}

// Pantry
#[derive(Debug, Clone, Deserialize)]
pub struct CreatePantryItemRequest {
    pub ingredient_id: IngredientId,
    pub quantity: f64,
    pub unit: String,
    pub expires_at: Option<NaiveDate>,
    pub purchased_at: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePantryItemRequest {
    pub quantity: f64,
    pub expires_at: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PantryItemResponse {
    pub id: PantryItemId,
    pub user_id: UserId,
    pub ingredient_id: IngredientId,
    pub quantity: f64,
    pub unit: String,
    pub expires_at: Option<NaiveDate>,
    pub purchased_at: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PantryItem> for PantryItemResponse {
    fn from(item: PantryItem) -> Self {
        Self {
            id: item.id,
            user_id: item.user_id,
            ingredient_id: item.ingredient_id,
            quantity: item.quantity,
            unit: item.unit,
            expires_at: item.expires_at,
            purchased_at: item.purchased_at,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

// Shopping Lists
#[derive(Debug, Clone, Deserialize)]
pub struct ShoppingListItemInput {
    pub ingredient_id: Option<IngredientId>,
    pub custom_item_name: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub category: Option<String>, // Defaults to 'Other' in service
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateShoppingListRequest {
    pub name: String,
    pub items: Vec<ShoppingListItemInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShoppingListResponse {
    pub id: ShoppingListId,
    pub user_id: UserId,
    pub name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShoppingList> for ShoppingListResponse {
    fn from(l: ShoppingList) -> Self {
        Self {
            id: l.id,
            user_id: l.user_id,
            name: l.name,
            status: l.status,
            created_at: l.created_at,
            updated_at: l.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ShoppingListItemResponse {
    pub id: ShoppingListItemId,
    pub shopping_list_id: ShoppingListId,
    pub ingredient_id: Option<IngredientId>,
    pub custom_item_name: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub is_acquired: bool,
    pub category: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShoppingListItem> for ShoppingListItemResponse {
    fn from(item: ShoppingListItem) -> Self {
        Self {
            id: item.id,
            shopping_list_id: item.shopping_list_id,
            ingredient_id: item.ingredient_id,
            custom_item_name: item.custom_item_name,
            quantity: item.quantity,
            unit: item.unit,
            is_acquired: item.is_acquired,
            category: item.category,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ShoppingListDetailResponse {
    pub list: ShoppingListResponse,
    pub items: Vec<ShoppingListItemResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateShoppingListItemAcquiredRequest {
    pub is_acquired: bool,
}
