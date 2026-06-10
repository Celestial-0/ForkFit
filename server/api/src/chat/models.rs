use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::common::id::{UserId, ChatThreadId, ChatMessageId, FeedbackId};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MessageRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "system" => Ok(Self::System),
            _ => Err(format!("Invalid message role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatThread {
    pub id: ChatThreadId,
    pub user_id: UserId,
    pub title: Option<String>,
    pub agent_type: String, // 'nutritionist', 'chef'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: ChatMessageId,
    pub thread_id: ChatThreadId,
    pub sender_role: MessageRole,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub id: FeedbackId,
    pub user_id: UserId,
    pub category: String, // 'chat_response', 'meal_plan', 'recipe'
    pub reference_id: Option<Uuid>,
    pub rating: i32, // 1 to 5
    pub comment: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
