use std::sync::Arc;
use axum::{
    Json,
    extract::{Query, State, Path},
};

use crate::app::AppState;
use crate::common::AppResult;
use crate::common::pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
use crate::middleware::{CurrentUser, require_permission};
use crate::background::models::BackgroundJob;
use crate::background::repository::BackgroundRepository;
use crate::infra::pg::background_repo::PgBackgroundRepository;
use crate::audit::models::AuditLog;
use crate::audit::repository::AuditRepository;
use crate::infra::pg::audit_repo::PgAuditRepository;

use crate::common::id::{RawFoodCostId, FoodItemId};
use crate::recipe::types::{
    CreateRawFoodCostRequest, UpdateRawFoodCostRequest, RawFoodCostResponse,
};
use crate::recipe::repository::RecipeRepository;
use crate::infra::pg::recipe_repo::PgRecipeRepository;

pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<BackgroundJob>>> {
    require_permission(&state, user.id, "admin", "read").await?;

    let repo = PgBackgroundRepository::new(state.db.clone());
    let page = params.page();
    let per_page = params.per_page();

    let (jobs, total) = repo.list_jobs(page, per_page).await?;
    let meta = PaginationMeta::new(page, per_page, total);

    Ok(Json(PaginatedResponse { data: jobs, meta }))
}

pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<AuditLog>>> {
    require_permission(&state, user.id, "admin", "read").await?;

    let repo = PgAuditRepository::new(state.db.clone());
    let page = params.page();
    let per_page = params.per_page();

    let (logs, total) = repo.list_audit_logs(page, per_page).await?;
    let meta = PaginationMeta::new(page, per_page, total);

    Ok(Json(PaginatedResponse { data: logs, meta }))
}

pub async fn list_raw_food_costs(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
    Query(search): Query<crate::recipe::handlers::SearchQuery>,
) -> AppResult<Json<PaginatedResponse<RawFoodCostResponse>>> {
    require_permission(&state, user.id, "admin", "read").await?;

    let repo = PgRecipeRepository::new(state.db.clone());
    let page = params.page();
    let per_page = params.per_page();

    let (costs, total) = repo.list_raw_food_costs(&search.q.unwrap_or_default(), page, per_page).await?;
    let data = costs.into_iter().map(RawFoodCostResponse::from).collect();
    let meta = PaginationMeta::new(page, per_page, total);

    Ok(Json(PaginatedResponse { data, meta }))
}

pub async fn create_raw_food_cost(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateRawFoodCostRequest>,
) -> AppResult<Json<RawFoodCostResponse>> {
    require_permission(&state, user.id, "admin", "write").await?;

    let repo = PgRecipeRepository::new(state.db.clone());
    let cost = repo.create_raw_food_cost(
        &payload.food_pattern,
        payload.cost_per_100g,
        payload.price_currency.as_deref().unwrap_or("INR"),
    ).await?;

    Ok(Json(RawFoodCostResponse::from(cost)))
}

pub async fn update_raw_food_cost(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<RawFoodCostId>,
    Json(payload): Json<UpdateRawFoodCostRequest>,
) -> AppResult<Json<RawFoodCostResponse>> {
    require_permission(&state, user.id, "admin", "write").await?;

    let repo = PgRecipeRepository::new(state.db.clone());
    let cost = repo.update_raw_food_cost(
        id,
        &payload.food_pattern,
        payload.cost_per_100g,
        payload.price_currency.as_deref().unwrap_or("INR"),
    ).await?;

    Ok(Json(RawFoodCostResponse::from(cost)))
}

pub async fn delete_raw_food_cost(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<RawFoodCostId>,
) -> AppResult<Json<serde_json::Value>> {
    require_permission(&state, user.id, "admin", "write").await?;

    let repo = PgRecipeRepository::new(state.db.clone());
    repo.delete_raw_food_cost(id).await?;

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

#[derive(serde::Deserialize)]
pub struct LinkFoodItemCostRequest {
    pub food_item_id: FoodItemId,
    pub raw_food_cost_id: Option<RawFoodCostId>,
}

pub async fn link_food_item_to_cost(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<LinkFoodItemCostRequest>,
) -> AppResult<Json<serde_json::Value>> {
    require_permission(&state, user.id, "admin", "write").await?;

    let repo = PgRecipeRepository::new(state.db.clone());
    repo.link_food_item_to_cost(payload.food_item_id, payload.raw_food_cost_id).await?;

    Ok(Json(serde_json::json!({ "status": "linked" })))
}
