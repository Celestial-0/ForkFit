use chrono::{Utc, NaiveDate};

use crate::common::AppResult;
use crate::common::id::{UserId, RecipeId, FoodItemId, FoodLogId};
use crate::common::error::AppError;

use super::models::{FoodItem, Recipe, RecipeFoodItem, RecipeWithFoodItems, FoodLog};
use super::repository::RecipeRepository;
use super::error::RecipeError;
use super::types::{
    CreateFoodItemRequest, CreateRecipeRequest, RecipeFoodItemDetailResponse,
    RecipeResponse, RecipeNutrients, RecipeDetailResponse, LogFoodRequest,
};

const ALLERGEN_TRIGGERS: &[(&str, &str)] = &[
    ("peanut", "Peanuts"),
    ("walnut", "Tree Nuts"),
    ("almond", "Tree Nuts"),
    ("cashew", "Tree Nuts"),
    ("milk", "Dairy"),
    ("cheese", "Dairy"),
    ("butter", "Dairy"),
    ("yogurt", "Dairy"),
    ("egg", "Eggs"),
    ("wheat", "Wheat"),
    ("soy", "Soy"),
    ("shrimp", "Shellfish"),
    ("crab", "Shellfish"),
    ("fish", "Fish"),
];

#[derive(Clone)]
pub struct RecipeService<R> {
    repo: R,
}

