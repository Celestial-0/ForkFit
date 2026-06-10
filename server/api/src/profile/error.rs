use thiserror::Error;

use crate::common::id::UserId;
use crate::common::error::AppError;

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("profile not found for user {0}")]
    NotFound(UserId),
    #[error("invalid biometric value: {0}")]
    InvalidBiometric(String),
    #[error("invalid date of birth: date cannot be in the future")]
    InvalidDateOfBirth,
    #[error("conflicting active goal for category '{0}'")]
    ConflictingGoal(String),
    #[error("unauthorized profile access")]
    Unauthorized,
    #[error("validation error: {0}")]
    ValidationError(String),
}

impl From<ProfileError> for AppError {
    fn from(err: ProfileError) -> Self {
        match err {
            ProfileError::NotFound(_) => AppError::NotFound,
            ProfileError::InvalidBiometric(msg) => AppError::BadRequest(msg),
            ProfileError::InvalidDateOfBirth => AppError::BadRequest("invalid date of birth: date cannot be in the future".to_string()),
            ProfileError::ConflictingGoal(msg) => AppError::Conflict(format!("conflicting active goal for category: {msg}")),
            ProfileError::Unauthorized => AppError::Forbidden,
            ProfileError::ValidationError(msg) => AppError::BadRequest(msg),
        }
    }
}
