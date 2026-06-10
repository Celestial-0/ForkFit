use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::app::AppState;
use crate::common::AppResult;

use crate::common::id::{UserId, TraceId, ChatThreadId, SessionId, ChatMessageId};
use crate::chat::models::{ChatMessage, MessageRole};
use crate::chat::repository::ChatRepository;
use crate::infra::pg::chat_repo::PgChatRepository;
use crate::infra::pg::intelligence_repo::PgIntelligenceRepository;
use crate::intelligence::repository::IntelligenceRepository;
use crate::intelligence::models::AiExecutionStep;
use crate::infra::redis::idempotency::release_orchestration_lock;

use super::stream::SseEvent;
use super::delta::DeltaAccumulator;
use super::context_builder::{build_user_context, build_chat_history};

pub async fn run_orchestration(
    state: Arc<AppState>,
    trace_id: TraceId,
    user_id: UserId,
    session_id: SessionId,
    thread_id: ChatThreadId,
    prompt: String,
    tx: tokio::sync::broadcast::Sender<SseEvent>,
) -> AppResult<()> {
    let start_time = Utc::now();
    let intel_repo = PgIntelligenceRepository::new(state.db.clone());

    // 1. Broadcast TraceStart
    let _ = tx.send(SseEvent::TraceStart {
        trace_id: trace_id.0,
        status: "running".to_string(),
    });

    // Update trace status in DB
    intel_repo.update_trace_status(
        trace_id,
        "running".to_string(),
        None,
        None,
        None,
        None,
    )
    .await?;

    let result: AppResult<(String, Vec<serde_json::Value>)> = async {
        // 2. Build Context
        let context = build_user_context(&state, user_id).await?;
        let history = build_chat_history(&state, thread_id, 20).await?;

        // 3. Call gRPC
        let mut stream = state.grpc.orchestrate_agent_graph(
            trace_id.to_string(),
            session_id.to_string(),
            prompt.clone(),
            context,
            history,
        )
        .await?;

        // 4. Create DeltaAccumulator
        let mut accumulator = DeltaAccumulator::new(tx.clone());
        let mut ui_elements = Vec::new();
        let mut received_deltas = false;

        // 5. Stream loop
        while let Some(msg) = stream.message().await? {
            if let Some(payload) = msg.payload {
                match payload {
                    crate::gateway::grpc_client::intelligence::orchestrate_graph_response::Payload::StepUpdate(step) => {
                        let step_id = Uuid::parse_str(&step.step_id).unwrap_or_else(|_| Uuid::new_v4());
                        let latency = Some(step.latency_ms as i32);
                        
                        let db_step = AiExecutionStep {
                            id: step_id,
                            trace_id,
                            parent_step_id: None,
                            step_name: step.agent_name.clone(),
                            step_type: step.step_type.clone(),
                            status: step.status.clone(),
                            input_payload: serde_json::from_str(&step.input_payload_json).unwrap_or_else(|_| serde_json::json!({})),
                            output_payload: serde_json::from_str(&step.output_payload_json).unwrap_or_else(|_| serde_json::json!({})),
                            model_name: Some("Gemma4".to_string()),
                            prompt_tokens: None,
                            completion_tokens: None,
                            latency_ms: latency,
                            error_message: if step.error_message.is_empty() { None } else { Some(step.error_message) },
                            started_at: Utc::now(),
                            completed_at: Some(Utc::now()),
                        };

                        let _ = intel_repo.create_step(db_step).await;
                        
                        let _ = tx.send(SseEvent::AgentStep {
                            agent: step.agent_name,
                            status: step.status,
                            step_type: step.step_type,
                            latency_ms: latency,
                        });
                    }
                    crate::gateway::grpc_client::intelligence::orchestrate_graph_response::Payload::TextDelta(delta) => {
                        received_deltas = true;
                        accumulator.push(&delta.content, &delta.delta_type, delta.is_complete);
                    }
                    crate::gateway::grpc_client::intelligence::orchestrate_graph_response::Payload::FinalText(text) => {
                        if !received_deltas {
                            accumulator.push(&text, "markdown", true);
                        }
                    }
                    crate::gateway::grpc_client::intelligence::orchestrate_graph_response::Payload::UiElement(ui) => {
                        let config_val: serde_json::Value = serde_json::from_str(&ui.config_json).unwrap_or_else(|_| serde_json::json!({}));
                        let data_val: serde_json::Value = serde_json::from_str(&ui.data_json).unwrap_or_else(|_| serde_json::json!({}));

                        let sse_ui = SseEvent::UiElement {
                            element_type: ui.r#type.clone(),
                            title: ui.title.clone(),
                            config_json: config_val.clone(),
                            data_json: data_val.clone(),
                        };
                        let _ = tx.send(sse_ui);

                        // Persist structured plans if present
                        if ui.r#type == "meal_plan" {
                            if let Ok(req) = serde_json::from_str::<crate::plan::types::CreateMealPlanRequest>(&ui.data_json) {
                                let plan_repo = crate::infra::pg::plan_repo::PgPlanRepository::new(state.db.clone());
                                let recipe_repo = crate::infra::pg::recipe_repo::PgRecipeRepository::new(state.db.clone());
                                let recipe_service = crate::recipe::service::RecipeService::new(recipe_repo);
                                let plan_service = crate::plan::service::PlanService::new(plan_repo, recipe_service);
                                
                                if let Err(e) = plan_service.create_meal_plan(user_id, req).await {
                                    tracing::error!("Failed to persist AI generated meal plan: {:?}", e);
                                } else {
                                    tracing::info!("Successfully persisted AI generated meal plan");
                                }
                            }
                        } else if ui.r#type == "shopping_list" {
                            if let Ok(req) = serde_json::from_str::<crate::plan::types::CreateShoppingListRequest>(&ui.data_json) {
                                let plan_repo = crate::infra::pg::plan_repo::PgPlanRepository::new(state.db.clone());
                                let recipe_repo = crate::infra::pg::recipe_repo::PgRecipeRepository::new(state.db.clone());
                                let recipe_service = crate::recipe::service::RecipeService::new(recipe_repo);
                                let plan_service = crate::plan::service::PlanService::new(plan_repo, recipe_service);
                                
                                if let Err(e) = plan_service.create_shopping_list(user_id, req).await {
                                    tracing::error!("Failed to persist AI generated shopping list: {:?}", e);
                                } else {
                                    tracing::info!("Successfully persisted AI generated shopping list");
                                }
                            }
                        }

                        ui_elements.push(serde_json::json!({
                            "type": ui.r#type,
                            "title": ui.title,
                            "config": config_val,
                            "data": data_val,
                        }));
                    }
                }
            }
        }

        let full_text = accumulator.finalize();
        Ok((full_text, ui_elements))
    }
    .await;

    // Always release lock and log completion stats
    let _ = release_orchestration_lock(&state.redis, user_id.to_string()).await;
    let end_time = Utc::now();
    let latency_ms = (end_time - start_time).num_milliseconds() as i32;

    match result {
        Ok((full_text, ui_elements)) => {
            // Save Assistant Message
            let chat_repo = PgChatRepository::new(state.db.clone());
            let assistant_msg = ChatMessage {
                id: ChatMessageId::new(),
                thread_id,
                sender_role: MessageRole::Assistant,
                content: full_text,
                metadata: serde_json::json!({ "ui_elements": ui_elements }),
                created_at: Utc::now(),
            };
            let _ = chat_repo.create_message(assistant_msg).await;

            // Update Trace as completed
            let _ = intel_repo.update_trace_status(
                trace_id,
                "completed".to_string(),
                Some(end_time),
                Some(latency_ms),
                Some(0),
                Some(0.0),
            )
            .await;

            // Broadcast Done
            let _ = tx.send(SseEvent::Done {
                trace_id: trace_id.0,
                total_latency_ms: latency_ms,
            });
            Ok(())
        }
        Err(err) => {
            // Update Trace as failed
            let _ = intel_repo.update_trace_status(
                trace_id,
                "failed".to_string(),
                Some(end_time),
                Some(latency_ms),
                None,
                None,
            )
            .await;

            // Broadcast Error
            let _ = tx.send(SseEvent::Error {
                trace_id: trace_id.0,
                message: err.to_string(),
            });
            Err(err)
        }
    }
}
