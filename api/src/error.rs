use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use messaging::application::usecases::SendMessageError;
use messaging::application::services::MessageServiceError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Message not found")]
    MessageNotFound,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Service error: {0}")]
    Service(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<SendMessageError> for ApiError {
    fn from(err: SendMessageError) -> Self {
        match err {
            SendMessageError::InvalidContent(msg) => ApiError::Validation(msg),
            SendMessageError::InvalidDestination(msg) => ApiError::Validation(msg),
            SendMessageError::DomainError(err) => ApiError::Validation(err.to_string()),
            SendMessageError::RepositoryError(err) => ApiError::Database(err.to_string()),
            SendMessageError::QueueError(err) => ApiError::Service(err.to_string()),
        }
    }
}

impl From<MessageServiceError> for ApiError {
    fn from(err: MessageServiceError) -> Self {
        match err {
            MessageServiceError::MessageNotFound(_) => ApiError::MessageNotFound,
            MessageServiceError::ProcessingError(err) => ApiError::Internal(err.to_string()),
            MessageServiceError::RepositoryError(err) => ApiError::Database(err.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::MessageNotFound => (StatusCode::NOT_FOUND, "Message not found".to_string()),
            ApiError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Service(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
