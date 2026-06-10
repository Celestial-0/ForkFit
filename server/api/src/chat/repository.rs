use crate::common::AppResult;
use crate::common::id::{UserId, ChatThreadId, ChatMessageId};
use super::models::{ChatThread, ChatMessage, UserFeedback};

pub trait ChatRepository: Send + Sync {
    async fn create_thread(&self, thread: ChatThread) -> AppResult<ChatThread>;
    async fn get_thread(&self, id: ChatThreadId) -> AppResult<Option<ChatThread>>;
    async fn list_threads(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<ChatThread>, u64)>;
    async fn delete_thread(&self, id: ChatThreadId) -> AppResult<()>;
    
    async fn create_message(&self, message: ChatMessage) -> AppResult<ChatMessage>;
    async fn get_message(&self, id: ChatMessageId) -> AppResult<Option<ChatMessage>>;
    async fn list_messages(&self, thread_id: ChatThreadId, page: u64, per_page: u64) -> AppResult<(Vec<ChatMessage>, u64)>;
    
    async fn create_feedback(&self, feedback: UserFeedback) -> AppResult<UserFeedback>;
}
