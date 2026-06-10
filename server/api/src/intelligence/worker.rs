use std::sync::Arc;
use crate::app::AppState;
use crate::common::id::{UserId, TraceId, ChatThreadId, SessionId};
use super::stream::SseEvent;
use super::orchestrator::run_orchestration;

pub fn spawn_orchestration_worker(
    state: Arc<AppState>,
    trace_id: TraceId,
    user_id: UserId,
    session_id: SessionId,
    thread_id: ChatThreadId,
    prompt: String,
    tx: tokio::sync::broadcast::Sender<SseEvent>,
) {
    tokio::spawn(async move {
        tracing::info!("Starting background orchestration worker for trace: {}", trace_id);
        if let Err(e) = run_orchestration(state, trace_id, user_id, session_id, thread_id, prompt, tx).await {
            tracing::error!("Error in orchestration worker execution: {:?}", e);
        }
    });
}
