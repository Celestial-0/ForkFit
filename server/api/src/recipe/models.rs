use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::id::{UserId, RecipeId, IngredientId, FoodLogId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingredient {
    pub id: IngredientId,
    pub name: String,
    pub description: Option<String>,
    pub calories_per_100g: f64,
    pub protein_per_100g: f64,
    pub carbs_per_100g: f64,
    pub fat_per_100g: f64,
    pub fiber_per_100g: f64,
    pub sodium_mg_per_100g: f64,
    pub micronutrients: serde_json::Value,
    pub estimated_cost_per_100g: f64,
    pub price_currency: String,
    pub barcode: Option<String>,
    pub is_verified: bool,
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
    pub dietary_tags: Vec<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub recipe_id: RecipeId,
    pub ingredient_id: IngredientId,
    pub quantity: f64,
    pub unit: String,
    pub grams_equivalent: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredientDetail {
    pub ingredient_id: IngredientId,
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
pub struct RecipeWithIngredients {
    pub recipe: Recipe,
    pub ingredients: Vec<RecipeIngredientDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodLog {
    pub id: FoodLogId,
    pub user_id: UserId,
    pub logged_at: DateTime<Utc>,
    pub meal_type: String,
    pub recipe_id: Option<RecipeId>,
    pub ingredient_id: Option<IngredientId>,
    pub custom_food_name: Option<String>,
    pub quantity: f64,
    pub unit: String,
    pub calories: f64,
    pub protein: f64,
    pub carbs: f64,
    pub fats: f64,
    pub created_at: DateTime<Utc>,
}
