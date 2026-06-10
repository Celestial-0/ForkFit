use std::sync::Arc;
use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::app::AppState;
use crate::common::AppResult;
use crate::common::id::ChatThreadId;
use crate::common::pagination::{PaginationParams, PaginatedResponse, PaginationMeta};
use crate::middleware::CurrentUser;
use crate::infra::pg::chat_repo::PgChatRepository;

use super::service::ChatService;
use super::types::*;

fn make_service(state: &AppState) -> ChatService<PgChatRepository> {
    let repo = PgChatRepository::new(state.db.clone());
    ChatService::new(repo, state.grpc.clone())
}

// Helper struct for standard success responses
#[derive(serde::Serialize)]
pub struct StatusCodeResponse {
    pub success: bool,
}

// --- Thread Handlers ---

pub async fn create_thread(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateThreadRequest>,
) -> AppResult<Json<ThreadResponse>> {
    let service = make_service(&state);
    let thread = service.create_thread(user.id, payload).await?;
    Ok(Json(thread))
}

pub async fn get_thread(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<ChatThreadId>,
) -> AppResult<Json<ThreadResponse>> {
    let service = make_service(&state);
    let thread = service.get_thread(user.id, id).await?;
    Ok(Json(thread))
}

pub async fn list_threads(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<ThreadResponse>>> {
    let service = make_service(&state);
    let page = params.page();
    let per_page = params.per_page();

    let (threads, total) = service.list_threads(user.id, page, per_page).await?;
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data: threads, meta }))
}

pub async fn delete_thread(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<ChatThreadId>,
) -> AppResult<Json<StatusCodeResponse>> {
    let service = make_service(&state);
    service.delete_thread(user.id, id).await?;
    Ok(Json(StatusCodeResponse { success: true }))
}

// --- Message Handlers ---

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<ChatThreadId>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<MessageResponse>>> {
    let service = make_service(&state);
    let page = params.page();
    let per_page = params.per_page();

    let (messages, total) = service.list_messages(user.id, id, page, per_page).await?;
    let meta = PaginationMeta::new(page, per_page, total);
    Ok(Json(PaginatedResponse { data: messages, meta }))
}

pub async fn post_message(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<ChatThreadId>,
    Json(payload): Json<CreateMessageRequest>,
) -> AppResult<Json<MessageResponse>> {
    let service = make_service(&state);
    let msg = service.post_message(user.id, user.session_id, id, payload).await?;
    Ok(Json(msg))
}

// --- Feedback Handlers ---

pub async fn create_feedback(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Json(payload): Json<CreateFeedbackRequest>,
) -> AppResult<Json<FeedbackResponse>> {
    let service = make_service(&state);
    let feedback = service.create_feedback(user.id, user.session_id, payload).await?;
    Ok(Json(feedback))
}
