use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::app::AppState;
use crate::common::AppResult;
use crate::common::pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
use crate::middleware::CurrentUser;
use crate::infra::pg::plan_repo::PgPlanRepository;
use crate::infra::pg::recipe_repo::PgRecipeRepository;
use crate::recipe::service::RecipeService;

use super::service::PlanService;
use super::types::*;

fn make_service(state: &AppState) -> PlanService<PgPlanRepository, PgRecipeRepository> {
    let plan_repo = PgPlanRepository::new(state.db.clone());
    let recipe_repo = PgRecipeRepository::new(state.db.clone());
    let recipe_service = RecipeService::new(recipe_repo);
    PlanService::new(plan_repo, recipe_service)
}

// --- Meal Plan Handlers ---

pub async fn create_meal_plan(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateMealPlanRequest>,
) -> AppResult<Json<MealPlanDetailResponse>> {
    let service = make_service(&state);
    let detail = service.create_meal_plan(user.id, payload).await?;
    Ok(Json(detail))
}

pub async fn get_meal_plan(
    State(state): State<Arc<AppState>>,
    _user: CurrentUser,
    Path(id): Path<crate::common::id::MealPlanId>,
) -> AppResult<Json<MealPlanDetailResponse>> {
    let service = make_service(&state);
    let detail = service.get_meal_plan(id).await?;
    Ok(Json(detail))
}

pub async fn get_active_meal_plan(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<MealPlanDetailResponse>> {
    let service = make_service(&state);
    let detail = service.get_active_meal_plan(user.id).await?;
    Ok(Json(detail))
}

pub async fn list_meal_plans(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<MealPlanResponse>>> {
    let service = make_service(&state);
    let page = params.page();
    let per_page = params.per_page();

    let (plans, total) = service.list_meal_plans(user.id, page, per_page).await?;

    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data: plans, meta }))
}

pub async fn update_meal_plan_status(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<crate::common::id::MealPlanId>,
    Json(payload): Json<UpdateMealPlanStatusRequest>,
) -> AppResult<Json<StatusCodeResponse>> {
    let service = make_service(&state);
    service.set_meal_plan_active(user.id, id, payload.is_active).await?;
    Ok(Json(StatusCodeResponse { success: true }))
}

pub async fn update_meal_plan_item_consumed(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((_id, item_id)): Path<(crate::common::id::MealPlanId, crate::common::id::MealPlanItemId)>,
    Json(payload): Json<UpdateMealPlanItemConsumedRequest>,
) -> AppResult<Json<MealPlanItemResponse>> {
    let service = make_service(&state);
    let updated = service.update_meal_plan_item_consumed(user.id, item_id, payload.consumed).await?;
    Ok(Json(updated))
}

// --- Pantry Handlers ---

pub async fn list_pantry_items(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<Vec<PantryItemResponse>>> {
    let service = make_service(&state);
    let items = service.list_pantry_items(user.id).await?;
    Ok(Json(items))
}

pub async fn create_pantry_item(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreatePantryItemRequest>,
) -> AppResult<Json<PantryItemResponse>> {
    let service = make_service(&state);
    let item = service.create_pantry_item(user.id, payload).await?;
    Ok(Json(item))
}

pub async fn update_pantry_item(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<crate::common::id::PantryItemId>,
    Json(payload): Json<UpdatePantryItemRequest>,
) -> AppResult<Json<PantryItemResponse>> {
    let service = make_service(&state);
    let item = service.update_pantry_item(user.id, id, payload).await?;
    Ok(Json(item))
}

pub async fn delete_pantry_item(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<crate::common::id::PantryItemId>,
) -> AppResult<Json<StatusCodeResponse>> {
    let service = make_service(&state);
    service.delete_pantry_item(user.id, id).await?;
    Ok(Json(StatusCodeResponse { success: true }))
}

// --- Shopping List Handlers ---

pub async fn list_shopping_lists(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<Vec<ShoppingListResponse>>> {
    let service = make_service(&state);
    let lists = service.list_shopping_lists(user.id).await?;
    Ok(Json(lists))
}

pub async fn get_shopping_list(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<crate::common::id::ShoppingListId>,
) -> AppResult<Json<ShoppingListDetailResponse>> {
    let service = make_service(&state);
    let detail = service.get_shopping_list(user.id, id).await?;
    Ok(Json(detail))
}

pub async fn create_shopping_list(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateShoppingListRequest>,
) -> AppResult<Json<ShoppingListDetailResponse>> {
    let service = make_service(&state);
    let detail = service.create_shopping_list(user.id, payload).await?;
    Ok(Json(detail))
}

pub async fn update_shopping_list_item_acquired(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path((list_id, item_id)): Path<(crate::common::id::ShoppingListId, crate::common::id::ShoppingListItemId)>,
    Json(payload): Json<UpdateShoppingListItemAcquiredRequest>,
) -> AppResult<Json<StatusCodeResponse>> {
    let service = make_service(&state);
    service.update_shopping_list_item_acquired(user.id, list_id, item_id, payload.is_acquired).await?;
    Ok(Json(StatusCodeResponse { success: true }))
}

// Helper struct for standard success responses
#[derive(serde::Serialize)]
pub struct StatusCodeResponse {
    pub success: bool,
}
