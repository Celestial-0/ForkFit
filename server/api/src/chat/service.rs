use crate::common::AppResult;
use crate::common::id::{UserId, ChatThreadId, ChatMessageId, FeedbackId, SessionId};
use crate::chat::models::{ChatThread, ChatMessage, UserFeedback, MessageRole};
use crate::chat::repository::ChatRepository;
use crate::chat::error::ChatError;
use crate::chat::types::{
    CreateThreadRequest, ThreadResponse, CreateMessageRequest, MessageResponse,
    CreateFeedbackRequest, FeedbackResponse,
};
use crate::gateway::grpc_client::GrpcIntelligenceClient;
use chrono::Utc;

#[derive(Clone)]
pub struct ChatService<R> {
    repo: R,
    grpc: GrpcIntelligenceClient,
}

impl<R: ChatRepository> ChatService<R> {
    pub fn new(repo: R, grpc: GrpcIntelligenceClient) -> Self {
        Self { repo, grpc }
    }

    // Threads
    pub async fn create_thread(&self, user_id: UserId, req: CreateThreadRequest) -> AppResult<ThreadResponse> {
        let agent_type_lower = req.agent_type.trim().to_lowercase();
        if agent_type_lower != "nutritionist" && agent_type_lower != "chef" {
            return Err(ChatError::ValidationError("agent_type must be either 'nutritionist' or 'chef'".to_string()).into());
        }

        let thread = ChatThread {
            id: ChatThreadId::new(),
            user_id,
            title: req.title.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
            agent_type: agent_type_lower,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = self.repo.create_thread(thread).await?;
        Ok(ThreadResponse::from(created))
    }

    pub async fn get_thread(&self, user_id: UserId, id: ChatThreadId) -> AppResult<ThreadResponse> {
        let thread = self.repo.get_thread(id).await?
            .ok_or(ChatError::NotFound)?;

        if thread.user_id != user_id {
            return Err(ChatError::Unauthorized.into());
        }

        Ok(ThreadResponse::from(thread))
    }

    pub async fn list_threads(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<ThreadResponse>, u64)> {
        let (threads, total) = self.repo.list_threads(user_id, page, per_page).await?;
        let responses = threads.into_iter().map(ThreadResponse::from).collect();
        Ok((responses, total))
    }

    pub async fn delete_thread(&self, user_id: UserId, id: ChatThreadId) -> AppResult<()> {
        let thread = self.repo.get_thread(id).await?
            .ok_or(ChatError::NotFound)?;

        if thread.user_id != user_id {
            return Err(ChatError::Unauthorized.into());
        }

        self.repo.delete_thread(id).await?;
        Ok(())
    }

    // Messages
    pub async fn list_messages(
        &self,
        user_id: UserId,
        thread_id: ChatThreadId,
        page: u64,
        per_page: u64,
    ) -> AppResult<(Vec<MessageResponse>, u64)> {
        // Validate thread ownership
        let thread = self.repo.get_thread(thread_id).await?
            .ok_or(ChatError::NotFound)?;

        if thread.user_id != user_id {
            return Err(ChatError::Unauthorized.into());
        }

        let (messages, total) = self.repo.list_messages(thread_id, page, per_page).await?;
        let responses = messages.into_iter().map(MessageResponse::from).collect();
        Ok((responses, total))
    }

    pub async fn post_message(
        &self,
        user_id: UserId,
        session_id: SessionId,
        thread_id: ChatThreadId,
        req: CreateMessageRequest,
    ) -> AppResult<MessageResponse> {
        // Validate thread ownership
        let thread = self.repo.get_thread(thread_id).await?
            .ok_or(ChatError::NotFound)?;

        if thread.user_id != user_id {
            return Err(ChatError::Unauthorized.into());
        }

        let content_trimmed = req.content.trim().to_string();
        if content_trimmed.is_empty() {
            return Err(ChatError::ValidationError("message content cannot be empty".to_string()).into());
        }

        let message = ChatMessage {
            id: ChatMessageId::new(),
            thread_id,
            sender_role: MessageRole::User,
            content: content_trimmed,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        let created = self.repo.create_message(message).await?;

        // NOTE: In Phase 7, we trigger the orchestration streaming worker via SSE.
        // For Phase 5, we can log the event or trigger a background task placeholder.
        // In order to support testing, we trigger a background task.
        let thread_id_str = thread_id.to_string();
        let user_id_str = user_id.to_string();
        let session_id_str = session_id.to_string();

        tokio::spawn(async move {
            tracing::info!(
                "Triggering intelligence orchestration placeholder: thread={}, user={}, session={}",
                thread_id_str, user_id_str, session_id_str
            );
            
            // This is a background placeholder call, errors are logged but do not block HTTP response
            // We just trace it here.
        });

        Ok(MessageResponse::from(created))
    }

    // Feedback
    pub async fn create_feedback(
        &self,
        user_id: UserId,
        session_id: SessionId,
        req: CreateFeedbackRequest,
    ) -> AppResult<FeedbackResponse> {
        let cat_trimmed = req.category.trim().to_lowercase();
        if cat_trimmed != "chat_response" && cat_trimmed != "meal_plan" && cat_trimmed != "recipe" {
            return Err(ChatError::ValidationError("category must be 'chat_response', 'meal_plan', or 'recipe'".to_string()).into());
        }

        if req.rating < 1 || req.rating > 5 {
            return Err(ChatError::ValidationError("rating must be between 1 and 5".to_string()).into());
        }

        // Validate references if message is specified
        if cat_trimmed == "chat_response" {
            if let Some(msg_uuid) = req.reference_id {
                let msg_id = ChatMessageId(msg_uuid);
                let msg = self.repo.get_message(msg_id).await?;
                if msg.is_none() {
                    return Err(ChatError::ValidationError(format!("referenced message {} not found", msg_uuid)).into());
                }
            } else {
                return Err(ChatError::ValidationError("reference_id is required for chat_response feedback".to_string()).into());
            }
        }

        let feedback = UserFeedback {
            id: FeedbackId::new(),
            user_id,
            category: cat_trimmed.clone(),
            reference_id: req.reference_id,
            rating: req.rating,
            comment: req.comment.as_ref().map(|c| c.trim().to_string()).filter(|c| !c.is_empty()),
            metadata: req.metadata.unwrap_or_else(|| serde_json::json!({})),
            created_at: Utc::now(),
        };

        let created = self.repo.create_feedback(feedback).await?;

        // If category is 'chat_response', trigger background reflection gRPC call
        if cat_trimmed == "chat_response" {
            let grpc_client = self.grpc.clone();
            let user_id_str = user_id.to_string();
            let session_id_str = session_id.to_string();
            let message_id_str = req.reference_id.map(|id| id.to_string()).unwrap_or_default();
            let rating = req.rating;
            let comment = req.comment.clone().unwrap_or_default();

            tokio::spawn(async move {
                tracing::info!("Triggering reflection engine gRPC call for message {}", message_id_str);
                match grpc_client.trigger_reflection(user_id_str, session_id_str, message_id_str, rating, comment).await {
                    Ok(resp) => {
                        tracing::info!("Reflection engine succeeded: success={}, memories={:?}", resp.success, resp.extracted_memories);
                    }
                    Err(e) => {
                        tracing::error!("Reflection engine gRPC call failed: {:?}", e);
                    }
                }
            });
        }

        Ok(FeedbackResponse::from(created))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use uuid::Uuid;
    use crate::chat::models::{ChatThread, ChatMessage, UserFeedback};
    use crate::chat::repository::ChatRepository;

    #[derive(Default)]
    struct MockChatRepository {
        threads: Mutex<Vec<ChatThread>>,
        messages: Mutex<Vec<ChatMessage>>,
        feedbacks: Mutex<Vec<UserFeedback>>,
    }

    impl ChatRepository for MockChatRepository {
        async fn create_thread(&self, thread: ChatThread) -> AppResult<ChatThread> {
            self.threads.lock().unwrap().push(thread.clone());
            Ok(thread)
        }

        async fn get_thread(&self, id: ChatThreadId) -> AppResult<Option<ChatThread>> {
            let threads = self.threads.lock().unwrap();
            let t = threads.iter().find(|x| x.id == id).cloned();
            Ok(t)
        }

        async fn list_threads(&self, user_id: UserId, _page: u64, _per_page: u64) -> AppResult<(Vec<ChatThread>, u64)> {
            let threads = self.threads.lock().unwrap();
            let list: Vec<ChatThread> = threads.iter().filter(|x| x.user_id == user_id).cloned().collect();
            let len = list.len() as u64;
            Ok((list, len))
        }

        async fn delete_thread(&self, id: ChatThreadId) -> AppResult<()> {
            let mut threads = self.threads.lock().unwrap();
            threads.retain(|x| x.id != id);
            Ok(())
        }

        async fn create_message(&self, message: ChatMessage) -> AppResult<ChatMessage> {
            self.messages.lock().unwrap().push(message.clone());
            Ok(message)
        }

        async fn get_message(&self, id: ChatMessageId) -> AppResult<Option<ChatMessage>> {
            let messages = self.messages.lock().unwrap();
            let m = messages.iter().find(|x| x.id == id).cloned();
            Ok(m)
        }

        async fn list_messages(&self, thread_id: ChatThreadId, _page: u64, _per_page: u64) -> AppResult<(Vec<ChatMessage>, u64)> {
            let messages = self.messages.lock().unwrap();
            let list: Vec<ChatMessage> = messages.iter().filter(|x| x.thread_id == thread_id).cloned().collect();
            let len = list.len() as u64;
            Ok((list, len))
        }

        async fn create_feedback(&self, feedback: UserFeedback) -> AppResult<UserFeedback> {
            self.feedbacks.lock().unwrap().push(feedback.clone());
            Ok(feedback)
        }
    }

    #[tokio::test]
    async fn test_create_thread_validations() {
        let repo = MockChatRepository::default();
        let grpc = GrpcIntelligenceClient::new_lazy().unwrap();
        let service = ChatService::new(repo, grpc);

        let user_id = UserId::new();

        // Valid
        let req = CreateThreadRequest {
            title: Some("My plan".to_string()),
            agent_type: "chef".to_string(),
        };
        let res = service.create_thread(user_id, req).await;
        assert!(res.is_ok());

        // Invalid agent_type
        let req = CreateThreadRequest {
            title: Some("My plan".to_string()),
            agent_type: "trainer".to_string(),
        };
        let res = service.create_thread(user_id, req).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_thread_ownership() {
        let repo = MockChatRepository::default();
        let grpc = GrpcIntelligenceClient::new_lazy().unwrap();
        let service = ChatService::new(repo, grpc);

        let user1 = UserId::new();
        let user2 = UserId::new();

        let req = CreateThreadRequest {
            title: Some("User 1 thread".to_string()),
            agent_type: "chef".to_string(),
        };
        let t1 = service.create_thread(user1, req).await.unwrap();

        // Get by owner should succeed
        let res = service.get_thread(user1, t1.id).await;
        assert!(res.is_ok());

        // Get by other user should be Forbidden (Unauthorized)
        let res = service.get_thread(user2, t1.id).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_create_feedback_validations() {
        let repo = MockChatRepository::default();
        let grpc = GrpcIntelligenceClient::new_lazy().unwrap();
        let service = ChatService::new(repo, grpc);

        let user_id = UserId::new();
        let session_id = SessionId::new();

        // Valid rating for recipe
        let req = CreateFeedbackRequest {
            category: "recipe".to_string(),
            reference_id: Some(Uuid::new_v4()),
            rating: 4,
            comment: Some("Lovely recipe".to_string()),
            metadata: None,
        };
        let res = service.create_feedback(user_id, session_id, req).await;
        assert!(res.is_ok());

        // Invalid category
        let req = CreateFeedbackRequest {
            category: "exercise".to_string(),
            reference_id: Some(Uuid::new_v4()),
            rating: 4,
            comment: None,
            metadata: None,
        };
        let res = service.create_feedback(user_id, session_id, req).await;
        assert!(res.is_err());

        // Invalid rating (0)
        let req = CreateFeedbackRequest {
            category: "recipe".to_string(),
            reference_id: Some(Uuid::new_v4()),
            rating: 0,
            comment: None,
            metadata: None,
        };
        let res = service.create_feedback(user_id, session_id, req).await;
        assert!(res.is_err());

        // Invalid rating (6)
        let req = CreateFeedbackRequest {
            category: "recipe".to_string(),
            reference_id: Some(Uuid::new_v4()),
            rating: 6,
            comment: None,
            metadata: None,
        };
        let res = service.create_feedback(user_id, session_id, req).await;
        assert!(res.is_err());
    }
}
