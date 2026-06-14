use crate::common::AppResult;
use crate::common::id::{UserId, RecipeId, FoodItemId, RawFoodCostId};
use super::models::{FoodItem, Recipe, RecipeFoodItem, RecipeWithFoodItems, FoodLog, FoodItemPortion, RawFoodCost};

pub trait RecipeRepository: Send + Sync {
    // Food Items
    async fn create_food_item(&self, food: FoodItem) -> AppResult<FoodItem>;
    async fn get_food_item(&self, id: FoodItemId) -> AppResult<Option<FoodItem>>;
    async fn search_food_items(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<FoodItem>, u64)>;
    async fn add_food_item_portion(&self, portion: FoodItemPortion) -> AppResult<FoodItemPortion>;
    async fn get_food_item_portions(&self, food_item_id: FoodItemId) -> AppResult<Vec<FoodItemPortion>>;

    // Recipes
    async fn create_recipe(&self, recipe: Recipe, food_items: Vec<RecipeFoodItem>) -> AppResult<RecipeWithFoodItems>;
    async fn get_recipe(&self, id: RecipeId) -> AppResult<Option<RecipeWithFoodItems>>;
    async fn list_recipes(&self, owner_id: Option<UserId>, page: u64, per_page: u64) -> AppResult<(Vec<Recipe>, u64)>;
    async fn get_recipes_by_ids(&self, ids: &[RecipeId]) -> AppResult<Vec<Recipe>>;

    // Food Logs
    async fn log_food(&self, log: FoodLog) -> AppResult<FoodLog>;
    async fn get_food_logs(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<FoodLog>, u64)>;
    async fn get_daily_macros(&self, user_id: UserId, date: chrono::NaiveDate) -> AppResult<(f64, f64, f64, f64)>; // (calories, protein, carbs, fats)

    // Raw Food Costs
    async fn create_raw_food_cost(&self, pattern: &str, cost: f64, currency: &str) -> AppResult<RawFoodCost>;
    async fn get_raw_food_cost(&self, id: RawFoodCostId) -> AppResult<Option<RawFoodCost>>;
    async fn list_raw_food_costs(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<RawFoodCost>, u64)>;
    async fn update_raw_food_cost(&self, id: RawFoodCostId, pattern: &str, cost: f64, currency: &str) -> AppResult<RawFoodCost>;
    async fn delete_raw_food_cost(&self, id: RawFoodCostId) -> AppResult<()>;
    async fn link_food_item_to_cost(&self, food_item_id: FoodItemId, cost_id: Option<RawFoodCostId>) -> AppResult<()>;
}
