use std::{convert::Infallible, time::Duration, sync::Arc};
use axum::{
    response::sse::{Event, KeepAlive, Sse},
    extract::{Path, State},
};
use futures::stream::{Stream, StreamExt};
use serde::Serialize;
use uuid::Uuid;
use tokio_stream::wrappers::BroadcastStream;

use crate::app::AppState;
use crate::common::AppResult;
use crate::common::error::AppError;
use crate::common::id::TraceId;
use crate::middleware::CurrentUser;
use crate::infra::pg::intelligence_repo::PgIntelligenceRepository;
use crate::intelligence::repository::IntelligenceRepository;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum SseEvent {
    TraceStart { trace_id: Uuid, status: String },
    AgentStep { agent: String, status: String, step_type: String, latency_ms: Option<i32> },
    MessageDelta { content: String, index: i32, delta_type: String },
    MessageComplete { content_length: usize },
    UiElement { element_type: String, title: String, config_json: serde_json::Value, data_json: serde_json::Value },
    Done { trace_id: Uuid, total_latency_ms: i32 },
    Error { trace_id: Uuid, message: String },
}

impl SseEvent {
    pub fn to_sse_event(&self) -> Result<Event, serde_json::Error> {
        let name = match self {
            Self::TraceStart { .. } => "trace_start",
            Self::AgentStep { .. } => "agent_step",
            Self::MessageDelta { .. } => "message_delta",
            Self::MessageComplete { .. } => "message_complete",
            Self::UiElement { .. } => "ui_element",
            Self::Done { .. } => "done",
            Self::Error { .. } => "error",
        };
        
        let data = serde_json::to_string(self)?;
        Ok(Event::default().event(name).data(data))
    }
}

pub async fn get_stream(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(trace_id): Path<TraceId>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // 1. Verify trace ownership
    let repo = PgIntelligenceRepository::new(state.db.clone());
    let trace = repo.get_trace(trace_id).await?
        .ok_or(AppError::NotFound)?;
        
    if let Some(trace_user_id) = trace.user_id {
        if trace_user_id != user.id {
            return Err(AppError::Forbidden);
        }
    }

    // 2. Look up the broadcast channel in state
    let channels = state.trace_channels.lock().await;
    let rx = if let Some(tx) = channels.get(&trace_id.0) {
        tx.subscribe()
    } else {
        return Err(AppError::BadRequest("Active stream channel not found for this trace".to_string()));
    };

    drop(channels);

    // 3. Map SseEvent to Event
    let rx_stream = BroadcastStream::new(rx);
    
    let mapped_stream = rx_stream.filter_map(move |item| {
        match item {
            Ok(event) => {
                let sse_ev = event.to_sse_event().ok();
                futures::future::ready(sse_ev)
            }
            Err(_) => futures::future::ready(None),
        }
    })
    .map(Ok);

    Ok(Sse::new(mapped_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15))))
}
