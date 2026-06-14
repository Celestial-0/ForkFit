use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use crate::common::id::{UserId, MealPlanId, MealPlanItemId, PantryItemId, ShoppingListId, ShoppingListItemId, RecipeId, FoodItemId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlan {
    pub id: MealPlanId,
    pub user_id: UserId,
    pub name: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanItem {
    pub id: MealPlanItemId,
    pub meal_plan_id: MealPlanId,
    pub planned_date: NaiveDate,
    pub meal_type: String, // 'breakfast', 'lunch', 'dinner', 'snack'
    pub recipe_id: Option<RecipeId>,
    pub food_item_id: Option<FoodItemId>,
    pub custom_food_name: Option<String>,
    pub servings: f64,
    pub consumed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PantryItem {
    pub id: PantryItemId,
    pub user_id: UserId,
    pub food_item_id: FoodItemId,
    pub quantity: f64,
    pub unit: String,
    pub expires_at: Option<NaiveDate>,
    pub purchased_at: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingList {
    pub id: ShoppingListId,
    pub user_id: UserId,
    pub name: String,
    pub status: String, // 'active', 'completed', 'archived'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListItem {
    pub id: ShoppingListItemId,
    pub shopping_list_id: ShoppingListId,
    pub food_item_id: Option<FoodItemId>,
    pub custom_item_name: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub is_acquired: bool,
    pub category: String, // 'Produce', 'Meat', 'Pantry', 'Other', etc.
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
