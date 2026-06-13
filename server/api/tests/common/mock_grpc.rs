use std::pin::Pin;
use futures::Stream;
use tonic::{Request, Response, Status};

use api::gateway::grpc_client::intelligence::{
    self,
    intelligence_service_server::IntelligenceService,
    IntentRequest, IntentResponse, OrchestrateGraphRequest, OrchestrateGraphResponse,
    ReflectionRequest, ReflectionResponse,
};

#[derive(Clone, Default)]
pub struct MockIntelligenceService {
    pub fail_orchestrate: bool,
    pub final_text_only: bool,
}

#[tonic::async_trait]
impl IntelligenceService for MockIntelligenceService {
    type OrchestrateAgentGraphStream = Pin<
        Box<dyn Stream<Item = Result<OrchestrateGraphResponse, Status>> + Send + 'static>,
    >;

    async fn process_intent(
        &self,
        _request: Request<IntentRequest>,
    ) -> Result<Response<IntentResponse>, Status> {
        let response = IntentResponse {
            goal: "muscle_gain".to_string(),
            diet: "vegetarian".to_string(),
            budget_limit: 250.0,
            budget_currency: "INR".to_string(),
            timeline: "weekly".to_string(),
            constraints: vec![],
            raw_analysis_json: "{}".to_string(),
        };
        Ok(Response::new(response))
    }

    async fn orchestrate_agent_graph(
        &self,
        _request: Request<OrchestrateGraphRequest>,
    ) -> Result<Response<Self::OrchestrateAgentGraphStream>, Status> {
        if self.fail_orchestrate {
            return Err(Status::internal("Mock gRPC orchestration failure requested"));
        }

        let responses = if self.final_text_only {
            vec![
                OrchestrateGraphResponse {
                    trace_id: "".to_string(),
                    payload: Some(intelligence::orchestrate_graph_response::Payload::FinalText(
                        "Here is the final generated meal plan text without deltas.".to_string(),
                    )),
                }
            ]
        } else {
            vec![
                // 1. Step 1 Update
                OrchestrateGraphResponse {
                    trace_id: "".to_string(),
                    payload: Some(intelligence::orchestrate_graph_response::Payload::StepUpdate(
                        intelligence::AgentStepUpdate {
                            step_id: uuid::Uuid::new_v4().to_string(),
                            agent_name: "safety_agent".to_string(),
                            step_type: "validation".to_string(),
                            status: "completed".to_string(),
                            input_payload_json: "{}".to_string(),
                            output_payload_json: "{}".to_string(),
                            latency_ms: 120,
                            error_message: "".to_string(),
                        },
                    )),
                },
                // 2. Token Delta 1
                OrchestrateGraphResponse {
                    trace_id: "".to_string(),
                    payload: Some(intelligence::orchestrate_graph_response::Payload::TextDelta(
                        intelligence::TextDelta {
                            content: "Hello! Here is your ".to_string(),
                            delta_index: 0,
                            is_complete: false,
                            delta_type: "markdown".to_string(),
                        },
                    )),
                },
                // 3. Token Delta 2
                OrchestrateGraphResponse {
                    trace_id: "".to_string(),
                    payload: Some(intelligence::orchestrate_graph_response::Payload::TextDelta(
                        intelligence::TextDelta {
                            content: "muscle gain meal plan.".to_string(),
                            delta_index: 1,
                            is_complete: true,
                            delta_type: "markdown".to_string(),
                        },
                    )),
                },
                // 4. UI Element for Meal Plan
                OrchestrateGraphResponse {
                    trace_id: "".to_string(),
                    payload: Some(intelligence::orchestrate_graph_response::Payload::UiElement(
                        intelligence::UiElement {
                            r#type: "meal_plan".to_string(),
                            title: "AI Meal Plan".to_string(),
                            config_json: "{}".to_string(),
                            data_json: serde_json::json!({
                                "name": "AI Muscle Gain Plan",
                                "start_date": "2026-06-10",
                                "end_date": "2026-06-16",
                                "is_active": true,
                                "items": [
                                    {
                                        "recipe_id": "11111111-1111-1111-1111-111111111111".to_string(),
                                        "planned_date": "2026-06-10",
                                        "meal_type": "breakfast",
                                        "servings": 1.0,
                                    }
                                ]
                            }).to_string(),
                        },
                    )),
                },
            ]
        };

        let stream = async_stream::stream! {
            // Initial short sleep to let the HTTP client subscribe
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            for resp in responses {
                yield Ok(resp);
                tokio::time::sleep(std::time::Duration::from_millis(15)).await;
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    async fn trigger_reflection(
        &self,
        _request: Request<ReflectionRequest>,
    ) -> Result<Response<ReflectionResponse>, Status> {
        let response = ReflectionResponse {
            success: true,
            extracted_memories: vec!["Prefers Paneer".to_string(), "Avoids Mushrooms".to_string()],
        };
        Ok(Response::new(response))
    }
}
