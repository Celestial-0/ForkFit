use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::common::id::{UserId, ChatThreadId, ChatMessageId, FeedbackId};
use super::models::{ChatThread, ChatMessage, UserFeedback, MessageRole};
use uuid::Uuid;

// Threads
#[derive(Debug, Clone, Deserialize)]
pub struct CreateThreadRequest {
    pub title: Option<String>,
    pub agent_type: String, // 'nutritionist', 'chef'
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreadResponse {
    pub id: ChatThreadId,
    pub user_id: UserId,
    pub title: Option<String>,
    pub agent_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ChatThread> for ThreadResponse {
    fn from(t: ChatThread) -> Self {
        Self {
            id: t.id,
            user_id: t.user_id,
            title: t.title,
            agent_type: t.agent_type,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

// Messages
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: ChatMessageId,
    pub thread_id: ChatThreadId,
    pub sender_role: MessageRole,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl From<ChatMessage> for MessageResponse {
    fn from(m: ChatMessage) -> Self {
        Self {
            id: m.id,
            thread_id: m.thread_id,
            sender_role: m.sender_role,
            content: m.content,
            metadata: m.metadata,
            created_at: m.created_at,
        }
    }
}

// Feedback
#[derive(Debug, Clone, Deserialize)]
pub struct CreateFeedbackRequest {
    pub category: String, // 'chat_response', 'meal_plan', 'recipe'
    pub reference_id: Option<Uuid>,
    pub rating: i32,
    pub comment: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeedbackResponse {
    pub id: FeedbackId,
    pub user_id: UserId,
    pub category: String,
    pub reference_id: Option<Uuid>,
    pub rating: i32,
    pub comment: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl From<UserFeedback> for FeedbackResponse {
    fn from(f: UserFeedback) -> Self {
        Self {
            id: f.id,
            user_id: f.user_id,
            category: f.category,
            reference_id: f.reference_id,
            rating: f.rating,
            comment: f.comment,
            metadata: f.metadata,
            created_at: f.created_at,
        }
    }
}
