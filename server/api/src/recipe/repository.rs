use crate::common::AppResult;
use crate::common::id::{UserId, RecipeId, IngredientId};
use super::models::{Ingredient, Recipe, RecipeIngredient, RecipeWithIngredients, FoodLog, IngredientPortion};

pub trait RecipeRepository: Send + Sync {
    // Ingredients
    async fn create_ingredient(&self, ing: Ingredient) -> AppResult<Ingredient>;
    async fn get_ingredient(&self, id: IngredientId) -> AppResult<Option<Ingredient>>;
    async fn search_ingredients(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<Ingredient>, u64)>;
    async fn add_ingredient_portion(&self, portion: IngredientPortion) -> AppResult<IngredientPortion>;
    async fn get_ingredient_portions(&self, ingredient_id: IngredientId) -> AppResult<Vec<IngredientPortion>>;

    // Recipes
    async fn create_recipe(&self, recipe: Recipe, ingredients: Vec<RecipeIngredient>) -> AppResult<RecipeWithIngredients>;
    async fn get_recipe(&self, id: RecipeId) -> AppResult<Option<RecipeWithIngredients>>;
    async fn list_recipes(&self, owner_id: Option<UserId>, page: u64, per_page: u64) -> AppResult<(Vec<Recipe>, u64)>;
    async fn get_recipes_by_ids(&self, ids: &[RecipeId]) -> AppResult<Vec<Recipe>>;

    // Food Logs
    async fn log_food(&self, log: FoodLog) -> AppResult<FoodLog>;
    async fn get_food_logs(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<FoodLog>, u64)>;
    async fn get_daily_macros(&self, user_id: UserId, date: chrono::NaiveDate) -> AppResult<(f64, f64, f64, f64)>; // (calories, protein, carbs, fats)
}
