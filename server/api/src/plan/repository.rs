use crate::common::AppResult;
use crate::common::id::{UserId, MealPlanId, MealPlanItemId, PantryItemId, ShoppingListId, ShoppingListItemId};
use super::models::{MealPlan, MealPlanItem, PantryItem, ShoppingList, ShoppingListItem};
use chrono::NaiveDate;

pub trait PlanRepository: Send + Sync {
    // Meal Plans
    async fn create_meal_plan(&self, plan: MealPlan, items: Vec<MealPlanItem>) -> AppResult<(MealPlan, Vec<MealPlanItem>)>;
    async fn get_meal_plan(&self, id: MealPlanId) -> AppResult<Option<(MealPlan, Vec<MealPlanItem>)>>;
    async fn get_active_meal_plan(&self, user_id: UserId) -> AppResult<Option<(MealPlan, Vec<MealPlanItem>)>>;
    async fn get_meal_plan_item(&self, id: MealPlanItemId) -> AppResult<Option<MealPlanItem>>;
    async fn list_meal_plans(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<MealPlan>, u64)>;
    async fn set_meal_plan_active(&self, user_id: UserId, id: MealPlanId, active: bool) -> AppResult<()>;
    async fn update_meal_plan_item_consumed(&self, item_id: MealPlanItemId, consumed: bool) -> AppResult<Option<MealPlanItem>>;

    // Pantry
    async fn get_pantry_item(&self, id: PantryItemId) -> AppResult<Option<PantryItem>>;
    async fn list_pantry_items(&self, user_id: UserId) -> AppResult<Vec<PantryItem>>;
    async fn create_pantry_item(&self, item: PantryItem) -> AppResult<PantryItem>;
    async fn update_pantry_item(&self, id: PantryItemId, quantity: f64, expires_at: Option<NaiveDate>) -> AppResult<PantryItem>;
    async fn delete_pantry_item(&self, id: PantryItemId) -> AppResult<()>;

    // Shopping Lists
    async fn create_shopping_list(&self, list: ShoppingList, items: Vec<ShoppingListItem>) -> AppResult<(ShoppingList, Vec<ShoppingListItem>)>;
    async fn get_shopping_list(&self, id: ShoppingListId) -> AppResult<Option<(ShoppingList, Vec<ShoppingListItem>)>>;
    async fn list_shopping_lists(&self, user_id: UserId) -> AppResult<Vec<ShoppingList>>;
    async fn update_shopping_list_item_acquired(&self, list_id: ShoppingListId, item_id: ShoppingListItemId, is_acquired: bool) -> AppResult<()>;
}
