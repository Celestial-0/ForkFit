use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::app::AppState;
use crate::common::AppResult;
use crate::common::pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
use crate::middleware::CurrentUser;
use crate::infra::pg::profile_repo::PgProfileRepository;

use super::service::ProfileService;
use super::types::*;

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<ProfileResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let profile = service.get_profile(user.id).await?;
    Ok(Json(ProfileResponse::from(profile)))
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> AppResult<Json<ProfileResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let profile = service.update_profile(user.id, payload).await?;
    Ok(Json(ProfileResponse::from(profile)))
}

pub async fn get_preferences(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<UserPreferenceResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let prefs = service.get_preferences(user.id).await?;
    Ok(Json(UserPreferenceResponse::from(prefs)))
}

pub async fn update_preferences(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<UpdatePreferenceRequest>,
) -> AppResult<Json<UserPreferenceResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let prefs = service.update_preferences(user.id, payload).await?;
    Ok(Json(UserPreferenceResponse::from(prefs)))
}

pub async fn get_medical_safety(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<MedicalSafetyResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let safety = service.get_medical_safety(user.id).await?;
    Ok(Json(MedicalSafetyResponse::from(safety)))
}

pub async fn update_medical_safety(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<UpdateMedicalSafetyRequest>,
) -> AppResult<Json<MedicalSafetyResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let safety = service.update_medical_safety(user.id, payload).await?;
    Ok(Json(MedicalSafetyResponse::from(safety)))
}

pub async fn log_biometric(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateBiometricRequest>,
) -> AppResult<Json<BiometricResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let biometric = service.log_biometric(user.id, payload).await?;
    Ok(Json(BiometricResponse::from(biometric)))
}

pub async fn get_biometric_history(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<BiometricResponse>>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let page = params.page();
    let per_page = params.per_page();
    
    let (logs, total) = service.get_biometric_history(user.id, page, per_page).await?;
    
    let data = logs.into_iter().map(BiometricResponse::from).collect();
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data, meta }))
}

pub async fn log_workout(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateWorkoutRequest>,
) -> AppResult<Json<WorkoutResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let workout = service.log_workout(user.id, payload).await?;
    Ok(Json(WorkoutResponse::from(workout)))
}

pub async fn get_recent_workouts(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<WorkoutResponse>>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let page = params.page();
    let per_page = params.per_page();
    
    let (logs, total) = service.get_recent_workouts(user.id, page, per_page).await?;
    
    let data = logs.into_iter().map(WorkoutResponse::from).collect();
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data, meta }))
}

pub async fn get_active_goals(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
) -> AppResult<Json<Vec<GoalResponse>>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let goals = service.get_active_goals(user.id).await?;
    Ok(Json(goals.into_iter().map(GoalResponse::from).collect()))
}

pub async fn create_goal(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateGoalRequest>,
) -> AppResult<Json<GoalResponse>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    let goal = service.create_goal(user.id, payload).await?;
    Ok(Json(GoalResponse::from(goal)))
}

pub async fn deactivate_goal(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(category): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let repo = PgProfileRepository::new(state.db.clone());
    let service = ProfileService::new(repo);
    service.deactivate_goal(user.id, &category).await?;
    Ok(Json(serde_json::json!({ "deactivated": true })))
}
