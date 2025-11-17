use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Entity already exists: {0}")]
    AlreadyExists(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Operation not allowed: {0}")]
    Forbidden(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

