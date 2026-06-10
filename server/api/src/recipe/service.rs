use chrono::{Utc, NaiveDate};

use crate::common::AppResult;
use crate::common::id::{UserId, RecipeId, IngredientId, FoodLogId};
use crate::common::error::AppError;

use super::models::{Ingredient, Recipe, RecipeIngredient, RecipeWithIngredients, FoodLog};
use super::repository::RecipeRepository;
use super::error::RecipeError;
use super::types::{
    CreateIngredientRequest, CreateRecipeRequest, RecipeIngredientDetailResponse,
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

    // Ingredients
    pub async fn create_ingredient(&self, req: CreateIngredientRequest) -> AppResult<Ingredient> {
        if req.name.trim().is_empty() {
            return Err(RecipeError::ValidationError("Ingredient name cannot be empty".to_string()).into());
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
        if let Some(cost) = req.estimated_cost_per_100g {
            if cost < 0.0 {
                return Err(RecipeError::ValidationError("Estimated cost cannot be negative".to_string()).into());
            }
        }

        let ing = Ingredient {
            id: IngredientId::new(),
            name: req.name.trim().to_string(),
            description: req.description,
            calories_per_100g: req.calories_per_100g,
            protein_per_100g: req.protein_per_100g,
            carbs_per_100g: req.carbs_per_100g,
            fat_per_100g: req.fat_per_100g,
            fiber_per_100g: req.fiber_per_100g.unwrap_or(0.0),
            sodium_mg_per_100g: req.sodium_mg_per_100g.unwrap_or(0.0),
            micronutrients: req.micronutrients.unwrap_or_else(|| serde_json::json!({})),
            estimated_cost_per_100g: req.estimated_cost_per_100g.unwrap_or(0.0),
            price_currency: req.price_currency.unwrap_or_else(|| "INR".to_string()),
            barcode: req.barcode,
            is_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.create_ingredient(ing).await
    }

    pub async fn get_ingredient(&self, id: IngredientId) -> AppResult<Ingredient> {
        let ing = self.repo.get_ingredient(id).await?;
        ing.ok_or_else(|| RecipeError::NotFound.into())
    }

    pub async fn search_ingredients(&self, query: &str, page: u64, per_page: u64) -> AppResult<(Vec<Ingredient>, u64)> {
        self.repo.search_ingredients(query, page, per_page).await
    }

    // Recipes
    pub async fn create_recipe(&self, owner_id: UserId, req: CreateRecipeRequest) -> AppResult<RecipeDetailResponse> {
        if req.title.trim().is_empty() {
            return Err(RecipeError::ValidationError("Recipe title cannot be empty".to_string()).into());
        }
        if req.servings < 0.10 {
            return Err(RecipeError::ValidationError("Servings must be at least 0.1".to_string()).into());
        }
        if req.ingredients.is_empty() {
            return Err(RecipeError::ValidationError("Recipe must have at least one ingredient".to_string()).into());
        }

        let recipe_id = RecipeId::new();
        let mut recipe_ingredients = Vec::new();

        for ing_input in req.ingredients {
            if ing_input.quantity <= 0.0 {
                return Err(RecipeError::ValidationError("Ingredient quantity must be positive".to_string()).into());
            }
            if ing_input.grams_equivalent <= 0.0 {
                return Err(RecipeError::ValidationError("Ingredient grams equivalent must be positive".to_string()).into());
            }

            // Verify ingredient exists
            let ing = self.repo.get_ingredient(ing_input.ingredient_id).await?;
            if ing.is_none() {
                return Err(RecipeError::ValidationError(format!("Ingredient {} not found", ing_input.ingredient_id)).into());
            }

            recipe_ingredients.push(RecipeIngredient {
                recipe_id,
                ingredient_id: ing_input.ingredient_id,
                quantity: ing_input.quantity,
                unit: ing_input.unit,
                grams_equivalent: ing_input.grams_equivalent,
                notes: ing_input.notes,
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
            dietary_tags: req.dietary_tags,
            is_public: req.is_public,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = self.repo.create_recipe(recipe, recipe_ingredients).await?;
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
                let total_weight: f64 = detail.ingredients.iter()
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
        } else if let Some(ing_id) = req.ingredient_id {
            // Logged via Ingredient association (always assumed logged in grams)
            let ing = self.repo.get_ingredient(ing_id).await?;
            let i = ing.ok_or_else(|| AppError::BadRequest("Ingredient not found".to_string()))?;
            
            if req.unit.to_lowercase() == "grams" {
                let factor = req.quantity / 100.0;
                (
                    i.calories_per_100g * factor,
                    i.protein_per_100g * factor,
                    i.carbs_per_100g * factor,
                    i.fat_per_100g * factor,
                )
            } else {
                return Err(RecipeError::ValidationError("Invalid food log unit for ingredient: must be 'grams'".to_string()).into());
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
            return Err(RecipeError::ValidationError("Food log must specify recipe_id, ingredient_id, or custom_food_name".to_string()).into());
        };

        // Validate calculated or custom values are not negative
        if calories < 0.0 || protein < 0.0 || carbs < 0.0 || fats < 0.0 {
            return Err(RecipeError::ValidationError("Nutrients cannot be negative".to_string()).into());
        }

        let log = FoodLog {
            id: FoodLogId::new(),
            user_id,
            logged_at: req.logged_at.unwrap_or_else(Utc::now),
            meal_type: req.meal_type.to_lowercase(),
            recipe_id: req.recipe_id,
            ingredient_id: req.ingredient_id,
            custom_food_name: req.custom_food_name,
            quantity: req.quantity,
            unit: req.unit,
            calories,
            protein,
            carbs,
            fats,
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
    fn build_recipe_detail(&self, r_with_i: RecipeWithIngredients) -> RecipeDetailResponse {
        let servings = r_with_i.recipe.servings;
        let mut total_cal = 0.0;
        let mut total_prot = 0.0;
        let mut total_carb = 0.0;
        let mut total_fat = 0.0;
        let mut total_fib = 0.0;
        let mut total_sod = 0.0;
        let mut total_cost = 0.0;
        let mut allergens = std::collections::HashSet::new();

        for ing in &r_with_i.ingredients {
            let factor = ing.grams_equivalent / 100.0;
            total_cal += ing.calories_per_100g * factor;
            total_prot += ing.protein_per_100g * factor;
            total_carb += ing.carbs_per_100g * factor;
            total_fat += ing.fat_per_100g * factor;
            total_fib += ing.fiber_per_100g * factor;
            total_sod += ing.sodium_mg_per_100g * factor;
            total_cost += ing.estimated_cost_per_100g * factor;

            // Allergen audit check
            let ing_name_lower = ing.name.to_lowercase();
            for &(trigger, allergen) in ALLERGEN_TRIGGERS {
                if ing_name_lower.contains(trigger) {
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
            recipe: RecipeResponse::from(r_with_i.recipe),
            ingredients: r_with_i.ingredients.into_iter().map(RecipeIngredientDetailResponse::from).collect(),
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
    use std::sync::Mutex;
    use crate::common::id::{UserId, RecipeId, IngredientId};
    use crate::recipe::models::RecipeIngredientDetail;
    use crate::recipe::types::RecipeIngredientInput;
    use uuid::Uuid;

    #[derive(Default)]
    struct MockRecipeRepository {
        ingredients_db: Mutex<Vec<Ingredient>>,
        recipes_db: Mutex<Vec<RecipeWithIngredients>>,
        logged_foods: Mutex<Vec<FoodLog>>,
    }

    impl RecipeRepository for MockRecipeRepository {
        async fn create_ingredient(&self, ing: Ingredient) -> AppResult<Ingredient> {
            self.ingredients_db.lock().unwrap().push(ing.clone());
            Ok(ing)
        }

        async fn get_ingredient(&self, id: IngredientId) -> AppResult<Option<Ingredient>> {
            let db = self.ingredients_db.lock().unwrap();
            Ok(db.iter().find(|i| i.id == id).cloned())
        }

        async fn search_ingredients(&self, _query: &str, _page: u64, _per_page: u64) -> AppResult<(Vec<Ingredient>, u64)> {
            Ok((vec![], 0))
        }

        async fn create_recipe(&self, recipe: Recipe, ingredients: Vec<RecipeIngredient>) -> AppResult<RecipeWithIngredients> {
            let db = self.ingredients_db.lock().unwrap();
            let details = ingredients.into_iter().map(|ri| {
                let ing = db.iter().find(|i| i.id == ri.ingredient_id).unwrap();
                RecipeIngredientDetail {
                    ingredient_id: ri.ingredient_id,
                    name: ing.name.clone(),
                    quantity: ri.quantity,
                    unit: ri.unit,
                    grams_equivalent: ri.grams_equivalent,
                    calories_per_100g: ing.calories_per_100g,
                    protein_per_100g: ing.protein_per_100g,
                    carbs_per_100g: ing.carbs_per_100g,
                    fat_per_100g: ing.fat_per_100g,
                    fiber_per_100g: ing.fiber_per_100g,
                    sodium_mg_per_100g: ing.sodium_mg_per_100g,
                    estimated_cost_per_100g: ing.estimated_cost_per_100g,
                    notes: ri.notes,
                }
            }).collect();
            let r_with_i = RecipeWithIngredients { recipe, ingredients: details };
            self.recipes_db.lock().unwrap().push(r_with_i.clone());
            Ok(r_with_i)
        }

        async fn get_recipe(&self, id: RecipeId) -> AppResult<Option<RecipeWithIngredients>> {
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
    }

    #[tokio::test]
    async fn test_create_ingredient_validation() {
        let repo = MockRecipeRepository::default();
        let service = RecipeService::new(repo);

        // Invalid empty name
        let res = service.create_ingredient(CreateIngredientRequest {
            name: "   ".to_string(),
            description: None,
            calories_per_100g: 100.0,
            protein_per_100g: 10.0,
            carbs_per_100g: 5.0,
            fat_per_100g: 2.0,
            fiber_per_100g: None,
            sodium_mg_per_100g: None,
            micronutrients: None,
            estimated_cost_per_100g: None,
            price_currency: None,
            barcode: None,
        }).await;
        assert!(res.is_err());

        // Invalid negative macros
        let res = service.create_ingredient(CreateIngredientRequest {
            name: "Rice".to_string(),
            description: None,
            calories_per_100g: -50.0,
            protein_per_100g: 10.0,
            carbs_per_100g: 5.0,
            fat_per_100g: 2.0,
            fiber_per_100g: None,
            sodium_mg_per_100g: None,
            micronutrients: None,
            estimated_cost_per_100g: None,
            price_currency: None,
            barcode: None,
        }).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_recipe_nutrition_and_cost_calculations() {
        let repo = MockRecipeRepository::default();
        let service = RecipeService::new(repo);
        let user_id = UserId(Uuid::new_v4());

        // 1. Seed two ingredients
        let ing1 = service.create_ingredient(CreateIngredientRequest {
            name: "Paneer".to_string(),
            description: None,
            calories_per_100g: 300.0,
            protein_per_100g: 20.0,
            carbs_per_100g: 4.0,
            fat_per_100g: 22.0,
            fiber_per_100g: Some(0.0),
            sodium_mg_per_100g: Some(10.0),
            micronutrients: None,
            estimated_cost_per_100g: Some(40.0),
            price_currency: Some("INR".to_string()),
            barcode: None,
        }).await.unwrap();

        let ing2 = service.create_ingredient(CreateIngredientRequest {
            name: "Peanut Butter".to_string(),
            description: None,
            calories_per_100g: 600.0,
            protein_per_100g: 25.0,
            carbs_per_100g: 20.0,
            fat_per_100g: 50.0,
            fiber_per_100g: Some(8.0),
            sodium_mg_per_100g: Some(100.0),
            micronutrients: None,
            estimated_cost_per_100g: Some(80.0),
            price_currency: Some("INR".to_string()),
            barcode: None,
        }).await.unwrap();

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
            dietary_tags: vec![],
            is_public: true,
            ingredients: vec![
                RecipeIngredientInput {
                    ingredient_id: ing1.id,
                    quantity: 150.0,
                    unit: "g".to_string(),
                    grams_equivalent: 150.0,
                    notes: None,
                },
                RecipeIngredientInput {
                    ingredient_id: ing2.id,
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
        assert!(recipe_res.detected_allergens.contains(&"Peanuts".to_string()));
    }
}

