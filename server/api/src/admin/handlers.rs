use std::sync::Arc;
use axum::{
    Json,
    extract::{Query, State},
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
