use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::id::{UserId, RecipeId, FoodItemId, FoodLogId, FoodItemPortionId, RawFoodCostId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodItem {
    pub id: FoodItemId,
    pub name: String,
    pub description: Option<String>,
    pub calories_per_100g: f64,
    pub protein_per_100g: f64,
    pub carbs_per_100g: f64,
    pub fat_per_100g: f64,
    pub fiber_per_100g: f64,
    pub sodium_mg_per_100g: f64,
    pub micronutrients: serde_json::Value,
    pub estimated_cost_per_100g: f64, // Populated dynamically via JOIN
    pub price_currency: String,
    pub barcode: Option<String>,
    pub is_verified: bool,
    pub food_code: Option<String>,
    pub primary_source: Option<String>,
    pub raw_food_cost_id: Option<RawFoodCostId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodItemPortion {
    pub id: FoodItemPortionId,
    pub food_item_id: FoodItemId,
    pub serving_unit: String,
    pub grams_equivalent: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFoodCost {
    pub id: RawFoodCostId,
    pub food_pattern: String,
    pub cost_per_100g: f64,
    pub price_currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub owner_id: Option<UserId>,
    pub parent_recipe_id: Option<RecipeId>,
    pub title: String,
    pub description: Option<String>,
    pub instructions: Vec<String>,
    pub prep_time_minutes: Option<i32>,
    pub cook_time_minutes: Option<i32>,
    pub servings: f64,
    pub cuisine: Option<String>,
    pub course: Option<String>,
    pub dietary_tags: Vec<String>,
    pub source_url: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFoodItem {
    pub recipe_id: RecipeId,
    pub food_item_id: FoodItemId,
    pub quantity: f64,
    pub unit: String,
    pub grams_equivalent: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFoodItemDetail {
    pub food_item_id: FoodItemId,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub grams_equivalent: f64,
    pub calories_per_100g: f64,
    pub protein_per_100g: f64,
    pub carbs_per_100g: f64,
    pub fat_per_100g: f64,
    pub fiber_per_100g: f64,
    pub sodium_mg_per_100g: f64,
    pub estimated_cost_per_100g: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeWithFoodItems {
    pub recipe: Recipe,
    pub food_items: Vec<RecipeFoodItemDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodLog {
    pub id: FoodLogId,
    pub user_id: UserId,
    pub logged_at: DateTime<Utc>,
    pub meal_type: String,
    pub recipe_id: Option<RecipeId>,
    pub food_item_id: Option<FoodItemId>,
    pub custom_food_name: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub calories: f64,
    pub protein: f64,
    pub carbs: f64,
    pub fats: f64,
    pub micronutrients_snapshot: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
