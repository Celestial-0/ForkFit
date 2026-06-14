use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::NaiveDate;

use crate::app::AppState;
use crate::common::AppResult;
use crate::common::pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
use crate::middleware::CurrentUser;
use crate::infra::pg::recipe_repo::PgRecipeRepository;

use super::service::RecipeService;
use super::types::*;

// --- Food Item Handlers ---

pub async fn create_food_item(
    State(state): State<Arc<AppState>>,
    _user: CurrentUser, // Verify authenticated
    Json(payload): Json<CreateFoodItemRequest>,
) -> AppResult<Json<FoodItemResponse>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let food = service.create_food_item(payload).await?;
    Ok(Json(FoodItemResponse::from(food)))
}

pub async fn get_food_item(
    State(state): State<Arc<AppState>>,
    _user: CurrentUser,
    Path(id): Path<crate::common::id::FoodItemId>,
) -> AppResult<Json<FoodItemResponse>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let food = service.get_food_item(id).await?;
    Ok(Json(FoodItemResponse::from(food)))
}

pub async fn search_food_items(
    State(state): State<Arc<AppState>>,
    _user: CurrentUser,
    Query(params): Query<PaginationParams>,
    Query(search): Query<SearchQuery>,
) -> AppResult<Json<PaginatedResponse<FoodItemResponse>>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let page = params.page();
    let per_page = params.per_page();
    
    let (foods, total) = service.search_food_items(&search.q.unwrap_or_default(), page, per_page).await?;
    
    let data = foods.into_iter().map(FoodItemResponse::from).collect();
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data, meta }))
}

// --- Recipes Handlers ---

pub async fn create_recipe(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateRecipeRequest>,
) -> AppResult<Json<RecipeDetailResponse>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let detail = service.create_recipe(user.id, payload).await?;
    Ok(Json(detail))
}

pub async fn get_recipe(
    State(state): State<Arc<AppState>>,
    _user: CurrentUser,
    Path(id): Path<crate::common::id::RecipeId>,
) -> AppResult<Json<RecipeDetailResponse>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let detail = service.get_recipe(id).await?;
    Ok(Json(detail))
}

pub async fn list_recipes(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
    Query(filter): Query<RecipeFilter>,
) -> AppResult<Json<PaginatedResponse<RecipeResponse>>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let page = params.page();
    let per_page = params.per_page();
    
    let filter_owner = if filter.mine.unwrap_or(false) {
        Some(user.id)
    } else {
        None
    };

    let (recipes, total) = service.list_recipes(filter_owner, page, per_page).await?;
    
    let data = recipes.into_iter().map(RecipeResponse::from).collect();
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data, meta }))
}

// --- Food Log Handlers ---

pub async fn log_food(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<LogFoodRequest>,
) -> AppResult<Json<FoodLogResponse>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let log = service.log_food(user.id, payload).await?;
    Ok(Json(FoodLogResponse::from(log)))
}

pub async fn get_food_logs(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<FoodLogResponse>>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let page = params.page();
    let per_page = params.per_page();
    
    let (logs, total) = service.get_food_logs(user.id, page, per_page).await?;
    
    let data = logs.into_iter().map(FoodLogResponse::from).collect();
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data, meta }))
}

pub async fn get_daily_macros(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(date_param): Query<DateQuery>,
) -> AppResult<Json<DailyMacrosSummary>> {
    let repo = PgRecipeRepository::new(state.db.clone());
    let service = RecipeService::new(repo);
    let date = date_param.date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    
    let (calories, protein, carbs, fats) = service.get_daily_macros(user.id, date).await?;
    Ok(Json(DailyMacrosSummary {
        date,
        calories,
        protein,
        carbs,
        fats,
    }))
}

// --- Query Structs ---

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RecipeFilter {
    pub mine: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct DateQuery {
    pub date: Option<NaiveDate>,
}
