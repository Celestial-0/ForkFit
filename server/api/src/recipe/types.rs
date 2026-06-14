use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::common::id::{UserId, RecipeId, FoodItemId, FoodLogId, RawFoodCostId};
use super::models::{FoodItem, Recipe, RecipeFoodItemDetail, FoodLog, RawFoodCost};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFoodItemRequest {
    pub name: String,
    pub description: Option<String>,
    pub calories_per_100g: f64,
    pub protein_per_100g: f64,
    pub carbs_per_100g: f64,
    pub fat_per_100g: f64,
    pub fiber_per_100g: Option<f64>,
    pub sodium_mg_per_100g: Option<f64>,
    pub micronutrients: Option<serde_json::Value>,
    pub barcode: Option<String>,
    pub raw_food_cost_id: Option<RawFoodCostId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodItemResponse {
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
    pub estimated_cost_per_100g: f64,
    pub price_currency: String,
    pub barcode: Option<String>,
    pub is_verified: bool,
    pub food_code: Option<String>,
    pub primary_source: Option<String>,
    pub raw_food_cost_id: Option<RawFoodCostId>,
    pub created_at: DateTime<Utc>,
}

impl From<FoodItem> for FoodItemResponse {
    fn from(i: FoodItem) -> Self {
        Self {
            id: i.id,
            name: i.name,
            description: i.description,
            calories_per_100g: i.calories_per_100g,
            protein_per_100g: i.protein_per_100g,
            carbs_per_100g: i.carbs_per_100g,
            fat_per_100g: i.fat_per_100g,
            fiber_per_100g: i.fiber_per_100g,
            sodium_mg_per_100g: i.sodium_mg_per_100g,
            micronutrients: i.micronutrients,
            estimated_cost_per_100g: i.estimated_cost_per_100g,
            price_currency: i.price_currency,
            barcode: i.barcode,
            is_verified: i.is_verified,
            food_code: i.food_code,
            primary_source: i.primary_source,
            raw_food_cost_id: i.raw_food_cost_id,
            created_at: i.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFoodItemInput {
    pub food_item_id: FoodItemId,
    pub quantity: f64,
    pub unit: String,
    pub grams_equivalent: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
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
    pub food_items: Vec<RecipeFoodItemInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeResponse {
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
}

impl From<Recipe> for RecipeResponse {
    fn from(r: Recipe) -> Self {
        Self {
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFoodItemDetailResponse {
    pub food_item_id: FoodItemId,
    pub name: String,
    pub quantity: f64,
    pub unit: String,
    pub grams_equivalent: f64,
    pub notes: Option<String>,
}

impl From<RecipeFoodItemDetail> for RecipeFoodItemDetailResponse {
    fn from(d: RecipeFoodItemDetail) -> Self {
        Self {
            food_item_id: d.food_item_id,
            name: d.name,
            quantity: d.quantity,
            unit: d.unit,
            grams_equivalent: d.grams_equivalent,
            notes: d.notes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeNutrients {
    pub calories: f64,
    pub protein: f64,
    pub carbs: f64,
    pub fat: f64,
    pub fiber: f64,
    pub sodium: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDetailResponse {
    pub recipe: RecipeResponse,
    pub food_items: Vec<RecipeFoodItemDetailResponse>,
    pub total_nutrition: RecipeNutrients,
    pub serving_nutrition: RecipeNutrients,
    pub total_estimated_cost: f64,
    pub serving_estimated_cost: f64,
    pub detected_allergens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFoodRequest {
    pub logged_at: Option<DateTime<Utc>>,
    pub meal_type: String, // 'breakfast', 'lunch', 'dinner', 'snack'
    pub recipe_id: Option<RecipeId>,
    pub food_item_id: Option<FoodItemId>,
    pub custom_food_name: Option<String>,
    pub quantity: f64,
    pub unit: String, // 'servings', 'grams'
    
    // Explicit denormalized macros if it is a custom food
    pub calories: Option<f64>,
    pub protein: Option<f64>,
    pub carbs: Option<f64>,
    pub fats: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodLogResponse {
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
    pub created_at: DateTime<Utc>,
}

impl From<FoodLog> for FoodLogResponse {
    fn from(f: FoodLog) -> Self {
        Self {
            id: f.id,
            user_id: f.user_id,
            logged_at: f.logged_at,
            meal_type: f.meal_type,
            recipe_id: f.recipe_id,
            food_item_id: f.food_item_id,
            custom_food_name: f.custom_food_name,
            quantity: f.quantity,
            unit: f.unit,
            calories: f.calories,
            protein: f.protein,
            carbs: f.carbs,
            fats: f.fats,
            created_at: f.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMacrosSummary {
    pub date: NaiveDate,
    pub calories: f64,
    pub protein: f64,
    pub carbs: f64,
    pub fats: f64,
}

// --- Raw Food Cost Requests / Responses ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRawFoodCostRequest {
    pub food_pattern: String,
    pub cost_per_100g: f64,
    pub price_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRawFoodCostRequest {
    pub food_pattern: String,
    pub cost_per_100g: f64,
    pub price_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFoodCostResponse {
    pub id: RawFoodCostId,
    pub food_pattern: String,
    pub cost_per_100g: f64,
    pub price_currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<RawFoodCost> for RawFoodCostResponse {
    fn from(c: RawFoodCost) -> Self {
        Self {
            id: c.id,
            food_pattern: c.food_pattern,
            cost_per_100g: c.cost_per_100g,
            price_currency: c.price_currency,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}
