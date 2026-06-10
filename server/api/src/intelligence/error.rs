use thiserror::Error;

use crate::common::error::AppError;

#[derive(Debug, Error)]
pub enum IntelligenceError {
    #[error("intelligence resource not found")]
    NotFound,
    #[error("unauthorized access to execution trace or memory")]
    Unauthorized,
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("database error: {0}")]
    DatabaseError(String),
}

impl From<IntelligenceError> for AppError {
    fn from(err: IntelligenceError) -> Self {
        match err {
            IntelligenceError::NotFound => AppError::NotFound,
            IntelligenceError::Unauthorized => AppError::Forbidden,
            IntelligenceError::ValidationError(msg) => AppError::BadRequest(msg),
            IntelligenceError::DatabaseError(msg) => AppError::BadRequest(msg),
        }
    }
}
