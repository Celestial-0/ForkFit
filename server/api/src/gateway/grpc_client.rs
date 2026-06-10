use std::env;
use std::time::Duration;
use tonic::transport::Channel;
use crate::common::error::{AppError, AppResult};

pub mod intelligence {
    // Include the generated proto structures by cargo/tonic
    tonic::include_proto!("forkfit.intelligence.v1");
}

use intelligence::intelligence_service_client::IntelligenceServiceClient;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use opentelemetry::propagation::TextMapPropagator;

fn inject_trace_context<T>(request: &mut tonic::Request<T>) {
    let context = tracing::Span::current().context();
    let propagator = opentelemetry_sdk::propagation::TraceContextPropagator::new();
    let mut fields = std::collections::HashMap::new();
    propagator.inject_context(&context, &mut fields);
    
    if let Some(traceparent) = fields.get("traceparent") {
        if let Ok(metadata_val) = traceparent.parse() {
            request.metadata_mut().insert("traceparent", metadata_val);
        }
    }
}

#[derive(Clone, Debug)]
pub struct GrpcIntelligenceClient {
    client: IntelligenceServiceClient<Channel>,
}

impl GrpcIntelligenceClient {
    /// Creates a new GrpcIntelligenceClient. It reads the endpoint from the 
    /// `INTELLIGENCE_GRPC_URL` environment variable, falling back to `http://127.0.0.1:50051`.
    pub async fn new() -> AppResult<Self> {
        let endpoint_url = env::var("INTELLIGENCE_GRPC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

        let channel = Channel::from_shared(endpoint_url.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid gRPC endpoint URL '{}': {}", endpoint_url, e)))?
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .connect()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to connect to gRPC server at '{}': {}", endpoint_url, e)))?;

        let client = IntelligenceServiceClient::new(channel);
        Ok(Self { client })
    }

    /// Creates a new GrpcIntelligenceClient lazily, without blocking for handshake.
    pub fn new_lazy() -> AppResult<Self> {
        let endpoint_url = env::var("INTELLIGENCE_GRPC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

        let channel = Channel::from_shared(endpoint_url.clone())
            .map_err(|e| AppError::BadRequest(format!("Invalid gRPC endpoint URL '{}': {}", endpoint_url, e)))?
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .connect_lazy();

        let client = IntelligenceServiceClient::new(channel);
        Ok(Self { client })
    }

    /// Analyzes raw user prompts to classify intent.
    pub async fn process_intent(
        &self,
        prompt: String,
        user_id: String,
    ) -> AppResult<intelligence::IntentResponse> {
        let mut client = self.client.clone();
        let mut request = tonic::Request::new(intelligence::IntentRequest { prompt, user_id });
        inject_trace_context(&mut request);
        
        let response = client.process_intent(request).await.map_err(|status| {
            AppError::BadRequest(format!("gRPC ProcessIntent failed: {}", status.message()))
        })?;
        
        Ok(response.into_inner())
    }

    /// Executes the cognitive reasoning graph (LangGraph) and returns a streaming response of updates.
    pub async fn orchestrate_agent_graph(
        &self,
        trace_id: String,
        session_id: String,
        prompt: String,
        context: intelligence::UserContext,
        history: Vec<intelligence::ChatMessageHistory>,
    ) -> AppResult<tonic::Streaming<intelligence::OrchestrateGraphResponse>> {
        let mut client = self.client.clone();
        let mut request = tonic::Request::new(intelligence::OrchestrateGraphRequest {
            trace_id,
            session_id,
            prompt,
            context: Some(context),
            history,
        });
        inject_trace_context(&mut request);
 
        let response = client.orchestrate_agent_graph(request).await.map_err(|status| {
            AppError::BadRequest(format!("gRPC OrchestrateAgentGraph failed: {}", status.message()))
        })?;

        Ok(response.into_inner())
    }

    /// Triggers the reflection engine for continuous learning and preference extraction.
    pub async fn trigger_reflection(
        &self,
        user_id: String,
        session_id: String,
        chat_message_id: String,
        feedback_rating: i32,
        feedback_text: String,
    ) -> AppResult<intelligence::ReflectionResponse> {
        let mut client = self.client.clone();
        let mut request = tonic::Request::new(intelligence::ReflectionRequest {
            user_id,
            session_id,
            chat_message_id,
            feedback_rating,
            feedback_text,
        });
        inject_trace_context(&mut request);
 
        let response = client.trigger_reflection(request).await.map_err(|status| {
            AppError::BadRequest(format!("gRPC TriggerReflection failed: {}", status.message()))
        })?;

        Ok(response.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence::UserContext;

    #[tokio::test]
    async fn test_grpc_handshake_with_mock_server() {
        // Set env variable to ensure we connect to the running mock server
        unsafe {
            std::env::set_var("INTELLIGENCE_GRPC_URL", "http://127.0.0.1:50051");
        }

        let client = match GrpcIntelligenceClient::new().await {
            Ok(c) => c,
            Err(_) => {
                println!("gRPC mock server is offline; skipping handshake verification test.");
                return;
            }
        };

        // 1. Test ProcessIntent
        let intent_res = client
            .process_intent("Vegetarian muscle gain plan under 250 INR".to_string(), "test-user-123".to_string())
            .await
            .unwrap();
        assert_eq!(intent_res.goal, "muscle_gain");
        assert_eq!(intent_res.diet, "vegetarian");
        assert_eq!(intent_res.budget_limit, 250.0);

        // 2. Test OrchestrateAgentGraph
        let context = UserContext {
            user_id: "test-user-123".to_string(),
            allergies: vec![],
            medical_conditions: vec![],
            preferred_foods: vec![],
            avoided_foods: vec![],
            daily_calorie_target: 2000.0,
            target_protein_g: 120.0,
            target_carbs_g: 200.0,
            target_fat_g: 60.0,
            budget_limit: 250.0,
            preferred_cuisine: "Indian".to_string(),
        };

        let mut stream = client
            .orchestrate_agent_graph(
                "test-trace-id".to_string(),
                "test-session-id".to_string(),
                "Give me a meal plan".to_string(),
                context,
                vec![],
            )
            .await
            .unwrap();

        let mut step_count = 0;
        let mut final_text_received = false;
        let mut ui_element_received = false;

        while let Some(msg) = stream.message().await.unwrap() {
            if let Some(payload) = msg.payload {
                match payload {
                    intelligence::orchestrate_graph_response::Payload::StepUpdate(step) => {
                        println!("Received step: {} (Status: {})", step.agent_name, step.status);
                        step_count += 1;
                    }
                    intelligence::orchestrate_graph_response::Payload::FinalText(text) => {
                        println!("Received final text: {}", text);
                        final_text_received = true;
                    }
                    intelligence::orchestrate_graph_response::Payload::UiElement(ui) => {
                        println!("Received UI Element: {} (Type: {})", ui.title, ui.r#type);
                        ui_element_received = true;
                    }
                    intelligence::orchestrate_graph_response::Payload::TextDelta(delta) => {
                        println!("Received text delta: {} (index: {})", delta.content, delta.delta_index);
                    }
                }
            }
        }

        assert_eq!(step_count, 3);
        assert!(final_text_received);
        assert!(ui_element_received);

        // 3. Test TriggerReflection
        let reflection_res = client
            .trigger_reflection(
                "test-user-123".to_string(),
                "test-session-id".to_string(),
                "msg-id-123".to_string(),
                5,
                "Great plan!".to_string(),
            )
            .await
            .unwrap();
        assert!(reflection_res.success);
        assert_eq!(reflection_res.extracted_memories, vec!["Prefers Paneer", "Avoids Mushrooms"]);
    }
}