impl<R: RecipeRepository> RecipeService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    // Food Items
    pub async fn create_food_item(&self, req: CreateFoodItemRequest) -> AppResult<FoodItem> {
        if req.name.trim().is_empty() {
            return Err(RecipeError::ValidationError("Food item name cannot be empty".to_string()).into());
        }
        if req.calories_per_100g < 0.0 || req.protein_per_100g < 0.0 || req.carbs_per_100g < 0.0 || req.fat_per_100g < 0.0 {
            return Err(RecipeError::ValidationError("Nutrient values cannot be negative".to_string()).into());
        }
        if let Some(fiber) = req.fiber_per_100g {
            if fiber < 0.0 {
                return Err(RecipeError::ValidationError("Fiber cannot be negative".to_string()).into());
            }
        }
        if let Some(sodium) = req.sodium_mg_per_100g {
            if sodium < 0.0 {
                return Err(RecipeError::ValidationError("Sodium cannot be negative".to_string()).into());
            }
        }

        let food = FoodItem {
            id: FoodItemId::new(),
            name: req.name.trim().to_string(),
            description: req.description,
            calories_per_100g: req.calories_per_100g,
            protein_per_100g: req.protein_per_100g,
            carbs_per_100g: req.carbs_per_100g,
            fat_per_100g: req.fat_per_100g,
            fiber_per_100g: req.fiber_per_100g.unwrap_or(0.0),
            sodium_mg_per_100g: req.sodium_mg_per_100g.unwrap_or(0.0),
            micronutrients: req.micronutrients.unwrap_or_else(|| serde_json::json!({})),
            estimated_cost_per_100g: 0.0, // Calculated dynamically in repository
            price_currency: "INR".to_string(),
            barcode: req.barcode,
            is_verified: false,
            food_code: None,
            primary_source: None,
            raw_food_cost_id: req.raw_food_cost_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.create_food_item(food).await
    }

    pub async fn get_food_item(&self, id: FoodItemId) -> AppResult<FoodItem> {
        let food = self.repo.get_food_item(id).await?;
        food.ok_or_else(|| RecipeError::NotFound.into())
    }

    pub async fn search_food_items(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<FoodItem>, u64)> {
        self.repo.search_food_items(query, page, per_page).await
    }

    // Recipes
    pub async fn create_recipe(&self, owner_id: UserId, req: CreateRecipeRequest) -> AppResult<RecipeDetailResponse> {
        if req.title.trim().is_empty() {
            return Err(RecipeError::ValidationError("Recipe title cannot be empty".to_string()).into());
        }
        if req.servings < 0.10 {
            return Err(RecipeError::ValidationError("Servings must be at least 0.1".to_string()).into());
        }
        if req.food_items.is_empty() {
            return Err(RecipeError::ValidationError("Recipe must have at least one food item".to_string()).into());
        }

        let recipe_id = RecipeId::new();
        let mut recipe_food_items = Vec::new();

        for food_input in req.food_items {
            if food_input.quantity <= 0.0 {
                return Err(RecipeError::ValidationError("Food item quantity must be positive".to_string()).into());
            }
            if food_input.grams_equivalent <= 0.0 {
                return Err(RecipeError::ValidationError("Food item grams equivalent must be positive".to_string()).into());
            }

            // Verify food item exists
            let food = self.repo.get_food_item(food_input.food_item_id).await?;
            if food.is_none() {
                return Err(RecipeError::ValidationError(format!("Food item {} not found", food_input.food_item_id)).into());
            }

            recipe_food_items.push(RecipeFoodItem {
                recipe_id,
                food_item_id: food_input.food_item_id,
                quantity: food_input.quantity,
                unit: food_input.unit,
                grams_equivalent: food_input.grams_equivalent,
                notes: food_input.notes,
            });
        }

        let recipe = Recipe {
            id: recipe_id,
            owner_id: Some(owner_id),
            parent_recipe_id: req.parent_recipe_id,
            title: req.title.trim().to_string(),
            description: req.description,
            instructions: req.instructions,
            prep_time_minutes: req.prep_time_minutes,
            cook_time_minutes: req.cook_time_minutes,
            servings: req.servings,
            cuisine: req.cuisine,
            course: req.course,
            dietary_tags: req.dietary_tags,
            source_url: req.source_url,
            is_public: req.is_public,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = self.repo.create_recipe(recipe, recipe_food_items).await?;
        Ok(self.build_recipe_detail(result))
    }

    pub async fn get_recipe(&self, id: RecipeId) -> AppResult<RecipeDetailResponse> {
        let recipe = self.repo.get_recipe(id).await?;
        match recipe {
            Some(r) => Ok(self.build_recipe_detail(r)),
            None => Err(RecipeError::NotFound.into()),
        }
    }

    pub async fn list_recipes(&self, owner_id: Option<UserId>, page: u64, per_page: u64) -> AppResult<(Vec<Recipe>, u64)> {
        self.repo.list_recipes(owner_id, page, per_page).await
    }

    // Food Logs
    pub async fn log_food(&self, user_id: UserId, req: LogFoodRequest) -> AppResult<FoodLog> {
        if req.quantity <= 0.0 {
            return Err(RecipeError::ValidationError("Quantity must be positive".to_string()).into());
        }

        let (calories, protein, carbs, fats) = if let Some(recipe_id) = req.recipe_id {
            // Logged via Recipe association
            let recipe = self.repo.get_recipe(recipe_id).await?;
            let r_with_i = recipe.ok_or_else(|| AppError::BadRequest("Recipe not found".to_string()))?;
            let detail = self.build_recipe_detail(r_with_i);
            
            if req.unit.to_lowercase() == "servings" {
                (
                    detail.serving_nutrition.calories * req.quantity,
                    detail.serving_nutrition.protein * req.quantity,
                    detail.serving_nutrition.carbs * req.quantity,
                    detail.serving_nutrition.fat * req.quantity,
                )
            } else if req.unit.to_lowercase() == "grams" {
                let total_weight: f64 = detail.food_items.iter()
                    .map(|ri| ri.grams_equivalent)
                    .sum();
                
                if total_weight > 0.0 {
                    let fraction = req.quantity / total_weight;
                    (
                        detail.total_nutrition.calories * fraction,
                        detail.total_nutrition.protein * fraction,
                        detail.total_nutrition.carbs * fraction,
                        detail.total_nutrition.fat * fraction,
                    )
                } else {
                    return Err(RecipeError::ValidationError("Recipe has no weight context".to_string()).into());
                }
            } else {
                return Err(RecipeError::ValidationError("Invalid food log unit for recipe: must be 'servings' or 'grams'".to_string()).into());
            }
        } else if let Some(food_item_id) = req.food_item_id {
            // Logged via Food Item association (always assumed logged in grams)
            let food = self.repo.get_food_item(food_item_id).await?;
            let f = food.ok_or_else(|| AppError::BadRequest("Food item not found".to_string()))?;
            
            if req.unit.to_lowercase() == "grams" {
                let factor = req.quantity / 100.0;
                (
                    f.calories_per_100g * factor,
                    f.protein_per_100g * factor,
                    f.carbs_per_100g * factor,
                    f.fat_per_100g * factor,
                )
            } else {
                return Err(RecipeError::ValidationError("Invalid food log unit for food item: must be 'grams'".to_string()).into());
            }
        } else if let Some(ref name) = req.custom_food_name {
            // Logged via Custom Food Name
            if name.trim().is_empty() {
                return Err(RecipeError::ValidationError("Custom food name cannot be empty".to_string()).into());
            }
            (
                req.calories.ok_or_else(|| AppError::BadRequest("Calories required for custom food".to_string()))?,
                req.protein.ok_or_else(|| AppError::BadRequest("Protein required for custom food".to_string()))?,
                req.carbs.ok_or_else(|| AppError::BadRequest("Carbs required for custom food".to_string()))?,
                req.fats.ok_or_else(|| AppError::BadRequest("Fats required for custom food".to_string()))?,
            )
        } else {
            return Err(RecipeError::ValidationError("Food log must specify recipe_id, food_item_id, or custom_food_name".to_string()).into());
        };

        // Validate calculated or custom values are not negative
        if calories < 0.0 || protein < 0.0 || carbs < 0.0 || fats < 0.0 {
            return Err(RecipeError::ValidationError("Nutrients cannot be negative".to_string()).into());
        }

        // Generate micronutrients snapshot (scale values from food if logged via food)
        let micronutrients_snapshot = if let Some(food_item_id) = req.food_item_id {
            let food = self.repo.get_food_item(food_item_id).await?;
            if let Some(f) = food {
                let factor = req.quantity / 100.0;
                if let serde_json::Value::Object(mut map) = f.micronutrients.clone() {
                    for (_, val) in map.iter_mut() {
                        if let Some(num) = val.as_f64() {
                            *val = serde_json::json!(num * factor);
                        }
                    }
                    serde_json::Value::Object(map)
                } else {
                    f.micronutrients.clone()
                }
            } else {
                serde_json::json!({})
            }
        } else {
            serde_json::json!({})
        };

        let log = FoodLog {
            id: FoodLogId::new(),
            user_id,
            logged_at: req.logged_at.unwrap_or_else(Utc::now),
            meal_type: req.meal_type.to_lowercase(),
            recipe_id: req.recipe_id,
            food_item_id: req.food_item_id,
            custom_food_name: req.custom_food_name,
            quantity: req.quantity,
            unit: req.unit,
            calories,
            protein,
            carbs,
            fats,
            micronutrients_snapshot,
            created_at: Utc::now(),
        };

        self.repo.log_food(log).await
    }

    pub async fn get_food_logs(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<FoodLog>, u64)> {
        self.repo.get_food_logs(user_id, page, per_page).await
    }

    pub async fn get_daily_macros(&self, user_id: UserId, date: NaiveDate) -> AppResult<(f64, f64, f64, f64)> {
        self.repo.get_daily_macros(user_id, date).await
    }

    // Private helpers
    fn build_recipe_detail(&self, r_with_f: RecipeWithFoodItems) -> RecipeDetailResponse {
        let servings = r_with_f.recipe.servings;
        let mut total_cal = 0.0;
        let mut total_prot = 0.0;
        let mut total_carb = 0.0;
        let mut total_fat = 0.0;
        let mut total_fib = 0.0;
        let mut total_sod = 0.0;
        let mut total_cost = 0.0;
        let mut allergens = std::collections::HashSet::new();

        for food in &r_with_f.food_items {
            let factor = food.grams_equivalent / 100.0;
            total_cal += food.calories_per_100g * factor;
            total_prot += food.protein_per_100g * factor;
            total_carb += food.carbs_per_100g * factor;
            total_fat += food.fat_per_100g * factor;
            total_fib += food.fiber_per_100g * factor;
            total_sod += food.sodium_mg_per_100g * factor;
            total_cost += food.estimated_cost_per_100g * factor;

            // Allergen audit check
            let food_name_lower = food.name.to_lowercase();
            for &(trigger, allergen) in ALLERGEN_TRIGGERS {
                if food_name_lower.contains(trigger) {
                    allergens.insert(allergen.to_string());
                }
            }
        }

        let total_nutrition = RecipeNutrients {
            calories: total_cal,
            protein: total_prot,
            carbs: total_carb,
            fat: total_fat,
            fiber: total_fib,
            sodium: total_sod,
        };

        let serving_nutrition = RecipeNutrients {
            calories: total_cal / servings,
            protein: total_prot / servings,
            carbs: total_carb / servings,
            fat: total_fat / servings,
            fiber: total_fib / servings,
            sodium: total_sod / servings,
        };

        RecipeDetailResponse {
            recipe: RecipeResponse::from(r_with_f.recipe),
            food_items: r_with_f.food_items.into_iter().map(RecipeFoodItemDetailResponse::from).collect(),
            total_nutrition,
            serving_nutrition,
            total_estimated_cost: total_cost,
            serving_estimated_cost: total_cost / servings,
            detected_allergens: allergens.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::common::id::{UserId, RecipeId, FoodItemId, RawFoodCostId};
    use crate::recipe::models::{RecipeFoodItemDetail, FoodItemPortion, RawFoodCost};
    use crate::recipe::types::RecipeFoodItemInput;
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct MockRecipeRepository {
        food_items_db: Arc<Mutex<Vec<FoodItem>>>,
        recipes_db: Arc<Mutex<Vec<RecipeWithFoodItems>>>,
        logged_foods: Arc<Mutex<Vec<FoodLog>>>,
    }

    impl RecipeRepository for MockRecipeRepository {
        async fn create_food_item(&self, food: FoodItem) -> AppResult<FoodItem> {
            self.food_items_db.lock().unwrap().push(food.clone());
            Ok(food)
        }

        async fn get_food_item(&self, id: FoodItemId) -> AppResult<Option<FoodItem>> {
            let db = self.food_items_db.lock().unwrap();
            Ok(db.iter().find(|i| i.id == id).cloned())
        }

        async fn search_food_items(&self, _query: &str, _page: u64, _per_page: u64) -> AppResult<(Vec<FoodItem>, u64)> {
            Ok((vec![], 0))
        }

        async fn create_recipe(&self, recipe: Recipe, food_items: Vec<RecipeFoodItem>) -> AppResult<RecipeWithFoodItems> {
            let db = self.food_items_db.lock().unwrap();
            let details = food_items.into_iter().map(|ri| {
                let food = db.iter().find(|i| i.id == ri.food_item_id).unwrap();
                RecipeFoodItemDetail {
                    food_item_id: ri.food_item_id,
                    name: food.name.clone(),
                    quantity: ri.quantity,
                    unit: ri.unit,
                    grams_equivalent: ri.grams_equivalent,
                    calories_per_100g: food.calories_per_100g,
                    protein_per_100g: food.protein_per_100g,
                    carbs_per_100g: food.carbs_per_100g,
                    fat_per_100g: food.fat_per_100g,
                    fiber_per_100g: food.fiber_per_100g,
                    sodium_mg_per_100g: food.sodium_mg_per_100g,
                    estimated_cost_per_100g: food.estimated_cost_per_100g,
                    notes: ri.notes,
                }
            }).collect();
            let r_with_f = RecipeWithFoodItems { recipe, food_items: details };
            self.recipes_db.lock().unwrap().push(r_with_f.clone());
            Ok(r_with_f)
        }

        async fn get_recipe(&self, id: RecipeId) -> AppResult<Option<RecipeWithFoodItems>> {
            let db = self.recipes_db.lock().unwrap();
            Ok(db.iter().find(|r| r.recipe.id == id).cloned())
        }

        async fn list_recipes(&self, _owner_id: Option<UserId>, _page: u64, _per_page: u64) -> AppResult<(Vec<Recipe>, u64)> {
            Ok((vec![], 0))
        }

        async fn get_recipes_by_ids(&self, _ids: &[RecipeId]) -> AppResult<Vec<Recipe>> {
            Ok(vec![])
        }

        async fn log_food(&self, log: FoodLog) -> AppResult<FoodLog> {
            self.logged_foods.lock().unwrap().push(log.clone());
            Ok(log)
        }

        async fn get_food_logs(&self, _user_id: UserId, _page: u64, _per_page: u64) -> AppResult<(Vec<FoodLog>, u64)> {
            Ok((vec![], 0))
        }

        async fn get_daily_macros(&self, _user_id: UserId, _date: NaiveDate) -> AppResult<(f64, f64, f64, f64)> {
            Ok((0.0, 0.0, 0.0, 0.0))
        }

        async fn add_food_item_portion(&self, portion: FoodItemPortion) -> AppResult<FoodItemPortion> {
            Ok(portion)
        }

        async fn get_food_item_portions(&self, _food_item_id: FoodItemId) -> AppResult<Vec<FoodItemPortion>> {
            Ok(vec![])
        }

        // --- Raw Food Cost Mocks ---

        async fn create_raw_food_cost(&self, pattern: &str, cost: f64, currency: &str) -> AppResult<RawFoodCost> {
            Ok(RawFoodCost {
                id: RawFoodCostId::new(),
                food_pattern: pattern.to_string(),
                cost_per_100g: cost,
                price_currency: currency.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
        async fn get_raw_food_cost(&self, _id: RawFoodCostId) -> AppResult<Option<RawFoodCost>> {
            Ok(None)
        }
        async fn list_raw_food_costs(&self, _query: &str, _page: u64, _per_page: u64) -> AppResult<(Vec<RawFoodCost>, u64)> {
            Ok((vec![], 0))
        }
        async fn update_raw_food_cost(&self, id: RawFoodCostId, pattern: &str, cost: f64, currency: &str) -> AppResult<RawFoodCost> {
            Ok(RawFoodCost {
                id,
                food_pattern: pattern.to_string(),
                cost_per_100g: cost,
                price_currency: currency.to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
        async fn delete_raw_food_cost(&self, _id: RawFoodCostId) -> AppResult<()> {
            Ok(())
        }
        async fn link_food_item_to_cost(&self, _food_item_id: FoodItemId, _cost_id: Option<RawFoodCostId>) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_food_item_validation() {
        let repo = MockRecipeRepository::default();
        let service = RecipeService::new(repo);

        // Invalid empty name
        let res = service.create_food_item(CreateFoodItemRequest {
            name: "   ".to_string(),
            description: None,
            calories_per_100g: 100.0,
            protein_per_100g: 10.0,
            carbs_per_100g: 5.0,
            fat_per_100g: 2.0,
            fiber_per_100g: None,
            sodium_mg_per_100g: None,
            micronutrients: None,
            barcode: None,
            raw_food_cost_id: None,
        }).await;
        assert!(res.is_err());

        // Invalid negative macros
        let res = service.create_food_item(CreateFoodItemRequest {
            name: "Rice".to_string(),
            description: None,
            calories_per_100g: -50.0,
            protein_per_100g: 10.0,
            carbs_per_100g: 5.0,
            fat_per_100g: 2.0,
            fiber_per_100g: None,
            sodium_mg_per_100g: None,
            micronutrients: None,
            barcode: None,
            raw_food_cost_id: None,
        }).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_recipe_nutrition_and_cost_calculations() {
        let repo = MockRecipeRepository::default();
        let service = RecipeService::new(repo.clone());
        let user_id = UserId(Uuid::new_v4());

        // 1. Seed two food items
        let food1 = service.create_food_item(CreateFoodItemRequest {
            name: "Paneer".to_string(),
            description: None,
            calories_per_100g: 300.0,
            protein_per_100g: 20.0,
            carbs_per_100g: 4.0,
            fat_per_100g: 22.0,
            fiber_per_100g: Some(0.0),
            sodium_mg_per_100g: Some(10.0),
            micronutrients: None,
            barcode: None,
            raw_food_cost_id: None,
        }).await.unwrap();

        // Workaround mock cost setting
        {
            let mut db = repo.food_items_db.lock().unwrap();
            db[0].estimated_cost_per_100g = 40.0;
        }

        let food2 = service.create_food_item(CreateFoodItemRequest {
            name: "Peanut Butter".to_string(),
            description: None,
            calories_per_100g: 600.0,
            protein_per_100g: 25.0,
            carbs_per_100g: 20.0,
            fat_per_100g: 50.0,
            fiber_per_100g: Some(8.0),
            sodium_mg_per_100g: Some(100.0),
            micronutrients: None,
            barcode: None,
            raw_food_cost_id: None,
        }).await.unwrap();

        {
            let mut db = repo.food_items_db.lock().unwrap();
            db[1].estimated_cost_per_100g = 80.0;
        }

        // 2. Create recipe using Paneer (150g grams_equivalent) and Peanut Butter (50g grams_equivalent)
        let recipe_res = service.create_recipe(user_id, CreateRecipeRequest {
            parent_recipe_id: None,
            title: "High Protein Snack".to_string(),
            description: Some("Paneer with peanut butter".to_string()),
            instructions: vec!["Mix them together".to_string()],
            prep_time_minutes: Some(5),
            cook_time_minutes: Some(0),
            servings: 2.0,
            cuisine: Some("Fusion".to_string()),
            course: None,
            dietary_tags: vec![],
            source_url: None,
            is_public: true,
            food_items: vec![
                RecipeFoodItemInput {
                    food_item_id: food1.id,
                    quantity: 150.0,
                    unit: "g".to_string(),
                    grams_equivalent: 150.0,
                    notes: None,
                },
                RecipeFoodItemInput {
                    food_item_id: food2.id,
                    quantity: 50.0,
                    unit: "g".to_string(),
                    grams_equivalent: 50.0,
                    notes: None,
                },
            ],
        }).await.unwrap();

        // Calories math:
        // Paneer: 300 kcal/100g * 1.5 = 450 kcal
        // PB: 600 kcal/100g * 0.5 = 300 kcal
        // Total = 750 kcal. Serving (servings = 2) = 375 kcal
        assert_eq!(recipe_res.total_nutrition.calories, 750.0);
        assert_eq!(recipe_res.serving_nutrition.calories, 375.0);

        // Protein math:
        // Paneer: 20 * 1.5 = 30g
        // PB: 25 * 0.5 = 12.5g
        // Total = 42.5g. Serving = 21.25g
        assert_eq!(recipe_res.total_nutrition.protein, 42.5);
        assert_eq!(recipe_res.serving_nutrition.protein, 21.25);

        // Cost math:
        // Paneer: 40 * 1.5 = 60 INR
        // PB: 80 * 0.5 = 40 INR
        // Total = 100 INR. Serving = 50 INR
        assert_eq!(recipe_res.total_estimated_cost, 100.0);
        assert_eq!(recipe_res.serving_estimated_cost, 50.0);

        // Allergen audit:
        // Peanut Butter contains "peanut" trigger -> should detect "Peanuts"
    }
}

