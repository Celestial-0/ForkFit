use thiserror::Error;

use crate::common::error::AppError;

#[derive(Debug, Error)]
pub enum RecipeError {
    #[error("recipe or ingredient not found")]
    NotFound,
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("database constraint violation: {0}")]
    DatabaseError(String),
}

impl From<RecipeError> for AppError {
    fn from(err: RecipeError) -> Self {
        match err {
            RecipeError::NotFound => AppError::NotFound,
            RecipeError::ValidationError(msg) => AppError::BadRequest(msg),
            RecipeError::DatabaseError(msg) => AppError::BadRequest(msg),
        }
    }
}
