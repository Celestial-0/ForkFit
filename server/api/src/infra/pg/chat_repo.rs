use sqlx::{PgPool, query_scalar};
use uuid::Uuid;
use std::str::FromStr;

use crate::common::AppResult;
use crate::common::error::AppError;
use crate::common::id::{UserId, ChatThreadId, ChatMessageId, FeedbackId};
use crate::chat::models::{ChatThread, ChatMessage, UserFeedback, MessageRole};
use crate::chat::repository::ChatRepository;

#[derive(Clone)]
pub struct PgChatRepository {
    pool: PgPool,
}

impl PgChatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ChatRepository for PgChatRepository {
    async fn create_thread(&self, thread: ChatThread) -> AppResult<ChatThread> {
        let thread_id: Uuid = thread.id.into();
        let user_uuid: Uuid = thread.user_id.into();

        let row = sqlx::query!(
            r#"
            INSERT INTO chat_threads (id, user_id, title, agent_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id as "id: ChatThreadId",
                user_id as "user_id: UserId",
                title,
                agent_type,
                created_at,
                updated_at
            "#,
            thread_id,
            user_uuid,
            thread.title,
            thread.agent_type,
            thread.created_at,
            thread.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ChatThread {
            id: row.id,
            user_id: row.user_id,
            title: row.title,
            agent_type: row.agent_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn get_thread(&self, id: ChatThreadId) -> AppResult<Option<ChatThread>> {
        let thread_uuid: Uuid = id.into();

        let row = sqlx::query!(
            r#"
            SELECT
                id as "id: ChatThreadId",
                user_id as "user_id: UserId",
                title,
                agent_type,
                created_at,
                updated_at
            FROM chat_threads
            WHERE id = $1
            "#,
            thread_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ChatThread {
            id: r.id,
            user_id: r.user_id,
            title: r.title,
            agent_type: r.agent_type,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list_threads(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<ChatThread>, u64)> {
        let user_uuid: Uuid = user_id.into();
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let total = query_scalar!(
            r#"SELECT count(*) FROM chat_threads WHERE user_id = $1"#,
            user_uuid
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id: ChatThreadId",
                user_id as "user_id: UserId",
                title,
                agent_type,
                created_at,
                updated_at
            FROM chat_threads
            WHERE user_id = $1
            ORDER BY updated_at DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_uuid,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let threads = rows
            .into_iter()
            .map(|r| ChatThread {
                id: r.id,
                user_id: r.user_id,
                title: r.title,
                agent_type: r.agent_type,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok((threads, total))
    }

    async fn delete_thread(&self, id: ChatThreadId) -> AppResult<()> {
        let thread_uuid: Uuid = id.into();

        sqlx::query!(
            "DELETE FROM chat_threads WHERE id = $1",
            thread_uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_message(&self, message: ChatMessage) -> AppResult<ChatMessage> {
        let message_id: Uuid = message.id.into();
        let thread_uuid: Uuid = message.thread_id.into();
        let role_str = message.sender_role.as_str();

        let row = sqlx::query!(
            r#"
            INSERT INTO chat_messages (id, thread_id, sender_role, content, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id as "id: ChatMessageId",
                thread_id as "thread_id: ChatThreadId",
                sender_role,
                content,
                metadata,
                created_at
            "#,
            message_id,
            thread_uuid,
            role_str,
            message.content,
            message.metadata,
            message.created_at
        )
        .fetch_one(&self.pool)
        .await?;

        // Update updated_at on the thread
        sqlx::query!(
            "UPDATE chat_threads SET updated_at = now() WHERE id = $1",
            thread_uuid
        )
        .execute(&self.pool)
        .await?;

        let sender_role = MessageRole::from_str(&row.sender_role)
            .map_err(|e| AppError::BadRequest(format!("Invalid message role in DB: {}", e)))?;

        Ok(ChatMessage {
            id: row.id,
            thread_id: row.thread_id,
            sender_role,
            content: row.content,
            metadata: row.metadata,
            created_at: row.created_at,
        })
    }

    async fn get_message(&self, id: ChatMessageId) -> AppResult<Option<ChatMessage>> {
        let message_uuid: Uuid = id.into();

        let row = sqlx::query!(
            r#"
            SELECT
                id as "id: ChatMessageId",
                thread_id as "thread_id: ChatThreadId",
                sender_role,
                content,
                metadata,
                created_at
            FROM chat_messages
            WHERE id = $1
            "#,
            message_uuid
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let sender_role = MessageRole::from_str(&r.sender_role)
                .map_err(|e| AppError::BadRequest(format!("Invalid message role in DB: {}", e)))?;
            Ok(Some(ChatMessage {
                id: r.id,
                thread_id: r.thread_id,
                sender_role,
                content: r.content,
                metadata: r.metadata,
                created_at: r.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_messages(&self, thread_id: ChatThreadId, page: u64, per_page: u64) -> AppResult<(Vec<ChatMessage>, u64)> {
        let thread_uuid: Uuid = thread_id.into();
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let total = query_scalar!(
            r#"SELECT count(*) FROM chat_messages WHERE thread_id = $1"#,
            thread_uuid
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = sqlx::query!(
            r#"
            SELECT
                id as "id: ChatMessageId",
                thread_id as "thread_id: ChatThreadId",
                sender_role,
                content,
                metadata,
                created_at
            FROM chat_messages
            WHERE thread_id = $1
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            thread_uuid,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        for r in rows {
            let sender_role = MessageRole::from_str(&r.sender_role)
                .map_err(|e| AppError::BadRequest(format!("Invalid message role in DB: {}", e)))?;
            messages.push(ChatMessage {
                id: r.id,
                thread_id: r.thread_id,
                sender_role,
                content: r.content,
                metadata: r.metadata,
                created_at: r.created_at,
            });
        }

        Ok((messages, total))
    }

    async fn create_feedback(&self, feedback: UserFeedback) -> AppResult<UserFeedback> {
        let feedback_id: Uuid = feedback.id.into();
        let user_uuid: Uuid = feedback.user_id.into();

        let row = sqlx::query!(
            r#"
            INSERT INTO user_feedbacks (id, user_id, category, reference_id, rating, comment, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id as "id: FeedbackId",
                user_id as "user_id: UserId",
                category,
                reference_id,
                rating,
                comment,
                metadata,
                created_at
            "#,
            feedback_id,
            user_uuid,
            feedback.category,
            feedback.reference_id,
            feedback.rating,
            feedback.comment,
            feedback.metadata,
            feedback.created_at
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserFeedback {
            id: row.id,
            user_id: row.user_id,
            category: row.category,
            reference_id: row.reference_id,
            rating: row.rating,
            comment: row.comment,
            metadata: row.metadata,
            created_at: row.created_at,
        })
    }
}
