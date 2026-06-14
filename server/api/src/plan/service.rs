use crate::common::AppResult;
use crate::common::id::{UserId, MealPlanId, MealPlanItemId, PantryItemId, ShoppingListId, ShoppingListItemId};
use crate::plan::models::{MealPlan, MealPlanItem, PantryItem, ShoppingList, ShoppingListItem};
use crate::plan::repository::PlanRepository;
use crate::recipe::service::RecipeService;
use crate::recipe::repository::RecipeRepository;
use crate::plan::error::PlanError;
use crate::plan::types::{
    CreateMealPlanRequest, MealPlanDetailResponse, MealPlanResponse, MealPlanItemResponse,
    CreatePantryItemRequest, PantryItemResponse, UpdatePantryItemRequest,
    CreateShoppingListRequest, ShoppingListDetailResponse, ShoppingListResponse, ShoppingListItemResponse,
};
use chrono::Utc;

#[derive(Clone)]
pub struct PlanService<R, RR> {
    repo: R,
    recipe_service: RecipeService<RR>,
}

impl<R: PlanRepository, RR: RecipeRepository> PlanService<R, RR> {
    pub fn new(repo: R, recipe_service: RecipeService<RR>) -> Self {
        Self { repo, recipe_service }
    }

    // Meal Plans
    pub async fn create_meal_plan(&self, user_id: UserId, req: CreateMealPlanRequest) -> AppResult<MealPlanDetailResponse> {
        if req.start_date > req.end_date {
            return Err(PlanError::ValidationError("start_date must be before or equal to end_date".to_string()).into());
        }
        if req.items.is_empty() {
            return Err(PlanError::ValidationError("meal plan must contain at least one item".to_string()).into());
        }

        let plan_id = MealPlanId::new();
        let mut items = Vec::new();

        for item_input in req.items {
            if item_input.servings < 0.10 {
                return Err(PlanError::ValidationError("servings must be at least 0.1".to_string()).into());
            }
            if item_input.recipe_id.is_none() && item_input.food_item_id.is_none() && item_input.custom_food_name.is_none() {
                return Err(PlanError::ValidationError("meal plan item must specify recipe_id, food_item_id, or custom_food_name".to_string()).into());
            }

            let meal_type_lower = item_input.meal_type.to_lowercase();
            if meal_type_lower != "breakfast" && meal_type_lower != "lunch" && meal_type_lower != "dinner" && meal_type_lower != "snack" {
                return Err(PlanError::ValidationError("meal_type must be breakfast, lunch, dinner, or snack".to_string()).into());
            }

            // Verify if recipe exists
            if let Some(recipe_id) = item_input.recipe_id {
                if self.recipe_service.get_recipe(recipe_id).await.is_err() {
                    return Err(PlanError::ValidationError(format!("recipe {} not found", recipe_id)).into());
                }
            }
            // Verify if food item exists
            if let Some(food_item_id) = item_input.food_item_id {
                if self.recipe_service.get_food_item(food_item_id).await.is_err() {
                    return Err(PlanError::ValidationError(format!("food item {} not found", food_item_id)).into());
                }
            }

            items.push(MealPlanItem {
                id: MealPlanItemId::new(),
                meal_plan_id: plan_id,
                planned_date: item_input.planned_date,
                meal_type: meal_type_lower,
                recipe_id: item_input.recipe_id,
                food_item_id: item_input.food_item_id,
                custom_food_name: item_input.custom_food_name,
                servings: item_input.servings,
                consumed: false,
                created_at: Utc::now(),
            });
        }

        let plan = MealPlan {
            id: plan_id,
            user_id,
            name: req.name,
            start_date: req.start_date,
            end_date: req.end_date,
            is_active: req.is_active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let (inserted_plan, inserted_items) = self.repo.create_meal_plan(plan, items).await?;

        Ok(MealPlanDetailResponse {
            plan: MealPlanResponse::from(inserted_plan),
            items: inserted_items.into_iter().map(MealPlanItemResponse::from).collect(),
        })
    }

    pub async fn get_meal_plan(&self, id: MealPlanId) -> AppResult<MealPlanDetailResponse> {
        let res = self.repo.get_meal_plan(id).await?;
        let (plan, items) = res.ok_or_else(|| PlanError::NotFound)?;
        Ok(MealPlanDetailResponse {
            plan: MealPlanResponse::from(plan),
            items: items.into_iter().map(MealPlanItemResponse::from).collect(),
        })
    }

    pub async fn get_active_meal_plan(&self, user_id: UserId) -> AppResult<MealPlanDetailResponse> {
        let res = self.repo.get_active_meal_plan(user_id).await?;
        let (plan, items) = res.ok_or_else(|| PlanError::NotFound)?;
        Ok(MealPlanDetailResponse {
            plan: MealPlanResponse::from(plan),
            items: items.into_iter().map(MealPlanItemResponse::from).collect(),
        })
    }

    pub async fn list_meal_plans(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<MealPlanResponse>, u64)> {
        let (plans, total) = self.repo.list_meal_plans(user_id, page, per_page).await?;
        Ok((plans.into_iter().map(MealPlanResponse::from).collect(), total))
    }

    pub async fn set_meal_plan_active(&self, user_id: UserId, id: MealPlanId, active: bool) -> AppResult<()> {
        let plan_res = self.repo.get_meal_plan(id).await?;
        let (plan, _) = plan_res.ok_or_else(|| PlanError::NotFound)?;
        if plan.user_id != user_id {
            return Err(crate::common::error::AppError::Forbidden);
        }
        self.repo.set_meal_plan_active(user_id, id, active).await
    }

    pub async fn update_meal_plan_item_consumed(&self, user_id: UserId, item_id: MealPlanItemId, consumed: bool) -> AppResult<MealPlanItemResponse> {
        let item = self.repo.get_meal_plan_item(item_id).await?
            .ok_or_else(|| PlanError::NotFound)?;

        let plan_res = self.repo.get_meal_plan(item.meal_plan_id).await?;
        let (plan, _) = plan_res.ok_or_else(|| PlanError::NotFound)?;
        if plan.user_id != user_id {
            return Err(crate::common::error::AppError::Forbidden);
        }

        // Trigger Food Log creation if transitioning to consumed
        if !item.consumed && consumed {
            let log_req = crate::recipe::types::LogFoodRequest {
                logged_at: Some(Utc::now()),
                meal_type: item.meal_type.clone(),
                recipe_id: item.recipe_id,
                food_item_id: item.food_item_id,
                custom_food_name: item.custom_food_name.clone(),
                quantity: if item.recipe_id.is_some() {
                    item.servings
                } else if item.food_item_id.is_some() {
                    item.servings * 100.0 // 1 serving = 100g for food items
                } else {
                    item.servings
                },
                unit: if item.recipe_id.is_some() {
                    "servings".to_string()
                } else if item.food_item_id.is_some() {
                    "grams".to_string()
                } else {
                    "servings".to_string()
                },
                calories: if item.recipe_id.is_none() && item.food_item_id.is_none() { Some(0.0) } else { None },
                protein: if item.recipe_id.is_none() && item.food_item_id.is_none() { Some(0.0) } else { None },
                carbs: if item.recipe_id.is_none() && item.food_item_id.is_none() { Some(0.0) } else { None },
                fats: if item.recipe_id.is_none() && item.food_item_id.is_none() { Some(0.0) } else { None },
            };

            self.recipe_service.log_food(user_id, log_req).await?;
        }

        let updated = self.repo.update_meal_plan_item_consumed(item_id, consumed).await?
            .ok_or_else(|| PlanError::NotFound)?;

        Ok(MealPlanItemResponse::from(updated))
    }

    // Pantry
    pub async fn get_pantry_item(&self, user_id: UserId, id: PantryItemId) -> AppResult<PantryItemResponse> {
        let item = self.repo.get_pantry_item(id).await?
            .ok_or_else(|| PlanError::NotFound)?;
        if item.user_id != user_id {
            return Err(crate::common::error::AppError::Forbidden);
        }
        Ok(PantryItemResponse::from(item))
    }

    pub async fn list_pantry_items(&self, user_id: UserId) -> AppResult<Vec<PantryItemResponse>> {
        let items = self.repo.list_pantry_items(user_id).await?;
        Ok(items.into_iter().map(PantryItemResponse::from).collect())
    }

    pub async fn create_pantry_item(&self, user_id: UserId, req: CreatePantryItemRequest) -> AppResult<PantryItemResponse> {
        if req.quantity <= 0.0 {
            return Err(PlanError::ValidationError("quantity must be greater than zero".to_string()).into());
        }
        if req.unit.trim().is_empty() {
            return Err(PlanError::ValidationError("unit cannot be empty".to_string()).into());
        }

        // Verify food item exists
        if self.recipe_service.get_food_item(req.food_item_id).await.is_err() {
            return Err(PlanError::ValidationError(format!("food item {} not found", req.food_item_id)).into());
        }

        let item = PantryItem {
            id: PantryItemId::new(),
            user_id,
            food_item_id: req.food_item_id,
            quantity: req.quantity,
            unit: req.unit.trim().to_string(),
            expires_at: req.expires_at,
            purchased_at: req.purchased_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let inserted = self.repo.create_pantry_item(item).await?;
        Ok(PantryItemResponse::from(inserted))
    }

    pub async fn update_pantry_item(&self, user_id: UserId, id: PantryItemId, req: UpdatePantryItemRequest) -> AppResult<PantryItemResponse> {
        if req.quantity <= 0.0 {
            return Err(PlanError::ValidationError("quantity must be greater than zero".to_string()).into());
        }

        let item = self.repo.get_pantry_item(id).await?
            .ok_or_else(|| PlanError::NotFound)?;
        if item.user_id != user_id {
            return Err(crate::common::error::AppError::Forbidden);
        }

        let updated = self.repo.update_pantry_item(id, req.quantity, req.expires_at).await?;
        Ok(PantryItemResponse::from(updated))
    }

    pub async fn delete_pantry_item(&self, user_id: UserId, id: PantryItemId) -> AppResult<()> {
        let item = self.repo.get_pantry_item(id).await?
            .ok_or_else(|| PlanError::NotFound)?;
        if item.user_id != user_id {
            return Err(crate::common::error::AppError::Forbidden);
        }
        self.repo.delete_pantry_item(id).await
    }

    // Shopping Lists
    pub async fn create_shopping_list(&self, user_id: UserId, req: CreateShoppingListRequest) -> AppResult<ShoppingListDetailResponse> {
        if req.name.trim().is_empty() {
            return Err(PlanError::ValidationError("shopping list name cannot be empty".to_string()).into());
        }
        if req.items.is_empty() {
            return Err(PlanError::ValidationError("shopping list must contain at least one item".to_string()).into());
        }

        let list_id = ShoppingListId::new();
        let mut items = Vec::new();

        for item_input in req.items {
            if item_input.quantity <= 0.0 {
                return Err(PlanError::ValidationError("quantity must be greater than zero".to_string()).into());
            }
            if item_input.unit.trim().is_empty() {
                return Err(PlanError::ValidationError("unit cannot be empty".to_string()).into());
            }
            if item_input.food_item_id.is_none() && item_input.custom_item_name.is_none() {
                return Err(PlanError::ValidationError("shopping list item must specify food_item_id or custom_item_name".to_string()).into());
            }

            if let Some(food_item_id) = item_input.food_item_id {
                if self.recipe_service.get_food_item(food_item_id).await.is_err() {
                    return Err(PlanError::ValidationError(format!("food item {} not found", food_item_id)).into());
                }
            }

            items.push(ShoppingListItem {
                id: ShoppingListItemId::new(),
                shopping_list_id: list_id,
                food_item_id: item_input.food_item_id,
                custom_item_name: item_input.custom_item_name,
                quantity: item_input.quantity,
                unit: item_input.unit.trim().to_string(),
                is_acquired: false,
                category: item_input.category.unwrap_or_else(|| "Other".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        let list = ShoppingList {
            id: list_id,
            user_id,
            name: req.name.trim().to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let (inserted_list, inserted_items) = self.repo.create_shopping_list(list, items).await?;

        Ok(ShoppingListDetailResponse {
            list: ShoppingListResponse::from(inserted_list),
            items: inserted_items.into_iter().map(ShoppingListItemResponse::from).collect(),
        })
    }

    pub async fn get_shopping_list(&self, user_id: UserId, id: ShoppingListId) -> AppResult<ShoppingListDetailResponse> {
        let res = self.repo.get_shopping_list(id).await?;
        let (list, items) = res.ok_or_else(|| PlanError::NotFound)?;
        if list.user_id != user_id {
            return Err(crate::common::error::AppError::Forbidden);
        }
        Ok(ShoppingListDetailResponse {
            list: ShoppingListResponse::from(list),
            items: items.into_iter().map(ShoppingListItemResponse::from).collect(),
        })
    }

    pub async fn list_shopping_lists(&self, user_id: UserId) -> AppResult<Vec<ShoppingListResponse>> {
        let lists = self.repo.list_shopping_lists(user_id).await?;
        Ok(lists.into_iter().map(ShoppingListResponse::from).collect())
    }

    pub async fn update_shopping_list_item_acquired(
        &self,
        user_id: UserId,
        list_id: ShoppingListId,
        item_id: ShoppingListItemId,
        is_acquired: bool,
    ) -> AppResult<()> {
        let list_res = self.repo.get_shopping_list(list_id).await?;
        let (list, _) = list_res.ok_or_else(|| PlanError::NotFound)?;
        if list.user_id != user_id {
            return Err(crate::common::error::AppError::Forbidden);
        }

        self.repo.update_shopping_list_item_acquired(list_id, item_id, is_acquired).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use chrono::NaiveDate;
    use crate::common::id::{UserId, MealPlanId, MealPlanItemId, PantryItemId, ShoppingListId, ShoppingListItemId, RecipeId, FoodItemId, RawFoodCostId};
    use crate::plan::types::MealPlanItemInput;
    use crate::recipe::models::{FoodItem, Recipe, RecipeFoodItem, RecipeWithFoodItems, FoodLog, FoodItemPortion, RawFoodCost};
    use crate::recipe::repository::RecipeRepository;
    use uuid::Uuid;

    #[derive(Default)]
    struct MockPlanRepository {
        plans: Mutex<Vec<(MealPlan, Vec<MealPlanItem>)>>,
        pantry: Mutex<Vec<PantryItem>>,
        lists: Mutex<Vec<(ShoppingList, Vec<ShoppingListItem>)>>,
    }

    impl PlanRepository for MockPlanRepository {
        async fn create_meal_plan(&self, plan: MealPlan, items: Vec<MealPlanItem>) -> AppResult<(MealPlan, Vec<MealPlanItem>)> {
            if plan.is_active {
                for (p, _) in self.plans.lock().unwrap().iter_mut() {
                    if p.user_id == plan.user_id {
                        p.is_active = false;
                    }
                }
            }
            self.plans.lock().unwrap().push((plan.clone(), items.clone()));
            Ok((plan, items))
        }

        async fn get_meal_plan(&self, id: MealPlanId) -> AppResult<Option<(MealPlan, Vec<MealPlanItem>)>> {
            let db = self.plans.lock().unwrap();
            Ok(db.iter().find(|(p, _)| p.id == id).cloned())
        }

        async fn get_active_meal_plan(&self, user_id: UserId) -> AppResult<Option<(MealPlan, Vec<MealPlanItem>)>> {
            let db = self.plans.lock().unwrap();
            Ok(db.iter().find(|(p, _)| p.user_id == user_id && p.is_active).cloned())
        }

        async fn get_meal_plan_item(&self, id: MealPlanItemId) -> AppResult<Option<MealPlanItem>> {
            let db = self.plans.lock().unwrap();
            for (_, items) in db.iter() {
                if let Some(item) = items.iter().find(|i| i.id == id) {
                    return Ok(Some(item.clone()));
                }
            }
            Ok(None)
        }

        async fn list_meal_plans(&self, user_id: UserId, _page: u64, _per_page: u64) -> AppResult<(Vec<MealPlan>, u64)> {
            let db = self.plans.lock().unwrap();
            let matched: Vec<MealPlan> = db.iter().filter(|(p, _)| p.user_id == user_id).map(|(p, _)| p.clone()).collect();
            let len = matched.len() as u64;
            Ok((matched, len))
        }

        async fn set_meal_plan_active(&self, user_id: UserId, id: MealPlanId, active: bool) -> AppResult<()> {
            let mut db = self.plans.lock().unwrap();
            if active {
                for (p, _) in db.iter_mut() {
                    if p.user_id == user_id {
                        p.is_active = false;
                    }
                }
            }
            if let Some((p, _)) = db.iter_mut().find(|(p, _)| p.id == id) {
                p.is_active = active;
            }
            Ok(())
        }

        async fn update_meal_plan_item_consumed(&self, item_id: MealPlanItemId, consumed: bool) -> AppResult<Option<MealPlanItem>> {
            let mut db = self.plans.lock().unwrap();
            for (_, items) in db.iter_mut() {
                if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                    item.consumed = consumed;
                    return Ok(Some(item.clone()));
                }
            }
            Ok(None)
        }

        async fn get_pantry_item(&self, id: PantryItemId) -> AppResult<Option<PantryItem>> {
            let db = self.pantry.lock().unwrap();
            Ok(db.iter().find(|i| i.id == id).cloned())
        }

        async fn list_pantry_items(&self, user_id: UserId) -> AppResult<Vec<PantryItem>> {
            let db = self.pantry.lock().unwrap();
            Ok(db.iter().filter(|i| i.user_id == user_id).cloned().collect())
        }

        async fn create_pantry_item(&self, item: PantryItem) -> AppResult<PantryItem> {
            self.pantry.lock().unwrap().push(item.clone());
            Ok(item)
        }

        async fn update_pantry_item(&self, id: PantryItemId, quantity: f64, expires_at: Option<NaiveDate>) -> AppResult<PantryItem> {
            let mut db = self.pantry.lock().unwrap();
            let item = db.iter_mut().find(|i| i.id == id).unwrap();
            item.quantity = quantity;
            item.expires_at = expires_at;
            Ok(item.clone())
        }

        async fn delete_pantry_item(&self, id: PantryItemId) -> AppResult<()> {
            let mut db = self.pantry.lock().unwrap();
            db.retain(|i| i.id != id);
            Ok(())
        }

        async fn create_shopping_list(&self, list: ShoppingList, items: Vec<ShoppingListItem>) -> AppResult<(ShoppingList, Vec<ShoppingListItem>)> {
            self.lists.lock().unwrap().push((list.clone(), items.clone()));
            Ok((list, items))
        }

        async fn get_shopping_list(&self, id: ShoppingListId) -> AppResult<Option<(ShoppingList, Vec<ShoppingListItem>)>> {
            let db = self.lists.lock().unwrap();
            Ok(db.iter().find(|(l, _)| l.id == id).cloned())
        }

        async fn list_shopping_lists(&self, user_id: UserId) -> AppResult<Vec<ShoppingList>> {
            let db = self.lists.lock().unwrap();
            Ok(db.iter().filter(|(l, _)| l.user_id == user_id).map(|(l, _)| l.clone()).collect())
        }

        async fn update_shopping_list_item_acquired(&self, list_id: ShoppingListId, item_id: ShoppingListItemId, is_acquired: bool) -> AppResult<()> {
            let mut db = self.lists.lock().unwrap();
            if let Some((_, items)) = db.iter_mut().find(|(l, _)| l.id == list_id) {
                if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                    item.is_acquired = is_acquired;
                }
            }
            Ok(())
        }
    }

    #[derive(Default)]
    struct MockRecipeRepository {
        food_items: Mutex<Vec<FoodItem>>,
        logged: Mutex<Vec<FoodLog>>,
    }

    impl RecipeRepository for MockRecipeRepository {
        async fn create_food_item(&self, food: FoodItem) -> AppResult<FoodItem> {
            self.food_items.lock().unwrap().push(food.clone());
            Ok(food)
        }
        async fn get_food_item(&self, id: FoodItemId) -> AppResult<Option<FoodItem>> {
            let db = self.food_items.lock().unwrap();
            Ok(db.iter().find(|i| i.id == id).cloned())
        }
        async fn search_food_items(&self, _query: &str, _page: u64, _per_page: u64) -> AppResult<(Vec<FoodItem>, u64)> {
            Ok((vec![], 0))
        }
        async fn create_recipe(&self, _recipe: Recipe, _food_items: Vec<RecipeFoodItem>) -> AppResult<RecipeWithFoodItems> {
            Err(crate::common::error::AppError::NotFound)
        }
        async fn get_recipe(&self, _id: RecipeId) -> AppResult<Option<RecipeWithFoodItems>> {
            Ok(None)
        }
        async fn list_recipes(&self, _owner_id: Option<UserId>, _page: u64, _per_page: u64) -> AppResult<(Vec<Recipe>, u64)> {
            Ok((vec![], 0))
        }
        async fn get_recipes_by_ids(&self, _ids: &[RecipeId]) -> AppResult<Vec<Recipe>> {
            Ok(vec![])
        }
        async fn log_food(&self, log: FoodLog) -> AppResult<FoodLog> {
            self.logged.lock().unwrap().push(log.clone());
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
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
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
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
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
    async fn test_create_meal_plan_validations() {
        let plan_repo = MockPlanRepository::default();
        let recipe_repo = MockRecipeRepository::default();
        let recipe_service = RecipeService::new(recipe_repo);
        let service = PlanService::new(plan_repo, recipe_service);
        let user_id = UserId(Uuid::new_v4());

        // 1. Invalid date range (start > end)
        let res = service.create_meal_plan(user_id, CreateMealPlanRequest {
            name: Some("Plan 1".to_string()),
            start_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            is_active: true,
            items: vec![],
        }).await;
        assert!(res.is_err());

        // 2. Empty items list
        let res = service.create_meal_plan(user_id, CreateMealPlanRequest {
            name: Some("Plan 1".to_string()),
            start_date: NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            is_active: true,
            items: vec![],
        }).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_active_plan_deactivates_others() {
        let plan_repo = MockPlanRepository::default();
        let recipe_repo = MockRecipeRepository::default();
        let recipe_service = RecipeService::new(recipe_repo);
        let service = PlanService::new(plan_repo, recipe_service);
        let user_id = UserId(Uuid::new_v4());

        // Seed an active plan
        let plan1 = service.create_meal_plan(user_id, CreateMealPlanRequest {
            name: Some("Plan 1".to_string()),
            start_date: NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            is_active: true,
            items: vec![
                MealPlanItemInput {
                    planned_date: NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
                    meal_type: "breakfast".to_string(),
                    recipe_id: None,
                    food_item_id: None,
                    custom_food_name: Some("Apple".to_string()),
                    servings: 1.0,
                }
            ],
        }).await.unwrap();

        assert!(plan1.plan.is_active);

        // Seed a second active plan
        let plan2 = service.create_meal_plan(user_id, CreateMealPlanRequest {
            name: Some("Plan 2".to_string()),
            start_date: NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            is_active: true,
            items: vec![
                MealPlanItemInput {
                    planned_date: NaiveDate::from_ymd_opt(2026, 6, 10).unwrap(),
                    meal_type: "lunch".to_string(),
                    recipe_id: None,
                    food_item_id: None,
                    custom_food_name: Some("Salad".to_string()),
                    servings: 1.0,
                }
            ],
        }).await.unwrap();

        assert!(plan2.plan.is_active);

        // Get plan 1, it should be inactive now
        let p1_fetched = service.get_meal_plan(plan1.plan.id).await.unwrap();
        assert!(!p1_fetched.plan.is_active);
    }
}
