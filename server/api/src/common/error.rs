use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("grpc connection error: {0}")]
    Tonic(#[from] tonic::transport::Error),
    #[error("grpc service error: {0}")]
    TonicStatus(#[from] tonic::Status),
}

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorMessage,
}

#[derive(Serialize)]
struct ErrorMessage {
    code: &'static str,
    message: String,
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::TonicStatus(status) => match status.code() {
                tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
                tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
                tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
                tonic::Code::NotFound => StatusCode::NOT_FOUND,
                tonic::Code::AlreadyExists => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::Sqlx(_)
            | Self::Migration(_)
            | Self::Bcrypt(_)
            | Self::Config(_)
            | Self::Redis(_)
            | Self::Tonic(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::Conflict(_) => "conflict",
            Self::Sqlx(_) => "database_error",
            Self::Migration(_) => "migration_error",
            Self::Bcrypt(_) => "password_error",
            Self::Config(_) => "configuration_error",
            Self::Redis(_) => "redis_error",
            Self::Tonic(_) => "grpc_connection_error",
            Self::TonicStatus(status) => match status.code() {
                tonic::Code::InvalidArgument => "bad_request",
                tonic::Code::Unauthenticated => "unauthorized",
                tonic::Code::PermissionDenied => "forbidden",
                tonic::Code::NotFound => "not_found",
                tonic::Code::AlreadyExists => "conflict",
                _ => "grpc_service_error",
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorBody {
            error: ErrorMessage {
                code: self.code(),
                message: self.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
