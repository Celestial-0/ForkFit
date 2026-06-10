use thiserror::Error;

use crate::common::error::AppError;

#[derive(Debug, Error)]
pub enum PlanError {
    #[error("meal plan, pantry item, or shopping list not found")]
    NotFound,
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("database constraint violation: {0}")]
    DatabaseError(String),
}

impl From<PlanError> for AppError {
    fn from(err: PlanError) -> Self {
        match err {
            PlanError::NotFound => AppError::NotFound,
            PlanError::ValidationError(msg) => AppError::BadRequest(msg),
            PlanError::DatabaseError(msg) => AppError::BadRequest(msg),
        }
    }
}
