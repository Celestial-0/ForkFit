use thiserror::Error;

use crate::common::error::AppError;

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("chat thread or message not found")]
    NotFound,
    #[error("unauthorized access to chat thread")]
    Unauthorized,
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("database error: {0}")]
    DatabaseError(String),
}

impl From<ChatError> for AppError {
    fn from(err: ChatError) -> Self {
        match err {
            ChatError::NotFound => AppError::NotFound,
            ChatError::Unauthorized => AppError::Forbidden,
            ChatError::ValidationError(msg) => AppError::BadRequest(msg),
            ChatError::DatabaseError(msg) => AppError::BadRequest(msg),
        }
    }
}
